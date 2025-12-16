use strato_widgets::prelude::*;
use strato_core::ui_node::{UiNode, WidgetNode, PropValue};

#[test]
fn test_view_macro_ast_generation() {
    let node = view! {
        Container {
            padding: 20.0,
            child: Text { "Snapshot" }
        }
    };

    let expected = UiNode::Widget(WidgetNode {
        name: "Container".to_string(),
        props: vec![
            ("padding".to_string(), PropValue::Float(20.0)),
        ],
        children: vec![
            UiNode::Widget(WidgetNode {
                name: "Text".to_string(),
                props: vec![
                    ("text".to_string(), PropValue::String("Snapshot".to_string()))
                ],
                children: vec![]
            })
        ],
    });

    assert_eq!(node, expected);
}

#[test]
fn test_view_macro_nested() {
    let node = view! {
        Column {
            spacing: 10.0,
            children: [
                Text { "A" },
                Text { "B" }
            ]
        }
    };
    
    // Note: implementation details of macro might change ordering or exact structure,
    // so this snapshot verifies the CURRENT behavior.
    
    // In current macro:
    // props: "spacing": 10.0 (Float)
    // children: explicit children list
    
    match node {
        UiNode::Widget(n) => {
            assert_eq!(n.name, "Column");
            assert!(n.props.contains(&("spacing".to_string(), PropValue::Float(10.0))));
            assert_eq!(n.children.len(), 2);
            
            // Verify children are Text Widgets
            if let UiNode::Widget(t1) = &n.children[0] {
                assert_eq!(t1.name, "Text");
                assert!(t1.props.contains(&("text".to_string(), PropValue::String("A".to_string()))));
            } else { panic!("child 0 not widget") }
            
             if let UiNode::Widget(t2) = &n.children[1] {
                assert_eq!(t2.name, "Text");
                assert!(t2.props.contains(&("text".to_string(), PropValue::String("B".to_string()))));
            } else { panic!("child 1 not widget") }
        }
        _ => panic!("Expected Widget node"),
    }
}
