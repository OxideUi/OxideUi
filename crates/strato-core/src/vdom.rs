//! Virtual DOM implementation with efficient diffing

use crate::types::NodeId;
use std::collections::HashMap;
use std::fmt;

/// Virtual DOM node types
#[derive(Debug, Clone, PartialEq)]
pub enum VNode {
    /// Element node with tag, attributes, and children
    Element {
        tag: String,
        attributes: HashMap<String, String>,
        children: Vec<VNode>,
        key: Option<String>,
    },
    /// Text node with content
    Text(String),
    /// Component node with props
    Component {
        name: String,
        props: HashMap<String, String>,
        children: Vec<VNode>,
        key: Option<String>,
    },
    /// Fragment node (container for multiple children)
    Fragment(Vec<VNode>),
}

impl VNode {
    /// Create a new element node
    pub fn element(tag: impl Into<String>) -> Self {
        VNode::Element {
            tag: tag.into(),
            attributes: HashMap::new(),
            children: Vec::new(),
            key: None,
        }
    }

    /// Create a new text node
    pub fn text(content: impl Into<String>) -> Self {
        VNode::Text(content.into())
    }

    /// Create a new component node
    pub fn component(name: impl Into<String>) -> Self {
        VNode::Component {
            name: name.into(),
            props: HashMap::new(),
            children: Vec::new(),
            key: None,
        }
    }

    /// Create a fragment node
    pub fn fragment(children: Vec<VNode>) -> Self {
        VNode::Fragment(children)
    }

    /// Set an attribute on an element
    pub fn attr(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        match &mut self {
            VNode::Element { attributes, .. } => {
                attributes.insert(key.into(), value.into());
            }
            VNode::Component { props, .. } => {
                props.insert(key.into(), value.into());
            }
            _ => {}
        }
        self
    }

    /// Set a key for the node (used in diffing)
    pub fn key(mut self, key: impl Into<String>) -> Self {
        match &mut self {
            VNode::Element { key: k, .. } | VNode::Component { key: k, .. } => {
                *k = Some(key.into());
            }
            _ => {}
        }
        self
    }

    /// Add children to the node
    pub fn children(mut self, children: Vec<VNode>) -> Self {
        match &mut self {
            VNode::Element { children: c, .. } | VNode::Component { children: c, .. } => {
                *c = children;
            }
            VNode::Fragment(c) => {
                *c = children;
            }
            _ => {}
        }
        self
    }

    /// Add a single child to the node
    pub fn child(mut self, child: VNode) -> Self {
        match &mut self {
            VNode::Element { children, .. } | VNode::Component { children, .. } => {
                children.push(child);
            }
            VNode::Fragment(children) => {
                children.push(child);
            }
            _ => {}
        }
        self
    }

    /// Get the key of the node
    pub fn get_key(&self) -> Option<&str> {
        match self {
            VNode::Element { key, .. } | VNode::Component { key, .. } => key.as_deref(),
            _ => None,
        }
    }

    /// Get the tag name for element nodes
    pub fn get_tag(&self) -> Option<&str> {
        match self {
            VNode::Element { tag, .. } => Some(tag),
            _ => None,
        }
    }

    /// Get the component name for component nodes
    pub fn get_component_name(&self) -> Option<&str> {
        match self {
            VNode::Component { name, .. } => Some(name),
            _ => None,
        }
    }

    /// Get text content for text nodes
    pub fn get_text(&self) -> Option<&str> {
        match self {
            VNode::Text(content) => Some(content),
            _ => None,
        }
    }

    /// Get children of the node
    pub fn get_children(&self) -> &[VNode] {
        match self {
            VNode::Element { children, .. } | VNode::Component { children, .. } => children,
            VNode::Fragment(children) => children,
            VNode::Text(_) => &[],
        }
    }

    /// Get mutable children for element, component, and fragment nodes
    pub fn get_children_mut(&mut self) -> &mut Vec<VNode> {
        match self {
            VNode::Element { children, .. } | VNode::Component { children, .. } => children,
            VNode::Fragment(children) => children,
            VNode::Text(_) => {
                // This method shouldn't be called on Text nodes anyway
                panic!("Cannot get mutable children from Text node")
            }
        }
    }

    /// Get attributes for element nodes
    pub fn get_attributes(&self) -> Option<&HashMap<String, String>> {
        match self {
            VNode::Element { attributes, .. } => Some(attributes),
            _ => None,
        }
    }

    /// Get props for component nodes
    pub fn get_props(&self) -> Option<&HashMap<String, String>> {
        match self {
            VNode::Component { props, .. } => Some(props),
            _ => None,
        }
    }
}

