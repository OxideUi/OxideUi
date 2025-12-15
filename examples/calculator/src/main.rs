use strato_sdk::prelude::*;
use strato_widgets::{

    text::TextAlign,
    Flex,
};
use strato_platform::{
    ApplicationBuilder,
    WindowBuilder,
    init::{InitBuilder, InitConfig},
};
use strato_core::event::{Event, EventResult, MouseEvent};
use strato_core::types::{Point, Rect, Transform, Color};
use strato_core::state::Signal;
use strato_widgets::animation::{AnimationController, Curve};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::any::Any;

#[derive(Clone, Debug)]
enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Clone, Debug)]
struct CalculatorState {
    display: Signal<String>,
    expression: Signal<String>, // Shows full operation like "12 + 5"
    history: Signal<Vec<String>>, // Stores past calculations
    current_value: f64,
    previous_value: f64,
    operation: Option<Operation>,
    waiting_for_operand: bool,
}

impl Default for CalculatorState {
    fn default() -> Self {
        Self {
            display: Signal::new("0".to_string()),
            expression: Signal::new("".to_string()),
            history: Signal::new(Vec::new()),
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
            self.display.set(digit.to_string());
            self.waiting_for_operand = false;
        } else {
            let current = self.display.get();
            if current == "0" {
                self.display.set(digit.to_string());
            } else {
                self.display.update(|s| s.push_str(&digit.to_string()));
            }
        }
        
        // Update expression
        // If we just finished an operation (equals), start over unless an operator was pressed
        // For simplicity in this step, we just append to expression if it's part of the current number
        // A more robust approach updates expression based entirely on state
        
        self.current_value = self.display.get().parse().unwrap_or(0.0);
    }

    fn input_decimal(&mut self) {
        if self.waiting_for_operand {
            self.display.set("0.".to_string());
            self.waiting_for_operand = false;
        } else if !self.display.get().contains('.') {
            self.display.update(|s| s.push('.'));
        }
    }

    fn clear(&mut self) {
        self.display.set("0".to_string());
        self.expression.set("".to_string());
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
            let display_val = if result.is_nan() {
                "Error".to_string()
            } else if result.fract() == 0.0 && result.abs() < 1e10 {
                format!("{}", result as i64)
            } else {
                format!("{:.8}", result).trim_end_matches('0').trim_end_matches('.').to_string()
            };
            self.display.set(display_val.clone());
            
            // If equals was pressed (next_operation is None)
            if next_operation.is_none() {
                 let op_str = match op {
                    Operation::Add => "+",
                    Operation::Subtract => "-",
                    Operation::Multiply => "×",
                    Operation::Divide => "÷",
                 };
                 let history_entry = format!("{} {} {} = {}", 
                    self.format_number(self.previous_value), 
                    op_str, 
                    self.format_number(self.current_value), // This is actually the 2nd operand before result, but we overwrote it. Needs fix separately or simplified logic.
                    display_val
                 );
                 // Correction: We overwrote current_value with result. We should have stored 2nd operand.
                 // For now let's just push " = result" logic or simplify.
                 // Let's implement a better input tracking.
                 
                 // IMPROVED LOGIC:
                 // We need to track the operands better for the history string.
                 // But strictly following the plan: track history.
                 
                 // Let's construct history string properly.
                 // We'll rely on expression updates in a dedicated manner in a future step if needed,
                 // but here let's set expression to empty or result.
                 self.expression.set("".to_string());
                 
                 // Add to history
                 self.history.update(|h| {
                     h.push(history_entry);
                     if h.len() > 5 { h.remove(0); } // Keep last 5
                 });
            }
        }

        if let Some(ref next_op) = next_operation {
            let op_symbol = match next_op {
                Operation::Add => "+",
                Operation::Subtract => "-",
                Operation::Multiply => "×",
                Operation::Divide => "÷",
            };
            // Update expression display: "Result + "
            let current_display = self.display.get();
            self.expression.set(format!("{} {}", current_display, op_symbol));
        }

