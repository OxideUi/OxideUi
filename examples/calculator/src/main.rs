//! Calculator example for OxideUI
//! 
//! This example demonstrates a functional calculator with a grid layout
//! to test various rendering capabilities and identify potential issues.

use oxide_ui::prelude::*;
use oxide_widgets::{ButtonStyle, text::TextAlign};
use oxide_core::theme::Color as CoreColor;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Clone, Debug)]
struct CalculatorState {
    display: String,
    current_value: f64,
    previous_value: f64,
    operation: Option<Operation>,
    waiting_for_operand: bool,
}

impl Default for CalculatorState {
    fn default() -> Self {
        Self {
            display: "0".to_string(),
            current_value: 0.0,
            previous_value: 0.0,
            operation: None,
            waiting_for_operand: false,
        }
    }
}

impl CalculatorState {
    fn input_digit(&mut self, digit: u8) {
        if self.waiting_for_operand {
            self.display = digit.to_string();
            self.waiting_for_operand = false;
        } else {
            if self.display == "0" {
                self.display = digit.to_string();
            } else {
                self.display.push_str(&digit.to_string());
            }
        }
        self.current_value = self.display.parse().unwrap_or(0.0);
    }

    fn input_decimal(&mut self) {
        if self.waiting_for_operand {
            self.display = "0.".to_string();
            self.waiting_for_operand = false;
        } else if !self.display.contains('.') {
            self.display.push('.');
        }
    }

    fn clear(&mut self) {
        self.display = "0".to_string();
        self.current_value = 0.0;
        self.previous_value = 0.0;
        self.operation = None;
        self.waiting_for_operand = false;
    }

    fn perform_operation(&mut self, next_operation: Option<Operation>) {
        if let Some(op) = &self.operation {
            let result = match op {
                Operation::Add => self.previous_value + self.current_value,
                Operation::Subtract => self.previous_value - self.current_value,
                Operation::Multiply => self.previous_value * self.current_value,
                Operation::Divide => {
                    if self.current_value != 0.0 {
                        self.previous_value / self.current_value
                    } else {
                        f64::NAN
                    }
                }
            };

            self.current_value = result;
            self.display = if result.is_nan() {
                "Error".to_string()
            } else if result.fract() == 0.0 && result.abs() < 1e10 {
                format!("{}", result as i64)
            } else {
                format!("{:.8}", result).trim_end_matches('0').trim_end_matches('.').to_string()
            };
        }

        self.waiting_for_operand = true;
        self.operation = next_operation;
        self.previous_value = self.current_value;
    }
}

fn main() -> Result<()> {
    // Use the new initialization system with optimized font loading
    let config = InitConfig {
        enable_logging: true,
        skip_problematic_fonts: true,
        max_font_faces: Some(25), // Even more restrictive for calculator
        ..Default::default()
    };

    InitBuilder::new()
        .with_config(config)
        .init_all()?;

    println!("Calculator - OxideUI initialized with font optimizations!");
    
    // Build and run the application
    ApplicationBuilder::new()
        .title("OxideUI Calculator")
        .window(WindowBuilder::new().with_size(320.0, 480.0).resizable(false))
        .run(build_calculator_ui());

    println!("Calculator UI created successfully!");
    println!("Optimizations applied:");
    println!("  • Reduced font faces to 25 for calculator app");
    println!("  • Avoided problematic system fonts");
    println!("  • Single text renderer instance shared across components");
}

fn build_calculator_ui() -> impl Widget {
    let state = Arc::new(Mutex::new(CalculatorState::default()));
    
    Container::new()
        .background(Color::rgb(0.1, 0.1, 0.1))
        .padding(10.0)
        .child(
            Column::new()
                .spacing(10.0)
                .children(vec![
                    // Display
                    Box::new(create_display(state.clone())),
                    
                    // Button grid
                    Box::new(create_button_grid(state.clone())),
                ])
        )
}

fn create_display(state: Arc<Mutex<CalculatorState>>) -> impl Widget {
    let display_text = {
        let state = state.lock().unwrap();
        state.display.clone()
    };
    
    Container::new()
        .background(Color::rgb(0.05, 0.05, 0.05))
        .padding(15.0)
        .height(80.0)
        .child(
            Text::new(&display_text)
                .size(32.0)
                .color(Color::rgb(1.0, 1.0, 1.0))
                .align(TextAlign::Right)
        )
}

