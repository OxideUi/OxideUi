use strato_platform::{
    application::Application,
    window::WindowBuilder,
};
use strato_widgets::{
    Button, ButtonStyle,
    Column,
    Container,
    Row,
    Flex,
    Text,
    text::FontWeight,
    input::{TextInput, InputType},
    Dropdown,
    Widget,
    layout::{MainAxisAlignment, CrossAxisAlignment},
};
use strato_core::types::Color;
use tracing::{info, warn};
use tracing_subscriber::prelude::*;

// Theme Colors
fn theme_bg() -> Color { Color::rgba(0.09, 0.09, 0.11, 1.0) } // #17171C
fn theme_surface() -> Color { Color::rgba(0.13, 0.13, 0.16, 1.0) } // #212129
fn theme_surface_hover() -> Color { Color::rgba(0.16, 0.16, 0.20, 1.0) } // #292933
fn theme_accent() -> Color { Color::rgba(0.39, 0.35, 0.88, 1.0) } // #6459E1
fn theme_text_primary() -> Color { Color::rgba(0.95, 0.95, 0.97, 1.0) }
fn theme_text_secondary() -> Color { Color::rgba(0.60, 0.60, 0.65, 1.0) }
const BORDER_RADIUS: f32 = 12.0;

fn main() -> anyhow::Result<()> {
    // Setup logging to file
    let file_appender = tracing_appender::rolling::daily("logs", "comprehensive_test.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false)
        )
        .with(tracing_subscriber::fmt::layer()
            .with_writer(std::io::stdout)
        )
        .init();

    info!("Starting Comprehensive Test Example - Revolutionized UI");

    // Create application
    let window_builder = WindowBuilder::new()
        .with_title("StratoUI Dashboard")
        .with_size(1280.0, 900.0)
        .resizable(true);
        
    let mut app = Application::new("StratoUI Dashboard", window_builder);

    // Create UI structure
    let main_container = Container::new()
        .background(theme_bg())
        .padding(30.0)
        .child(
            Column::new()
                .spacing(30.0)
                .main_axis_alignment(MainAxisAlignment::Start)
                .cross_axis_alignment(CrossAxisAlignment::Stretch)
                .children(vec![
                    create_header(),
                    
                    // Main Content Grid (simulated with Row/Column)
                    Box::new(Row::new()
                        .spacing(30.0)
                        .cross_axis_alignment(CrossAxisAlignment::Start)
                        .children(vec![
                            // Left Column (Interactive)
                            Box::new(Flex::new(
                                create_interactive_section()
                            )) as Box<dyn Widget>,
                            
                            // Right Column (Layouts/Cards)
                            Box::new(Flex::new(
                                create_layout_section()
                            )) as Box<dyn Widget>,
                        ])
                    ) as Box<dyn Widget>,
                ])
        );

    app.set_root(Box::new(main_container));

    info!("Application initialized, entering event loop");
    app.run()
}

fn create_header() -> Box<dyn Widget> {
    Box::new(Container::new()
        .background(theme_surface())
        .padding(25.0)
        .border_radius(BORDER_RADIUS)
        .child(
            Row::new()
                .main_axis_alignment(MainAxisAlignment::SpaceBetween)
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(vec![
                    Box::new(
                        Column::new()
                            .spacing(5.0)
                            .children(vec![
                                Box::new(
                                    Text::new("StratoUI Dashboard")
                                        .font_size(28.0)
                                        .font_weight(FontWeight::Bold)
                                        .color(theme_text_primary())
                                ) as Box<dyn Widget>,
                                Box::new(
                                    Text::new("Next-gen Rust Native UI Framework")
                                        .font_size(14.0)
                                        .color(theme_text_secondary())
                                ) as Box<dyn Widget>,
                            ])
                    ) as Box<dyn Widget>,
                    
                    Box::new(
                        Row::new()
                            .spacing(15.0)
                            .children(vec![
                                Box::new(
                                    Button::new("Documentation")
                                        .outline()
                                        .on_click(|| info!("Docs clicked"))
                                ) as Box<dyn Widget>,
                                Box::new(
                                    Button::new("New Project")
                                        .primary()
                                        .on_click(|| info!("New Project clicked"))
                                ) as Box<dyn Widget>,
                            ])
                    ) as Box<dyn Widget>,
                ])
        ))
}