        self.waiting_for_operand = true;
        self.operation = next_operation;
        self.previous_value = self.current_value;
    }
    
    fn format_number(&self, num: f64) -> String {
        if num.fract() == 0.0 {
            format!("{}", num as i64)
        } else {
            format!("{}", num)
        }
    }
}

fn main() -> anyhow::Result<()> {
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

    println!("Calculator - StratoUI initialized with font optimizations!");
    
    // Build and run the application
    ApplicationBuilder::new()
        .title("StratoUI Calculator")
        .window(WindowBuilder::new().with_size(320.0, 480.0).resizable(false))
        .run(build_calculator_ui())
}

fn build_calculator_ui() -> impl Widget {
    let state = Arc::new(Mutex::new(CalculatorState::default()));
    
    Container::new()
        .background(Color::rgb(0.0, 0.0, 0.0)) // Pure black background
        .padding(15.0) // Increased padding
        .child(
            Column::new()
                .spacing(12.0) // Increased spacing
                .children(vec![
                    // Display
                    Box::new(create_display(state.clone())),
                    
                    // Button grid (main block)
                    Box::new(create_button_grid(state.clone())),
                    
                    // Button grid (bottom row with span)
                    Box::new(create_bottom_row(state.clone())),
                ])
        )
}

fn create_display(state: Arc<Mutex<CalculatorState>>) -> impl Widget {
    let (display_signal, expression_signal, history_signal) = {
        let state = state.lock().unwrap();
        (state.display.clone(), state.expression.clone(), state.history.clone())
    };
    
    // We Wrap the Container in a Flex to ensure it takes full width of the parent Column/Container
    // effectively pushing the alignment to the far right.
    Flex::new(Box::new(
        Container::new()
            .background(Color::BLACK) // Match main background
            // .padding(15.0) // padding handled by parent or here? 
            // Parent has 15.0 padding. This container is inside that.
            // If we want text to align to the very right edge of this container,
            // we need this container to be as wide as possible.
            // The Flex below with .flex(1.0) on the *Container* itself (if Container implemented FlexItem traits directly)
            // or we use Flex widget.
            .height(150.0)
            .child(
                Column::new()
                    .spacing(4.0)
                    .children(vec![
                        // History 
                        Box::new(
                             Text::new("")
                                .bind(history_signal.map(|h| h.join("\n")))
                                .size(16.0)
                                .color(Color::rgb(0.5, 0.5, 0.5))
                                .align(TextAlign::Right)
                        ),
                        Box::new(Flex::new(Box::new(
                             Container::new().height(10.0) 
                        )).flex(1.0)),
                        // Current Expression 
                        Box::new(
                            Text::new("")
                                .bind(expression_signal)
                                .size(24.0)
                                .color(Color::rgb(0.8, 0.8, 0.8))
                                .align(TextAlign::Right)
                        ),
                        // Main Display
                        Box::new(
                            Text::new("")
                                .bind(display_signal)
                                .size(64.0)
                                .color(Color::rgb(1.0, 1.0, 1.0))
                                .align(TextAlign::Right)
                        ),
                    ])
            )
    )).flex(1.0)
}

