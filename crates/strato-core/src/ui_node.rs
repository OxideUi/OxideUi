
use crate::types::Color;

/// A node in the semantic UI tree.
/// This decouples the description of the UI from its runtime instantiation.
#[derive(Debug, Clone, PartialEq)]
pub enum UiNode {
    /// A widget with a name, properties, and children.
    Widget(WidgetNode),
    /// A text node.
    Text(String),
    /// A container for multiple nodes.
    Fragment(Vec<UiNode>),
}

/// Description of a widget.
#[derive(Debug, Clone, PartialEq)]
pub struct WidgetNode {
    /// The name of the widget (e.g., "Container", "Button").
    pub name: String,
    /// Properties configuration.
    pub props: Vec<(String, PropValue)>,
    /// Child nodes.
    pub children: Vec<UiNode>,
}

/// Value of a property.
/// Note: `Any` is restricted to callbacks and runtime handles.
#[derive(Debug)]
pub enum PropValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Color(Color),
    // Callbacks or IDs can be added here explicitly, e.g. Callback(usize)
}

impl PartialEq for PropValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PropValue::String(a), PropValue::String(b)) => a == b,
            (PropValue::Int(a), PropValue::Int(b)) => a == b,
            (PropValue::Float(a), PropValue::Float(b)) => a == b,
            (PropValue::Bool(a), PropValue::Bool(b)) => a == b,
            (PropValue::Color(a), PropValue::Color(b)) => a == b,
            _ => false,
        }
    }
}

impl Clone for PropValue {
    fn clone(&self) -> Self {
        match self {
            PropValue::String(s) => PropValue::String(s.clone()),
            PropValue::Int(i) => PropValue::Int(*i),
            PropValue::Float(f) => PropValue::Float(*f),
            PropValue::Bool(b) => PropValue::Bool(*b),
            PropValue::Color(c) => PropValue::Color(*c),
        }
    }
}

// Helpers for manual construction if needed (though macro handles this)
impl UiNode {
    pub fn widget(name: impl Into<String>) -> Self {
        UiNode::Widget(WidgetNode {
            name: name.into(),
            props: Vec::new(),
            children: Vec::new(),
        })
    }

    pub fn text(text: impl Into<String>) -> Self {
        UiNode::Text(text.into())
    }
}

impl WidgetNode {
    pub fn prop(mut self, name: impl Into<String>, value: PropValue) -> Self {
        self.props.push((name.into(), value));
        self
    }

    pub fn child(mut self, node: UiNode) -> Self {
        self.children.push(node);
        self
    }
}

// Initial `From` implementations for easy conversion in macro generation
impl From<String> for PropValue {
    fn from(v: String) -> Self { PropValue::String(v) }
}
impl From<&str> for PropValue {
    fn from(v: &str) -> Self { PropValue::String(v.to_string()) }
}
impl From<i64> for PropValue {
    fn from(v: i64) -> Self { PropValue::Int(v) }
}
impl From<i32> for PropValue {
    fn from(v: i32) -> Self { PropValue::Int(v as i64) }
}
impl From<f64> for PropValue {
    fn from(v: f64) -> Self { PropValue::Float(v) }
}
impl From<f32> for PropValue {
    fn from(v: f32) -> Self { PropValue::Float(v as f64) }
}
impl From<bool> for PropValue {
    fn from(v: bool) -> Self { PropValue::Bool(v) }
}
impl From<Color> for PropValue {
    fn from(v: Color) -> Self { PropValue::Color(v) }
}
