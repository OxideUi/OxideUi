use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use strato_core::ui_node::{UiNode, WidgetNode, PropValue};
use crate::widget::{Widget, WidgetContext, WidgetId};
use strato_core::event::{Event, EventResult};
use strato_core::layout::{Constraints, Layout, Size};
use strato_core::types::Point;
use strato_renderer::batch::RenderBatch;
use crate::prelude::*;
use crate::image::{Image, ImageSource, ImageFit};

/// A builder function that creates a widget from properties.
type WidgetBuilder = Box<dyn Fn(Vec<(String, PropValue)>, Vec<UiNode>, &WidgetRegistry) -> Box<dyn Widget> + Send + Sync>;

/// Registry for mapping widget names to their constructors.
pub struct WidgetRegistry {
    builders: HashMap<String, WidgetBuilder>,
}

/// Wrapper to allow Box<dyn Widget> to satisfy impl Widget
#[derive(Debug)]
pub struct BoxedWidget(pub Box<dyn Widget>);

impl Widget for BoxedWidget {
    fn id(&self) -> WidgetId { self.0.id() }
    
    fn layout(&mut self, constraints: Constraints) -> Size {
        self.0.layout(constraints)
    }
    
    fn render(&self, batch: &mut RenderBatch, layout: Layout) {
        self.0.render(batch, layout)
    }
    
    fn handle_event(&mut self, event: &Event) -> EventResult {
        self.0.handle_event(event)
    }
    
    fn update(&mut self, ctx: &WidgetContext) {
        self.0.update(ctx)
    }
    