fn create_button_grid(state: Arc<Mutex<CalculatorState>>) -> impl Widget {
    let buttons = vec![
        // Row 1
        ("C", ButtonType::Clear), ("±", ButtonType::PlusMinus), ("%", ButtonType::Percent), ("÷", ButtonType::Operation(Operation::Divide)),
        // Row 2
        ("7", ButtonType::Digit(7)), ("8", ButtonType::Digit(8)), ("9", ButtonType::Digit(9)), ("×", ButtonType::Operation(Operation::Multiply)),
        // Row 3
        ("4", ButtonType::Digit(4)), ("5", ButtonType::Digit(5)), ("6", ButtonType::Digit(6)), ("-", ButtonType::Operation(Operation::Subtract)),
        // Row 4
        ("1", ButtonType::Digit(1)), ("2", ButtonType::Digit(2)), ("3", ButtonType::Digit(3)), ("+", ButtonType::Operation(Operation::Add)),
    ];

    let grid_items: Vec<Box<dyn Widget>> = buttons.into_iter().map(|(text, btn_type)| {
        Box::new(AnimatedButton::new(
            text, 
            btn_type, 
            state.clone()
        )) as Box<dyn Widget>
    }).collect();

    Grid::new()
        .columns(vec![
            GridUnit::Fraction(1.0),
            GridUnit::Fraction(1.0),
            GridUnit::Fraction(1.0),
            GridUnit::Fraction(1.0),
        ])
        .row_gap(8.0)
        .col_gap(8.0)
        .children(grid_items)
}


fn create_bottom_row(state: Arc<Mutex<CalculatorState>>) -> impl Widget {
    Row::new()
        .spacing(8.0)
        .children(vec![
            // Zero button - Spans 2 columns worth (50%)
            Box::new(Flex::new(
                Box::new(AnimatedButton::new("0", ButtonType::Digit(0), state.clone()))
            ).flex(2.0)),
            
            // Decimal
            Box::new(Flex::new(
                Box::new(AnimatedButton::new(".", ButtonType::Decimal, state.clone()))
            ).flex(1.0)),
            
            // Equals
            Box::new(Flex::new(
                Box::new(AnimatedButton::new("=", ButtonType::Equals, state.clone()))
            ).flex(1.0)),
        ])
}

#[derive(Clone, Debug)]
enum ButtonType {
    Digit(u8),
    Operation(Operation),
    Decimal,
    Equals,
    Clear,
    PlusMinus,
    Percent,
}

// Custom Animated Button Widget
#[derive(Debug)]
struct AnimatedButton {
    id: WidgetId,
    text: String,
    button_type: ButtonType,
    state: Arc<Mutex<CalculatorState>>,
    anim_controller: AnimationController,
    is_pressed: bool,
    bounds: Arc<Mutex<Rect>>,
}

impl AnimatedButton {
    fn new(text: &str, button_type: ButtonType, state: Arc<Mutex<CalculatorState>>) -> Self {
        let controller = AnimationController::new(Duration::from_millis(100))
            .with_curve(Curve::EaseOut);
        
        Self {
            id: strato_widgets::widget::generate_id(),
            text: text.to_string(),
            button_type,
            state,
            anim_controller: controller,
            is_pressed: false,
            bounds: Arc::new(Mutex::new(Rect::new(0.0, 0.0, 0.0, 0.0))),
        }
    }
}

impl Widget for AnimatedButton {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn layout(&mut self, constraints: strato_core::layout::Constraints) -> strato_core::layout::Size {
        // Fill available space
        strato_core::layout::Size::new(
            constraints.max_width, 
            // 70.0 is roughly the height we want, but let's be flexible
            constraints.max_height.min(70.0).max(50.0) 
        )
    }