fn create_interactive_section() -> Box<dyn Widget> {
    Box::new(Column::new()
        .spacing(20.0)
        .children(vec![
            create_section_title("Controls & Inputs"),
            
            Container::new()
                .background(theme_surface())
                .padding(20.0)
                .border_radius(BORDER_RADIUS)
                .child(
                    Column::new()
                        .spacing(25.0)
                        .cross_axis_alignment(CrossAxisAlignment::Stretch)
                        .children(vec![
                            // Buttons Group
                            Box::new(Column::new()
                                .spacing(10.0)
                                .children(vec![
                                    Box::new(Text::new("Buttons").font_size(14.0).color(theme_text_secondary())) as Box<dyn Widget>,
                                    Box::new(Row::new()
                                        .spacing(10.0)
                                        .children(vec![
                                            Box::new(Flex::new(Box::new(Button::new("Primary").primary())).flex(1.0)) as Box<dyn Widget>,
                                            Box::new(Flex::new(Box::new(Button::new("Secondary").secondary())).flex(1.0)) as Box<dyn Widget>,
                                            Box::new(Flex::new(Box::new(Button::new("Danger").danger())).flex(1.0)) as Box<dyn Widget>,
                                        ])
                                    ) as Box<dyn Widget>,
                                ])
                            ) as Box<dyn Widget>,

                            // Dropdown Group
                            Box::new(Column::new()
                                .spacing(10.0)
                                .children(vec![
                                    Box::new(Text::new("Selection").font_size(14.0).color(theme_text_secondary())) as Box<dyn Widget>,
                                    Box::new(
                                        Dropdown::new()
                                            .add_value("Development".to_string())
                                            .add_value("Staging".to_string())
                                            .add_value("Production".to_string())
                                            .placeholder("Select Environment".to_string())
                                    ) as Box<dyn Widget>,
                                ])
                            ) as Box<dyn Widget>,

                            // Inputs Group
                            Box::new(Column::new()
                                .spacing(15.0)
                                .children(vec![
                                    Box::new(Text::new("Authentication").font_size(14.0).color(theme_text_secondary())) as Box<dyn Widget>,
                                    Box::new(
                                        TextInput::new()
                                            .placeholder("Username or Email")
                                    ) as Box<dyn Widget>,
                                    Box::new(
                                        TextInput::new()
                                            .placeholder("Password")
                                            .input_type(InputType::Password)
                                    ) as Box<dyn Widget>,
                                ])
                            ) as Box<dyn Widget>,
                        ])
                )
                .into_boxed()
        ])
    )
}

fn create_layout_section() -> Box<dyn Widget> {
    Box::new(Column::new()
        .spacing(20.0)
        .children(vec![
            create_section_title("Project Status"),
            
            // Stats Row
            Box::new(Row::new()
                .spacing(20.0)
                .children(vec![
                    create_stat_card("Active Users", "12.5k", "+15%", theme_accent()),
                    create_stat_card("Server Load", "42%", "-5%", Color::rgba(0.2, 0.8, 0.4, 1.0)),
                    create_stat_card("Errors", "0.01%", "-2%", Color::rgba(0.9, 0.3, 0.3, 1.0)),
                ])
            ) as Box<dyn Widget>,
            
            // Detailed Cards
            Box::new(Container::new()
                .background(theme_surface())
                .padding(20.0)
                .border_radius(BORDER_RADIUS)
                .child(
                    Column::new()
                        .spacing(15.0)
                        .children(vec![
                            Box::new(Text::new("Recent Activity").font_size(16.0).font_weight(FontWeight::SemiBold).color(theme_text_primary())) as Box<dyn Widget>,
                            create_activity_item("Deployment #1024", "Successful", "2 mins ago"),
                            create_activity_item("Database Backup", "Processing", "15 mins ago"),
                            create_activity_item("User Registration", "New User", "1 hour ago"),
                        ])
                )
            ) as Box<dyn Widget>,
        ])
    )
}

fn create_section_title(title: &str) -> Box<dyn Widget> {
    Box::new(Text::new(title)
        .font_size(18.0)
        .font_weight(FontWeight::SemiBold)
        .color(theme_text_primary())
    )
}

fn create_stat_card(label: &str, value: &str, trend: &str, trend_color: Color) -> Box<dyn Widget> {
    Box::new(Flex::new(Box::new(Container::new()
        .background(theme_surface())
        .padding(20.0)
        .border_radius(BORDER_RADIUS)
        .child(
            Column::new()
                .spacing(10.0)
                .children(vec![
                    Box::new(Text::new(label).font_size(14.0).color(theme_text_secondary())) as Box<dyn Widget>,
                    Box::new(Text::new(value).font_size(24.0).font_weight(FontWeight::Bold).color(theme_text_primary())) as Box<dyn Widget>,
                    Box::new(Text::new(trend).font_size(12.0).color(trend_color)) as Box<dyn Widget>,
                ])
        ))))
}

fn create_activity_item(title: &str, status: &str, time: &str) -> Box<dyn Widget> {
    Box::new(Container::new()
        .background(theme_surface_hover())
        .padding(12.0)
        .border_radius(8.0)
        .child(
            Row::new()
                .main_axis_alignment(MainAxisAlignment::SpaceBetween)
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .children(vec![
                    Box::new(
                        Column::new()
                            .spacing(4.0)
                            .children(vec![
                                Box::new(Text::new(title).font_size(14.0).color(theme_text_primary())) as Box<dyn Widget>,
                                Box::new(Text::new(status).font_size(12.0).color(theme_accent())) as Box<dyn Widget>,
                            ])
                    ) as Box<dyn Widget>,
                    Box::new(Text::new(time).font_size(12.0).color(theme_text_secondary())) as Box<dyn Widget>,
                ])
        ))
}

// Extension trait helper to box widgets easily
trait WidgetExt {
    fn into_boxed(self) -> Box<dyn Widget>;
}

impl<T: Widget + 'static> WidgetExt for T {
    fn into_boxed(self) -> Box<dyn Widget> {
        Box::new(self)
    }
}
