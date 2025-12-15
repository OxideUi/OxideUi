//! Counter example demonstrating state management in StratoUI

use strato_core::{
    types::Color,
    error::Result,
};
use strato_widgets::prelude::*;
use strato_widgets::ButtonStyle;
use strato_platform::{ApplicationBuilder, WindowBuilder};
use std::sync::Arc;
use parking_lot::RwLock;

struct CounterApp {
    count: Arc<RwLock<i32>>,
}

impl CounterApp {
    fn new() -> Self {
        Self {
            count: Arc::new(RwLock::new(0)),
        }
    }

    fn increment(&self) {
        *self.count.write() += 1;
    }

    fn decrement(&self) {
        *self.count.write() -= 1;
    }

    fn reset(&self) {
        *self.count.write() = 0;
    }
}

fn main() -> Result<()> {
    // Initialize all StratoUI modules
    strato_core::init()?;
    strato_widgets::init()?;
    strato_platform::init().map_err(|e| strato_core::error::StratoError::platform(format!("{:?}", e)))?;
    
    let app = Arc::new(CounterApp::new());

    // Create and run the application
    ApplicationBuilder::new()
        .title("Counter Example")
        .window(
            WindowBuilder::new()
                .with_size(350.0, 250.0)
                .resizable(false)
        )
        .run(build_ui(app));
    
    Ok(())
}

fn build_ui(app: Arc<CounterApp>) -> impl Widget {
    let app_inc = app.clone();
    let app_dec = app.clone();
    let app_reset = app.clone();

    Container::new()
        .background(Color::WHITE)
        .padding(30.0)
        .child(
            Column::new()
                .spacing(20.0)
                .main_axis_alignment(MainAxisAlignment::Center)
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(vec![
                    Box::new(
                        Text::new("Counter App")
                            .size(28.0)
                            .color(Color::rgb(0.2, 0.2, 0.2))
                    ),
                    Box::new(
                        Container::new()
                            .padding(20.0)
                            .background(Color::rgba(0.0, 0.0, 0.0, 0.05))
                            .border_radius(8.0)
                            .child(
                                Text::new(format!("{}", *app.count.read()))
                                    .size(48.0)
                                    .color(Color::BLACK)
                            )
                    ),
                    Box::new(
                        Row::new()
                            .spacing(10.0)
                            .main_axis_alignment(MainAxisAlignment::Center)
                            .children(vec![
                                Box::new(
                                    Button::new("-")
                                        .style(ButtonStyle::secondary())
                                        .size(50.0, 40.0)
                                        .on_click(move || {
                                            app_dec.decrement();
                                            println!("Count: {}", *app_dec.count.read());
                                        })
                                ),
                                Box::new(
                                    Button::new("Reset")
                                        .style(ButtonStyle::text())
                                        .on_click(move || {
                                            app_reset.reset();
                                            println!("Count reset to 0");
                                        })
                                ),
                                Box::new(
                                    Button::new("+")
                                        .style(ButtonStyle::primary())
                                        .size(50.0, 40.0)
                                        .on_click(move || {
                                            app_inc.increment();
                                            println!("Count: {}", *app_inc.count.read());
                                        })
                                ),
                            ])
                    ),
                ])
        )
}
