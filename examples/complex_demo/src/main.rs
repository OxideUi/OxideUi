use strato_core::inspector::{inspector, InspectorConfig};
use strato_core::types::Color;
use strato_platform::{application::Application, window::WindowBuilder};
use strato_widgets::{
    layout::{CrossAxisAlignment, MainAxisAlignment},
    text::{FontWeight, TextAlign},
    Button, ButtonStyle, Column, Container, InspectorOverlay, Row, Text, TextStyle, Widget,
};

fn main() -> anyhow::Result<()> {
    // Create application
    let window_builder = WindowBuilder::new()
        .with_title("Complex Demo")
        .with_size(1024.0, 768.0)
        .resizable(true);

    inspector().configure(InspectorConfig {
        enabled: true,
        ..Default::default()
    });

    let mut app = Application::new("Complex Demo", window_builder);

    // Create UI structure
    // Main container with background
    let main_container = InspectorOverlay::new(
        Container::new()
            .background(Color::rgba(0.95, 0.95, 0.97, 1.0)) // Light gray background
            .padding(20.0)
            .child(
                Column::new()
                    .spacing(20.0)
                    .main_axis_alignment(MainAxisAlignment::Start)
                    .cross_axis_alignment(CrossAxisAlignment::Stretch)
                    .children(vec![
                        // Header Section
                        create_header(),
                        // Content Section (Row with Sidebar and Main Content)
                        Box::new(Row::new().spacing(20.0).children(vec![
                            // Sidebar
                            create_sidebar(),
                            // Main Content Area
                            create_main_content(),
                        ])) as Box<dyn Widget>,
                    ]),
            ),
    );

    app.set_root(Box::new(main_container));

    println!("Starting Complex Demo...");
    app.run()
}

fn create_header() -> Box<dyn Widget> {
    Box::new(
        Container::new()
            .background(Color::rgba(1.0, 1.0, 1.0, 1.0))
            .padding(15.0)
            .border_radius(8.0)
            .child(
                Row::new()
                    .main_axis_alignment(MainAxisAlignment::SpaceBetween)
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(vec![
                        Box::new(
                            Text::new("StratoUI Dashboard")
                                .font_size(24.0)
                                .font_weight(FontWeight::Bold)
                                .color(Color::rgba(0.1, 0.1, 0.2, 1.0)),
                        ) as Box<dyn Widget>,
                        Box::new(Row::new().spacing(10.0).children(vec![
                            Box::new(Button::new("Settings").ghost()) as Box<dyn Widget>,
                            Box::new(Button::new("Profile").primary()) as Box<dyn Widget>,
                        ])) as Box<dyn Widget>,
                    ]),
            ),
    )
}

fn create_sidebar() -> Box<dyn Widget> {
    Box::new(
        Container::new()
            .width(200.0)
            .background(Color::rgba(1.0, 1.0, 1.0, 1.0))
            .padding(10.0)
            .border_radius(8.0)
            .child(
                Column::new()
                    .spacing(5.0)
                    .cross_axis_alignment(CrossAxisAlignment::Stretch)
                    .children(vec![
                        Box::new(
                            Text::new("MENU")
                                .font_size(12.0)
                                .color(Color::rgba(0.5, 0.5, 0.5, 1.0)),
                        ) as Box<dyn Widget>,
                        Box::new(
                            Button::new("Dashboard")
                                .ghost()
                                .on_click(|| println!("Dashboard clicked")),
                        ) as Box<dyn Widget>,
                        Box::new(Button::new("Analytics").ghost()) as Box<dyn Widget>,
                        Box::new(Button::new("Reports").ghost()) as Box<dyn Widget>,
                        Box::new(Button::new("Users").ghost()) as Box<dyn Widget>,
                        Box::new(Container::new().height(20.0)) as Box<dyn Widget>, // Spacer
                        Box::new(
                            Text::new("SYSTEM")
                                .font_size(12.0)
                                .color(Color::rgba(0.5, 0.5, 0.5, 1.0)),
                        ) as Box<dyn Widget>,
                        Box::new(Button::new("Configuration").ghost()) as Box<dyn Widget>,
                        Box::new(Button::new("Logs").ghost()) as Box<dyn Widget>,
                    ]),
            ),
    )
}

fn create_main_content() -> Box<dyn Widget> {
    Box::new(Container::new()
        .background(Color::rgba(1.0, 1.0, 1.0, 1.0))
        .padding(20.0)
        .border_radius(8.0)
        .child(
            Column::new()
                .spacing(15.0)
                .cross_axis_alignment(CrossAxisAlignment::Start)
                .children(vec![
                    Box::new(
                        Text::new("Welcome back, User!")
                            .font_size(20.0)
                            .font_weight(FontWeight::SemiBold)
                    ) as Box<dyn Widget>,
                    Box::new(
                        Text::new("Here is an overview of your system status. This text demonstrates automatic wrapping and layout capabilities of the StratoUI framework. It should flow naturally within the container.")
                            .color(Color::rgba(0.3, 0.3, 0.3, 1.0))
                            .style(TextStyle {
                                line_height: 1.6,
                                ..TextStyle::default()
                            })
                    ) as Box<dyn Widget>,
                    Box::new(Container::new().height(20.0)) as Box<dyn Widget>, // Spacer

                    // Cards Row
                    Box::new(
                        Row::new()
                            .spacing(15.0)
                            .children(vec![
                                create_card("Total Users", "1,234", Color::rgba(0.2, 0.6, 1.0, 0.1)),
                                create_card("Active Sessions", "56", Color::rgba(0.2, 0.8, 0.4, 0.1)),
                                create_card("Server Load", "42%", Color::rgba(1.0, 0.6, 0.2, 0.1)),
                            ])
                    ) as Box<dyn Widget>,

                    Box::new(Container::new().height(20.0)) as Box<dyn Widget>, // Spacer

                    // Typography Showcase
                    Box::new(Text::new("Typography & Spacing").font_size(18.0).font_weight(FontWeight::Medium)) as Box<dyn Widget>,
                    Box::new(
                        Text::new("Wide Spacing Example")
                            .style(TextStyle {
                                letter_spacing: 2.0,
                                ..TextStyle::default()
                            })
                    ) as Box<dyn Widget>,
                    Box::new(
                        Text::new("Tight Spacing Example")
                            .style(TextStyle {
                                letter_spacing: -0.5,
                                ..TextStyle::default()
                            })
                    ) as Box<dyn Widget>,
                ])
        ))
}

fn create_card(title: &str, value: &str, bg_color: Color) -> Box<dyn Widget> {
    Box::new(
        Container::new()
            .width(150.0)
            .height(100.0)
            .background(bg_color)
            .border_radius(8.0)
            .padding(15.0)
            .child(
                Column::new()
                    .main_axis_alignment(MainAxisAlignment::Center)
                    .cross_axis_alignment(CrossAxisAlignment::Start)
                    .children(vec![
                        Box::new(
                            Text::new(title)
                                .font_size(14.0)
                                .color(Color::rgba(0.4, 0.4, 0.4, 1.0)),
                        ) as Box<dyn Widget>,
                        Box::new(
                            Text::new(value)
                                .font_size(24.0)
                                .font_weight(FontWeight::Bold),
                        ) as Box<dyn Widget>,
                    ]),
            ),
    )
}
