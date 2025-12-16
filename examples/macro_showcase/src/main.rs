use strato_platform::{ApplicationBuilder, WindowBuilder};
use strato_widgets::prelude::*;
use strato_core::types::Color;
use strato_macros::view;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let builder = ApplicationBuilder::new()
        .title("Macro DSL Showcase")
        .window(WindowBuilder::new().with_size(800.0, 600.0));
    
    // Using the view! macro to declaratively build the UI
    let root = view! {
        Container {
            padding: 20.0,
            background: Color::WHITE,
            child: Column {
                    spacing: 15.0,
                    children: [
                        Text { "Declarative UI with Macros!" },
                        Container {
                            padding: 10.0,
                            background: Color::rgba(0.9, 0.9, 0.9, 1.0),
                            child: Text { "This entire structure is built using the view! macro." }
                        },
                        Row {
                            spacing: 10.0,
                            children: [
                                Button { "Cancel" },
                                Button { "Submit" }
                            ]
                        }
                    ]
                }
        }
    };

    // Create legacy registry
    let registry = strato_widgets::registry::create_default_registry();

    // Build the widget tree using the registry
    let root_widget = registry.build(root);

    builder.run(root_widget);
}
