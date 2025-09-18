//! Hello World example for OxideUI

use oxide_ui::prelude::*;

fn main() -> Result<()> {
    // Initialize all OxideUI modules
    oxide_ui::init_all()?;
    
    //Build and run the application
    ApplicationBuilder::new()
        .title("Hello OxideUI")
        .window(WindowBuilder::new().with_size(400.0, 300.0).resizable(true))
        .run(build_ui());
    

    
}

fn build_ui() -> impl Widget {
    Container::new()
        .background(Color::rgb(0.2, 0.2, 0.8))  // Blue background instead of gray
        .padding(20.0)
        .child(
            Column::new()
                .spacing(20.0)
                .main_axis_alignment(MainAxisAlignment::Center)
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(vec![
                    Box::new(
                        Text::new("Hello, OxideUI!")
                            .size(32.0)
                            .color(Color::rgb(1.0, 1.0, 0.0))  // Yellow text
                    ),
                    Box::new(
                        Text::new("Welcome to the future of UI development!")
                            .size(20.0)
                            .color(Color::rgb(0.0, 1.0, 0.0))  // Green text
                    ),
                    Box::new(
                        Button::new("Click me!")
                            .on_click(|| {
                                println!("Button clicked!");
                            })
                    ),
                ])
        )
}
