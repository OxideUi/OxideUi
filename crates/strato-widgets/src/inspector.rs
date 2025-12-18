//! In-app inspector overlay for visualizing widget trees, state snapshots, and performance timelines.

use std::collections::HashMap;

use glam::Vec2;
use strato_core::event::{Event, EventResult, KeyCode, KeyboardEvent, Modifiers};
use strato_core::inspector::{self, ComponentNodeSnapshot, InspectorSnapshot, LayoutBoxSnapshot};
use strato_core::layout::{Constraints, Layout, Size};
use strato_core::types::{Color, Rect, Transform};
use strato_renderer::batch::RenderBatch;

use crate::container::Container;
use crate::layout::Column;
use crate::scroll_view::ScrollView;
use crate::text::Text;
use crate::widget::{generate_id, Widget, WidgetId};
use slotmap::Key;

const DEFAULT_PANEL_WIDTH: f32 = 340.0;
const DEFAULT_PANEL_HEIGHT: f32 = 320.0;

/// Overlay widget that renders the inspector panel and captures instrumentation data.
#[derive(Debug)]
pub struct InspectorOverlay {
    id: WidgetId,
    child: Box<dyn Widget>,
    shortcut: (KeyCode, Modifiers),
    pub visible: bool,
    cached_child_size: Size,
    panel: Option<Box<dyn Widget>>,
    panel_size: Option<Size>,
}

impl InspectorOverlay {
    /// Create a new overlay wrapping the provided child widget.
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            id: generate_id(),
            child: Box::new(child),
            shortcut: (
                KeyCode::I,
                Modifiers {
                    control: true,
                    shift: true,
                    alt: false,
                    super_key: false,
                },
            ),
            visible: false,
            cached_child_size: Size::zero(),
            panel: None,
            panel_size: None,
        }
    }

    /// Override the keyboard shortcut used to toggle visibility.
    pub fn shortcut(mut self, key: KeyCode, modifiers: Modifiers) -> Self {
        self.shortcut = (key, modifiers);
        self
    }

    fn shortcut_pressed(&self, key: &KeyboardEvent) -> bool {
        key.key_code == self.shortcut.0
            && key.modifiers.control == self.shortcut.1.control
            && key.modifiers.shift == self.shortcut.1.shift
            && key.modifiers.alt == self.shortcut.1.alt
            && key.modifiers.super_key == self.shortcut.1.super_key
    }

    fn collect_components(
        &self,
        widget: &(dyn Widget + '_),
        depth: usize,
        nodes: &mut Vec<ComponentNodeSnapshot>,
    ) {
        nodes.push(ComponentNodeSnapshot {
            id: strato_core::widget::WidgetId(widget.id()),
            name: format!("{:?}", widget),
            depth,
            props: HashMap::new(),
            state: HashMap::new(),
        });

        for child in widget.children() {
            self.collect_components(child, depth + 1, nodes);
        }
    }

    fn build_panel(&self, snapshot: &InspectorSnapshot) -> Box<dyn Widget> {
        let mut lines: Vec<Box<dyn Widget>> = Vec::new();
        lines.push(Box::new(
            Text::new("Inspector (Ctrl+Shift+I)")
                .font_size(16.0)
                .color(Color::rgb(1.0, 1.0, 1.0)),
        ));

        lines.push(Box::new(
            Text::new("Component hierarchy")
                .font_size(14.0)
                .color(Color::rgb(0.8, 0.9, 1.0)),
        ));
        if snapshot.components.is_empty() {
            lines.push(Box::new(
                Text::new("(no widgets rendered yet)").font_size(12.0),
            ));
        } else {
            for node in &snapshot.components {
                let indent = "  ".repeat(node.depth);
                let line = format!("{}• {} #{:?}", indent, node.name, node.id);
                lines.push(Box::new(
                    Text::new(line)
                        .font_size(12.0)
                        .color(Color::rgb(0.9, 0.9, 0.9)),
                ));
            }
        }

        lines.push(Box::new(
            Text::new("State snapshots")
                .font_size(14.0)
                .color(Color::rgb(0.8, 0.9, 1.0)),
        ));
        if snapshot.state_snapshots.is_empty() {
            lines.push(Box::new(
                Text::new("(no state mutations captured)").font_size(12.0),
            ));
        } else {
            for snapshot in snapshot.state_snapshots.iter().take(8) {
                let line = format!("• {:?} => {}", snapshot.state_id.data(), snapshot.detail);
                lines.push(Box::new(Text::new(line).font_size(12.0)));
            }
        }

        lines.push(Box::new(
            Text::new("Layout boxes")
                .font_size(14.0)
                .color(Color::rgb(0.8, 0.9, 1.0)),
        ));
        lines.push(Box::new(
            Text::new(format!(
                "{} boxes captured this frame",
                snapshot.layout_boxes.len()
            ))
            .font_size(12.0),
        ));

        lines.push(Box::new(
            Text::new("Performance timeline")
                .font_size(14.0)
                .color(Color::rgb(0.8, 0.9, 1.0)),
        ));
        if snapshot.frame_timelines.is_empty() {
            lines.push(Box::new(
                Text::new("(no frames recorded yet)").font_size(12.0),
            ));
        } else {
            for frame in snapshot.frame_timelines.iter().rev().take(5) {
                let note = frame.notes.clone().unwrap_or_else(|| "".to_string());
                let line = format!(
                    "• Frame {}: {:.2}ms cpu / {:.2}ms gpu {}",
                    frame.frame_id, frame.cpu_time_ms, frame.gpu_time_ms, note
                );
                lines.push(Box::new(Text::new(line).font_size(12.0)));
            }
        }

        let column = Column::new().spacing(4.0).children(lines);
        let scrollable = ScrollView::new(column);

        Box::new(
            Container::new()
                .padding(12.0)
                .background(Color::rgba(0.08, 0.1, 0.14, 0.92))
                .border(1.0, Color::rgba(0.4, 0.6, 1.0, 0.4))
                .child(scrollable),
        )
    }
}