impl fmt::Display for VNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VNode::Element {
                tag,
                attributes,
                children,
                ..
            } => {
                write!(f, "<{}", tag)?;
                for (key, value) in attributes {
                    write!(f, " {}=\"{}\"", key, value)?;
                }
                if children.is_empty() {
                    write!(f, " />")
                } else {
                    write!(f, ">")?;
                    for child in children {
                        write!(f, "{}", child)?;
                    }
                    write!(f, "</{}>", tag)
                }
            }
            VNode::Text(content) => write!(f, "{}", content),
            VNode::Component {
                name,
                props,
                children,
                ..
            } => {
                write!(f, "<{}", name)?;
                for (key, value) in props {
                    write!(f, " {}=\"{}\"", key, value)?;
                }
                if children.is_empty() {
                    write!(f, " />")
                } else {
                    write!(f, ">")?;
                    for child in children {
                        write!(f, "{}", child)?;
                    }
                    write!(f, "</{}>", name)
                }
            }
            VNode::Fragment(children) => {
                for child in children {
                    write!(f, "{}", child)?;
                }
                Ok(())
            }
        }
    }
}

/// Diff operation types
#[derive(Debug, Clone, PartialEq)]
pub enum DiffOp {
    /// Insert a new node at the given index
    Insert { index: usize, node: VNode },
    /// Remove a node at the given index
    Remove { index: usize },
    /// Replace a node at the given index
    Replace { index: usize, node: VNode },
    /// Update attributes of a node
    UpdateAttributes {
        index: usize,
        attributes: HashMap<String, Option<String>>, // None means remove attribute
    },
    /// Update text content
    UpdateText { index: usize, text: String },
    /// Move a node from one index to another
    Move { from: usize, to: usize },
}

/// Virtual DOM differ
pub struct VDomDiffer {
    /// Current virtual DOM tree
    current: Option<VNode>,
}

impl VDomDiffer {
    /// Create a new VDom differ
    pub fn new() -> Self {
        Self { current: None }
    }

    /// Diff two virtual DOM trees and return the operations needed to transform old to new
    pub fn diff(&mut self, new_tree: VNode) -> Vec<DiffOp> {
        let ops = match &self.current {
            Some(old_tree) => self.diff_nodes(old_tree, &new_tree, 0),
            None => vec![DiffOp::Insert {
                index: 0,
                node: new_tree.clone(),
            }],
        };

        self.current = Some(new_tree);
        ops
    }

    /// Diff two individual nodes
    fn diff_nodes(&self, old: &VNode, new: &VNode, index: usize) -> Vec<DiffOp> {
        let mut ops = Vec::new();

        match (old, new) {
            // Both are text nodes
            (VNode::Text(old_text), VNode::Text(new_text)) => {
                if old_text != new_text {
                    ops.push(DiffOp::UpdateText {
                        index,
                        text: new_text.clone(),
                    });
                }
            }

            // Both are elements with the same tag
            (
                VNode::Element {
                    tag: old_tag,
                    attributes: old_attrs,
                    children: old_children,
                    ..
                },
                VNode::Element {
                    tag: new_tag,
                    attributes: new_attrs,
                    children: new_children,
                    ..
                },
            ) if old_tag == new_tag => {
                // Diff attributes
                let attr_diff = self.diff_attributes(old_attrs, new_attrs);
                if !attr_diff.is_empty() {
                    ops.push(DiffOp::UpdateAttributes {
                        index,
                        attributes: attr_diff,
                    });
                }

                // Diff children
                ops.extend(self.diff_children(old_children, new_children, index));
            }

            // Both are components with the same name
            (
                VNode::Component {
                    name: old_name,
                    props: old_props,
                    children: old_children,
                    ..
                },
                VNode::Component {
                    name: new_name,
                    props: new_props,
                    children: new_children,
                    ..
                },
            ) if old_name == new_name => {
                // Diff props (treated like attributes)
                let prop_diff = self.diff_attributes(old_props, new_props);
                if !prop_diff.is_empty() {
                    ops.push(DiffOp::UpdateAttributes {
                        index,
                        attributes: prop_diff,
                    });
                }

                // Diff children
                ops.extend(self.diff_children(old_children, new_children, index));
            }

            // Both are fragments
            (VNode::Fragment(old_children), VNode::Fragment(new_children)) => {
                ops.extend(self.diff_children(old_children, new_children, index));
            }

            // Different node types or different tags/names - replace entirely
            _ => {
                ops.push(DiffOp::Replace {
                    index,
                    node: new.clone(),
                });
            }
        }

        ops
    }