    fn children(&self) -> Vec<&(dyn Widget + '_)> { self.0.children() }
    fn children_mut(&mut self) -> Vec<&mut (dyn Widget + '_)> { self.0.children_mut() }
    
    fn hit_test(&self, point: Point, layout: Layout) -> bool {
        self.0.hit_test(point, layout)
    }
    
    fn as_any(&self) -> &dyn std::any::Any { self.0.as_any() }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self.0.as_any_mut() }
    
    fn clone_widget(&self) -> Box<dyn Widget> {
        self.0.clone_widget()
    }
}

impl WidgetRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            builders: HashMap::new(),
        };
        registry.register_defaults();
        registry
    }

    /// Register a widget builder.
    pub fn register<F>(&mut self, name: &str, builder: F)
    where
        F: Fn(Vec<(String, PropValue)>, Vec<UiNode>, &WidgetRegistry) -> Box<dyn Widget> + Send + Sync + 'static,
    {
        self.builders.insert(name.to_string(), Box::new(builder));
    }

    /// Build a widget tree from a UiNode.
    pub fn build(&self, node: UiNode) -> BoxedWidget {
        BoxedWidget(match node {
            UiNode::Widget(node) => self.build_widget(node),
            UiNode::Text(text) => Box::new(Text::new(text)),
            UiNode::Fragment(children) => {
                 let mut col = Column::new();
                 for child in children {
                     col = col.child(self.build(child).0);
                 }
                 Box::new(col)
            }
        })
    }
    
    fn build_widget(&self, node: WidgetNode) -> Box<dyn Widget> {
        if let Some(builder) = self.builders.get(&node.name) {
            builder(node.props, node.children, self)
        } else {
            // Fallback for unknown widgets - maybe a red text?
            Box::new(Text::new(format!("Unknown widget: {}", node.name)))
        }
    }

    fn register_defaults(&mut self) {
        // Container
        self.register("Container", |props, children, registry| {
            let mut widget = Container::new();
            for (name, value) in props {
                match (name.as_str(), value) {
                    ("padding", PropValue::Float(v)) => widget = widget.padding(v as f32),
                     ("background", PropValue::Color(c)) => widget = widget.background(c),
                     ("width", PropValue::Float(v)) => widget = widget.width(v as f32),
                     ("height", PropValue::Float(v)) => widget = widget.height(v as f32),
                     ("radius", PropValue::Float(v)) => widget = widget.border_radius(v as f32),
                     ("margin", PropValue::Float(v)) => widget = widget.margin(v as f32),
                    _ => {}
                }
            }
             // Handle "child" logic (Container takes 1 child usually, but our generic AST has list)
            if let Some(first) = children.first() {
                widget = widget.child(registry.build(first.clone()));
            }
            Box::new(widget)
        });

        // Column
        self.register("Column", |props, children, registry| {
            let mut widget = Column::new();
            for (name, value) in props {
                if name == "spacing" {
                    if let PropValue::Float(v) = value {
                        widget = widget.spacing(v as f32);
                    }
                }
            }
            // Column::children takes Vec<Box<dyn Widget>>. 
            // registry.build returns BoxedWidget. 
            // We need to unwrap or map.
            let child_widgets: Vec<Box<dyn Widget>> = children.into_iter()
                .map(|child| registry.build(child).0)
                .collect();
                
            widget = widget.children(child_widgets);
            Box::new(widget)
        });

        // Row
        self.register("Row", |props, children, registry| {
            let mut widget = Row::new();
            for (name, value) in props {
                 if name == "spacing" {
                    if let PropValue::Float(v) = value {
                        widget = widget.spacing(v as f32);
                    }
                }
            }
            let child_widgets: Vec<Box<dyn Widget>> = children.into_iter()
                .map(|child| registry.build(child).0)
                .collect();
            widget = widget.children(child_widgets);
            Box::new(widget)
        });

        // Text
        self.register("Text", |props, _children, _registry| {
            let mut text = String::new();
            
            // First pass: find text semantic prop
            for (name, value) in &props {
                 if name == "text" {
                    if let PropValue::String(s) = value {
                        text = s.clone();
                    }
                 }
            }
            
            let mut widget = Text::new(text);
            
            // Second pass: apply properties
            for (name, value) in props { 
                match (name.as_str(), value) {
                    ("color", PropValue::Color(c)) => widget = widget.color(c),
                     ("size", PropValue::Float(v)) => widget = widget.size(v as f32),
                    _ => {}
                }
             }

            Box::new(widget)
        });

        // Button
        self.register("Button", |props, _children, _registry| {
            let mut label = String::new();
             for (name, value) in &props {
                if name == "text" {
                    if let PropValue::String(s) = value {
                        label = s.clone();
                    }
                }
            }
            
            let widget = Button::new(label);
            
            // Button usually doesn't take children in this framework, just text in constructor?
            // But macro might support `Button { child: Icon }`?
            // Existing Button::new implementation takes string.
            // If children present, ignored? or fallback?
            
            for (name, value) in props {
                 match (name.as_str(), value) {
                     // disabled?

                     // events?
                    _ => {}
                }
            }
            Box::new(widget)
        });
        // Image
        self.register("Image", |props, _children, _registry| {
            let mut source = ImageSource::Placeholder { 
                width: 100, 
                height: 100, 
                color: Color::GRAY 
            };
            
            for (name, value) in &props {
                if name == "source" {
                    if let PropValue::String(s) = value {
                        // Simple heuristic for source type
                        if s.starts_with("http") {
                             source = ImageSource::Url(s.clone());
                        } else if s.starts_with("placeholder") {
                            // Format: placeholder:width:height:hex
                            // Simplified parsing for now: placeholder -> default
                        } else {
                             source = ImageSource::File(std::path::PathBuf::from(s));
                        }
                    }
                }
            }
            
            let mut widget = Image::new(source);
             for (name, value) in props {
                match (name.as_str(), value) {
                    ("fit", PropValue::String(s)) => {
                        let fit = match s.as_str() {
                            "cover" => ImageFit::Cover,
                            "contain" => ImageFit::Contain,
                            "fill" => ImageFit::Fill,
                            _ => ImageFit::None,
                        };
                        widget = widget.fit(fit);
                    }
                    ("opacity", PropValue::Float(v)) => widget = widget.opacity(v as f32),
                     ("radius", PropValue::Float(v)) => widget = widget.border_radius(v as f32),
                    _ => {}
                }
             }
             Box::new(widget)
        });

        // TopBar
        self.register("TopBar", |props, _children, _registry| {
            let mut title = "".to_string();
            // First pass for title
            for (name, value) in &props {
                if name == "title" {
                    if let PropValue::String(s) = value {
                        title = s.clone();
                    }
                }
            }

            let mut widget = crate::top_bar::TopBar::new(title);
            
            for (name, value) in props {
                match (name.as_str(), value) {
                    ("background", PropValue::Color(c)) => widget = widget.with_background(c),
                     // height handles directly field access if needed, or via method if added
                    _ => {}
                }
            }
            Box::new(widget)
        });
    }
}

// Global registry instance (lazy static approach usually, but here we instantiate it)
pub fn create_default_registry() -> WidgetRegistry {
    WidgetRegistry::new()
}
