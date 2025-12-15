//! Hello World example for StratoSDK showing state management and modern UI
use strato_sdk::prelude::*;
use strato_sdk::strato_widgets::{
    Button, Column, Container, Text,
    text::TextAlign,
    layout::{MainAxisAlignment, CrossAxisAlignment},
};
use strato_sdk::strato_platform::{
    ApplicationBuilder, WindowBuilder,
    init::{InitBuilder, InitConfig},
};
use strato_sdk::strato_core::types::Color;
use strato_sdk::strato_core::state::Signal;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
struct HelloWorldState {
    counter: Signal<i32>,
    message: Signal<String>,
}

impl Default for HelloWorldState {
    fn default() -> Self {
        Self {
            counter: Signal::new(0),
            message: Signal::new("Welcome to StratoSDK!".to_string()),
        }
    }
}

impl HelloWorldState {
    fn increment(&mut self) {
        self.counter.update(|c| *c += 1);
        let count = self.counter.get();
        if count == 1 {
            self.message.set("First click! Keep going!".to_string());
        } else if count == 5 {
             self.message.set("You're getting the hang of it!".to_string());
        } else if count == 10 {
             self.message.set("Double digits! 🚀".to_string());
        }
    }
}

fn main() -> anyhow::Result<()> {
    // Initialize StratoSDK
    InitBuilder::new()
        .with_config(InitConfig {
            enable_logging: true,
            ..Default::default()
        })
        .init_all()?;

    println!("Hello World - StratoSDK initialized!");

    ApplicationBuilder::new()
        .title("Hello StratoSDK")
        .window(WindowBuilder::new().with_size(500.0, 400.0).resizable(true))
        .run(build_ui())
}

fn build_ui() -> impl Widget {
    let state = Arc::new(Mutex::new(HelloWorldState::default()));
    
    // Main Window Background
    Container::new()
        .background(Color::rgb(0.05, 0.05, 0.05)) // Deep dark background
        .child(
            // Center content using Column alignment
            Column::new()
                .main_axis_alignment(MainAxisAlignment::Center)
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(vec![
                    Box::new(create_interaction_card(state))
                ])
        )
}

fn create_interaction_card(state: Arc<Mutex<HelloWorldState>>) -> impl Widget {
    let (counter_signal, message_signal) = {
        let state = state.lock().unwrap();
        (state.counter.clone(), state.message.clone())
    };
    
    // Card Container
    Container::new()
        .background(Color::rgb(0.12, 0.12, 0.12)) // Slightly lighter card
        .width(350.0)
        // .border_radius(16.0) // If/when available
        .padding(30.0)
        .child(
            Column::new()
                .spacing(25.0)
                .cross_axis_alignment(CrossAxisAlignment::Center) // Center children horizontally
                .children(vec![
                    // Icon / Logo Placeholder (Circle)
                    // Centering inside a container via Column again if needed, or just container with size
                    Box::new(
                        Container::new()
                            .width(60.0)
                            .height(60.0)
                            .background(Color::rgb(0.25, 0.4, 0.9)) // Accent Blue
                            // .corner_radius(30.0) // Circle
                    ),
                    
                    // Title
                    Box::new(
                        Text::new("Hello, StratoSDK")
                            .size(32.0)
                            .color(Color::WHITE)
                            .align(TextAlign::Center)
                    ),
                    
                    // Dynamic Message
                    Box::new(
                        Text::new("")
                            .bind(message_signal)
                            .size(16.0)
                            .color(Color::rgb(0.7, 0.7, 0.7))
                            .align(TextAlign::Center)
                    ),
                    
                    // Spacer
                    Box::new(Container::new().height(10.0)),
                    
                    // Counter Display
                    Box::new(
                        Text::new("")
                            .bind(counter_signal.map(|c| format!("Clicks: {}", c)))
                            .size(48.0)
                            .color(Color::rgb(0.25, 0.8, 0.4)) // Green accent
                            .align(TextAlign::Center)
                    ),
                    
                    // Interactive Button
                    Box::new(
                         Button::new("Increment Counter")
                            .on_click(move || {
                                let mut state = state.lock().unwrap();
                                state.increment();
                            })
                            // .style(...) // if we had style API on button directly
                    ),
                ])
        )
}