impl Widget for InspectorOverlay {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn clone_widget(&self) -> Box<dyn Widget> {
        // Since we can't easily clone the boxed child trait object without more bounds,
        // and InspectorOverlay is likely a singleton/special widget,
        // we might need a specific strategy.
        // For now, assuming `InspectorOverlay` needs to be Clone but `child` is `Box<dyn Widget>`.
        // `Box<dyn Widget>` isn't automatically cloneable unless `Widget` has `clone_widget`.
        // We can use the child's `clone_widget` method.
        Box::new(Self {
            id: generate_id(), // Generate new ID on clone? Or copy? Usually clone implies new ID for widgets or copy?
            // BaseWidget generates new ID.
            child: self.child.clone_widget(),
            shortcut: self.shortcut,
            visible: self.visible,
            cached_child_size: self.cached_child_size,
            panel: self.panel.as_ref().map(|p| p.clone_widget()),
            panel_size: self.panel_size,
        })
    }

    fn layout(&mut self, constraints: Constraints) -> Size {
        let inspector = inspector::inspector();
        if inspector.is_enabled() && self.visible {
            inspector.begin_frame();
            let mut nodes = Vec::new();
            self.collect_components(self.child.as_ref(), 0, &mut nodes);
            inspector.record_component_tree(nodes);
        }

        self.cached_child_size = self.child.layout(constraints);

        if inspector::inspector().is_enabled() && self.visible {
            let snapshot = inspector::inspector().snapshot();
            let mut panel = self.build_panel(&snapshot);
            let panel_constraints = Constraints {
                min_width: DEFAULT_PANEL_WIDTH,
                max_width: DEFAULT_PANEL_WIDTH,
                min_height: 0.0,
                max_height: constraints.max_height.min(DEFAULT_PANEL_HEIGHT),
            };
            self.panel_size = Some(panel.layout(panel_constraints));
            self.panel = Some(panel);
        } else {
            self.panel = None;
            self.panel_size = None;
        }

        self.cached_child_size
    }

    fn render(&self, batch: &mut RenderBatch, layout: Layout) {
        let child_layout = Layout::new(layout.position, self.cached_child_size);
        self.child.render(batch, child_layout);

        if inspector::inspector().is_enabled() && self.visible {
            inspector::inspector().record_layout_box(LayoutBoxSnapshot {
                widget_id: strato_core::widget::WidgetId(self.child.id()),
                bounds: Rect::new(
                    layout.position.x,
                    layout.position.y,
                    self.cached_child_size.width,
                    self.cached_child_size.height,
                ),
            });

            let snapshot = inspector::inspector().snapshot();
            for layout_box in &snapshot.layout_boxes {
                batch.add_rect(
                    layout_box.bounds,
                    Color::rgba(0.1, 0.7, 1.0, 0.15),
                    Transform::identity(),
                );
            }

            if let (Some(panel), Some(panel_size)) = (&self.panel, self.panel_size) {
                let panel_pos = Vec2::new(
                    layout.position.x + layout.size.width - panel_size.width - 12.0,
                    layout.position.y + 12.0,
                );
                let panel_layout = Layout::new(panel_pos, panel_size);
                panel.render(batch, panel_layout);

                inspector::inspector().record_layout_box(LayoutBoxSnapshot {
                    widget_id: strato_core::widget::WidgetId(panel.id()),
                    bounds: Rect::new(
                        panel_pos.x,
                        panel_pos.y,
                        panel_size.width,
                        panel_size.height,
                    ),
                });
            }
        }
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        if let Event::KeyDown(key) = event {
            if self.shortcut_pressed(key) {
                let now_visible = !self.visible;
                self.visible = now_visible;
                inspector::inspector().set_enabled(now_visible);
                return EventResult::Handled;
            }
        }

        self.child.handle_event(event)
    }
}
