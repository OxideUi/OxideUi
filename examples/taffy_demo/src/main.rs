use strato_core::taffy::prelude::*;
use strato_widgets::prelude::*;
use strato_platform::{ApplicationBuilder, WindowBuilder};

fn main() {
    strato_core::init();

    tracing::info!("Starting Taffy Demo");

    // 1. Create a widget tree
    // We use a Column as the root container
    let root = Column::new()
        .spacing(10.0)
        .child(Box::new(
            Row::new()
                .spacing(5.0)
                .child(Box::new(Button::new("Button 1")))
                .child(Box::new(Button::new("Button 2")))
                .child(Box::new(Button::new("Button 3")))
        ))
        .child(Box::new(
            Stack::new()
                .child(Box::new(Button::new("Overlay Button")))
        ));

    // 2. Run the application
    ApplicationBuilder::new()
        .title("Taffy Demo")
        .window(WindowBuilder::new().with_size(800.0, 600.0).resizable(true))
        .with_taffy(true) // Enable Taffy layout engine
        .run(root);
}