    /// Diff attributes/props
    fn diff_attributes(
        &self,
        old_attrs: &HashMap<String, String>,
        new_attrs: &HashMap<String, String>,
    ) -> HashMap<String, Option<String>> {
        let mut diff = HashMap::new();

        // Check for new or changed attributes
        for (key, new_value) in new_attrs {
            match old_attrs.get(key) {
                Some(old_value) if old_value != new_value => {
                    diff.insert(key.clone(), Some(new_value.clone()));
                }
                None => {
                    diff.insert(key.clone(), Some(new_value.clone()));
                }
                _ => {} // No change
            }
        }

        // Check for removed attributes
        for key in old_attrs.keys() {
            if !new_attrs.contains_key(key) {
                diff.insert(key.clone(), None);
            }
        }

        diff
    }

    /// Diff children using a keyed diffing algorithm
    fn diff_children(
        &self,
        old_children: &[VNode],
        new_children: &[VNode],
        parent_index: usize,
    ) -> Vec<DiffOp> {
        let mut ops = Vec::new();

        // Simple algorithm for now - can be optimized with keyed diffing later
        let old_len = old_children.len();
        let new_len = new_children.len();
        let min_len = old_len.min(new_len);

        // Diff existing children
        for i in 0..min_len {
            ops.extend(self.diff_nodes(&old_children[i], &new_children[i], parent_index + i + 1));
        }

        // Handle length differences
        if new_len > old_len {
            // Insert new children
            for i in old_len..new_len {
                ops.push(DiffOp::Insert {
                    index: parent_index + i + 1,
                    node: new_children[i].clone(),
                });
            }
        } else if old_len > new_len {
            // Remove extra children (in reverse order to maintain indices)
            for i in (new_len..old_len).rev() {
                ops.push(DiffOp::Remove {
                    index: parent_index + i + 1,
                });
            }
        }

        ops
    }

    /// Get the current virtual DOM tree
    pub fn current(&self) -> Option<&VNode> {
        self.current.as_ref()
    }
}

impl Default for VDomDiffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Virtual DOM tree for managing the entire UI state
pub struct VDomTree {
    /// Root node of the tree
    root: Option<VNode>,
    /// Differ for computing changes
    differ: VDomDiffer,
    /// Node ID counter
    next_id: NodeId,
}

impl VDomTree {
    /// Create a new virtual DOM tree
    pub fn new() -> Self {
        Self {
            root: None,
            differ: VDomDiffer::new(),
            next_id: NodeId(0),
        }
    }

    /// Update the tree with a new root node and return diff operations
    pub fn update(&mut self, new_root: VNode) -> Vec<DiffOp> {
        let ops = self.differ.diff(new_root.clone());
        self.root = Some(new_root);
        ops
    }

    /// Get the current root node
    pub fn root(&self) -> Option<&VNode> {
        self.root.as_ref()
    }

    /// Generate a new unique node ID
    pub fn next_node_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id.0 += 1;
        id
    }

    /// Render the tree to a string (for debugging)
    pub fn render_to_string(&self) -> String {
        match &self.root {
            Some(node) => format!("{}", node),
            None => String::new(),
        }
    }
}

impl Default for VDomTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vnode_creation() {
        let node = VNode::element("div")
            .attr("class", "container")
            .child(VNode::text("Hello, World!"));

        assert_eq!(node.get_tag(), Some("div"));
        assert_eq!(node.get_children().len(), 1);
        assert_eq!(node.get_children()[0].get_text(), Some("Hello, World!"));
    }

    #[test]
    fn test_vdom_diff_text_change() {
        let mut differ = VDomDiffer::new();

        let old_tree = VNode::text("Hello");
        let new_tree = VNode::text("World");

        differ.current = Some(old_tree);
        let ops = differ.diff(new_tree);

        assert_eq!(ops.len(), 1);
        match &ops[0] {
            DiffOp::UpdateText { text, .. } => assert_eq!(text, "World"),
            _ => panic!("Expected UpdateText operation"),
        }
    }

    #[test]
    fn test_vdom_diff_attribute_change() {
        let mut differ = VDomDiffer::new();

        let old_tree = VNode::element("div").attr("class", "old");
        let new_tree = VNode::element("div").attr("class", "new");

        differ.current = Some(old_tree);
        let ops = differ.diff(new_tree);

        assert_eq!(ops.len(), 1);
        match &ops[0] {
            DiffOp::UpdateAttributes { attributes, .. } => {
                assert_eq!(attributes.get("class"), Some(&Some("new".to_string())));
            }
            _ => panic!("Expected UpdateAttributes operation"),
        }
    }

    #[test]
    fn test_vdom_tree_update() {
        let mut tree = VDomTree::new();

        let root = VNode::element("div").child(VNode::text("Hello"));

        let ops = tree.update(root);
        assert_eq!(ops.len(), 1);

        match &ops[0] {
            DiffOp::Insert { node, .. } => {
                assert_eq!(node.get_tag(), Some("div"));
            }
            _ => panic!("Expected Insert operation"),
        }
    }
}