fn create_button_grid(state: Arc<Mutex<CalculatorState>>) -> impl Widget {
    Column::new()
        .spacing(8.0)
        .children(vec![
            // Row 1: Clear, +/-, %, ÷
            Box::new(create_button_row(vec![
                ("C", ButtonType::Clear, state.clone()),
                ("±", ButtonType::PlusMinus, state.clone()),
                ("%", ButtonType::Percent, state.clone()),
                ("÷", ButtonType::Operation(Operation::Divide), state.clone()),
            ])),
            
            // Row 2: 7, 8, 9, ×
            Box::new(create_button_row(vec![
                ("7", ButtonType::Digit(7), state.clone()),
                ("8", ButtonType::Digit(8), state.clone()),
                ("9", ButtonType::Digit(9), state.clone()),
                ("×", ButtonType::Operation(Operation::Multiply), state.clone()),
            ])),
            
            // Row 3: 4, 5, 6, -
            Box::new(create_button_row(vec![
                ("4", ButtonType::Digit(4), state.clone()),
                ("5", ButtonType::Digit(5), state.clone()),
                ("6", ButtonType::Digit(6), state.clone()),
                ("-", ButtonType::Operation(Operation::Subtract), state.clone()),
            ])),
            
            // Row 4: 1, 2, 3, +
            Box::new(create_button_row(vec![
                ("1", ButtonType::Digit(1), state.clone()),
                ("2", ButtonType::Digit(2), state.clone()),
                ("3", ButtonType::Digit(3), state.clone()),
                ("+", ButtonType::Operation(Operation::Add), state.clone()),
            ])),
            
            // Row 5: 0 (wide), ., =
            Box::new(create_bottom_row(state.clone())),
        ])
}

fn create_button_row(buttons: Vec<(&str, ButtonType, Arc<Mutex<CalculatorState>>)>) -> impl Widget {
    Row::new()
        .spacing(8.0)
        .children(
            buttons.into_iter()
                .map(|(text, button_type, state)| {
                    Box::new(create_calculator_button(text, button_type, state)) as Box<dyn Widget>
                })
                .collect()
        )
}

fn create_bottom_row(state: Arc<Mutex<CalculatorState>>) -> impl Widget {
    Row::new()
        .spacing(8.0)
        .children(vec![
            // Wide 0 button (takes 2 columns)
            Box::new(
                Container::new()
                    .width(148.0) // (70 * 2) + 8 for spacing
                    .height(70.0)
                    .background(Color::rgb(0.2, 0.2, 0.2))
                    .child(
                        Button::new("0")
                            .on_click({
                                let state = state.clone();
                                move || {
                                    let mut state = state.lock().unwrap();
                                    state.input_digit(0);
                                }
                            })
                    )
            ),
            
            // Decimal point
            Box::new(create_calculator_button(".", ButtonType::Decimal, state.clone())),
            
            // Equals
            Box::new(create_calculator_button("=", ButtonType::Equals, state.clone())),
        ])
}

#[derive(Clone)]
enum ButtonType {
    Digit(u8),
    Operation(Operation),
    Decimal,
    Equals,
    Clear,
    PlusMinus,
    Percent,
}

fn create_calculator_button(
    text: &str, 
    button_type: ButtonType, 
    state: Arc<Mutex<CalculatorState>>
) -> impl Widget {
    let bg_color = match button_type {
        ButtonType::Operation(_) | ButtonType::Equals => Color::rgb(1.0, 0.6, 0.0), // Orange
        ButtonType::Clear | ButtonType::PlusMinus | ButtonType::Percent => Color::rgb(0.6, 0.6, 0.6), // Light gray
        _ => Color::rgb(0.2, 0.2, 0.2), // Dark gray
    };
    
    let text_color = match button_type {
        ButtonType::Clear | ButtonType::PlusMinus | ButtonType::Percent => Color::rgb(0.0, 0.0, 0.0), // Black text
        _ => Color::rgb(1.0, 1.0, 1.0), // White text
    };
    
    Container::new()
        .width(70.0)
        .height(70.0)
        .background(bg_color)
        .child(
            Button::new(text)
                .style(ButtonStyle {
                    background_color: CoreColor::rgba(bg_color.r, bg_color.g, bg_color.b, bg_color.a),
                    text_color: CoreColor::rgba(text_color.r, text_color.g, text_color.b, text_color.a),
                    font_size: 24.0,
                    ..Default::default()
                })
                .on_click({
                    let state = state.clone();
                    let button_type = button_type.clone();
                    move || {
                        handle_button_click(&button_type, state.clone());
                    }
                })
        )
}

fn handle_button_click(button_type: &ButtonType, state: Arc<Mutex<CalculatorState>>) {
    let mut state = state.lock().unwrap();
    
    match button_type {
        ButtonType::Digit(digit) => {
            state.input_digit(*digit);
        }
        ButtonType::Decimal => {
            state.input_decimal();
        }
        ButtonType::Operation(op) => {
            state.perform_operation(Some(op.clone()));
        }
        ButtonType::Equals => {
            state.perform_operation(None);
        }
        ButtonType::Clear => {
            state.clear();
        }
        ButtonType::PlusMinus => {
            state.current_value = -state.current_value;
            state.display = if state.current_value.fract() == 0.0 {
                format!("{}", state.current_value as i64)
            } else {
                format!("{}", state.current_value)
            };
        }
        ButtonType::Percent => {
            state.current_value = state.current_value / 100.0;
            state.display = format!("{}", state.current_value);
        }
    }
    
    println!("Calculator state: {:?}", *state);
}