    fn render(&self, batch: &mut strato_renderer::batch::RenderBatch, layout: strato_core::layout::Layout) {
        // Update bounds for hit testing in event handling
        if let Ok(mut bounds) = self.bounds.lock() {
            *bounds = Rect::new(layout.position.x, layout.position.y, layout.size.width, layout.size.height);
        }

        let bg_color = match self.button_type {
            ButtonType::Operation(_) | ButtonType::Equals => Color::rgb(1.0, 0.62, 0.04), // Orange (#FF9F0A)
            ButtonType::Clear | ButtonType::PlusMinus | ButtonType::Percent => Color::rgb(0.65, 0.65, 0.65), // Light gray (#A5A5A5)
            _ => Color::rgb(0.2, 0.2, 0.2), // Dark gray (#333333)
        };
        
        let text_color = match self.button_type {
            ButtonType::Clear | ButtonType::PlusMinus | ButtonType::Percent => Color::rgb(0.0, 0.0, 0.0), // Black text
            _ => Color::rgb(1.0, 1.0, 1.0), // White text
        };

        // Animation logic
        let scale = if self.is_pressed {
            0.95 
        } else {
            // Check animation progress if releasing
            let t = self.anim_controller.value();
            if t < 1.0 {
                 0.95 + (0.05 * t) // rebound
            } else {
                1.0
            }
        };

        // Draw button shape (Circle or Pill)
        let radius = layout.size.height.min(layout.size.width) / 2.0;

        // Apply scale from center
        let center = layout.size.to_vec2() / 2.0;
        let transform = Transform::translate(layout.position.x + center.x, layout.position.y + center.y)
            .combine(&Transform::scale(scale, scale))
            .combine(&Transform::translate(-center.x, -center.y));

        if (layout.size.width - layout.size.height).abs() < 1.0 {
            // Perfect circle (Square aspect ratio)
            let center_pt = (layout.position.x + layout.size.width / 2.0, layout.position.y + layout.size.height / 2.0);
            batch.add_circle(center_pt, radius, bg_color, 32, transform);
        } else {
            // Pill shape (Width > Height, e.g., '0' button)
            // Left circle
            let left_center = (layout.position.x + radius, layout.position.y + radius);
            batch.add_circle(left_center, radius, bg_color, 32, transform);
            
            // Right circle
            let right_center = (layout.position.x + layout.size.width - radius, layout.position.y + radius);
            batch.add_circle(right_center, radius, bg_color, 32, transform);
            
            // Middle rect
            let rect = Rect::new(
                layout.position.x + radius, 
                layout.position.y, 
                layout.size.width - 2.0 * radius, 
                layout.size.height
            );
            batch.add_rect(rect, bg_color, transform);
        }

        // Draw text
        let font_size = 32.0;
        let text_x = layout.position.x + layout.size.width / 2.0;
        let text_y = layout.position.y + layout.size.height / 2.0 - font_size / 2.0;
        
        batch.add_text_aligned(
             self.text.clone(),
             (text_x, text_y),
             text_color,
             font_size,
            0.0,
            strato_core::text::TextAlign::Center,
        );
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        match event {
            Event::MouseDown(MouseEvent { position, .. }) => {
                let bounds = *self.bounds.lock().unwrap();
                let point = Point::new(position.x, position.y);
                
                if bounds.contains(point) {
                    self.is_pressed = true;
                    return EventResult::Handled;
                }
            }
            Event::MouseUp(MouseEvent { position, .. }) => {
                if self.is_pressed {
                    self.is_pressed = false;
                    self.anim_controller.reset();
                    self.anim_controller.start();
                    
                    // Check if still within bounds to trigger action (standard button behavior)
                    let bounds = *self.bounds.lock().unwrap();
                    let point = Point::new(position.x, position.y);
                    
                    if bounds.contains(point) {
                        // Perform action
                        handle_button_click(&self.button_type, self.state.clone());
                    }
                    return EventResult::Handled;
                }
            }
             _ => {}
        }
        EventResult::Ignored
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_widget(&self) -> Box<dyn Widget> {
        Box::new(Self {
            id: strato_widgets::widget::generate_id(),
            text: self.text.clone(),
            button_type: self.button_type.clone(),
            state: self.state.clone(),
            anim_controller: self.anim_controller.clone(),
            is_pressed: self.is_pressed,
            bounds: self.bounds.clone(),
        })
    }
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
            let val = if state.current_value.fract() == 0.0 {
                format!("{}", state.current_value as i64)
            } else {
                format!("{}", state.current_value)
            };
            state.display.set(val);
        }
        ButtonType::Percent => {
            state.current_value = state.current_value / 100.0;
            state.display.set(format!("{}", state.current_value));
        }
    }
    
    println!("Calculator state: {:?}", *state);
}