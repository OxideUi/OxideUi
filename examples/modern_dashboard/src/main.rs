use strato_core::{
    types::Color,
};
use strato_platform::{application::ApplicationBuilder, window::WindowBuilder};
use strato_widgets::{
    prelude::*,
    container::Container,
    text::Text,
    image::Image,
    top_bar::TopBar,
    layout::{Column, Row, Flex}, 
};

fn main() {
    // We don't need registry for direct widget construction
    let root_widget = build_ui();

    ApplicationBuilder::new()
        .window(
            WindowBuilder::new()
                .with_title("Modern Dashboard")
                .with_size(1200.0, 800.0)
                .resizable(true)
                .transparent(true) // Enable glassmorphism support
                .decorations(true) // Restore native window controls
        )
        .run(root_widget);
}


// --- Theme Colors ---
// Catppuccin-inspired Logic format (0.0-1.0)
// --- Theme Colors (Refined) ---
fn col_bg() -> Color { Color::rgba(0.13, 0.13, 0.18, 0.90) } // Semi-transparent background
fn col_sidebar() -> Color { Color::rgba(0.18, 0.18, 0.25, 0.95) } 
fn col_card_bg() -> Color { Color::rgba(0.20, 0.20, 0.28, 0.8) } // Glassy cards
fn col_text() -> Color { Color::rgba(0.95, 0.95, 0.95, 1.0) }
fn col_text_dim() -> Color { Color::rgba(0.70, 0.70, 0.80, 1.0) }
fn col_subtext() -> Color { Color::rgba(0.60, 0.60, 0.70, 1.0) } 
fn col_accent() -> Color { Color::rgba(0.12, 0.45, 0.98, 1.0) } 
fn col_header() -> Color { Color::rgba(0.50, 0.50, 0.60, 1.0) }

fn build_ui() -> Container {
    Container::new()
        .background(Color::TRANSPARENT)
        .child(Row::new()
            .children(vec![
                // Sidebar (Fixed Width, Full Height)
                Box::new(Container::new()
                    .width(260.0)
                    .height(800.0)
                    .background(col_sidebar())
                    .padding(0.0)
                    .border_radius(12.0)
                    .margin(10.0)
                    .child(Column::new()
                        .spacing(2.0)
                        .children(vec![
                            // Window Controls Spacer
                            Box::new(Container::new().height(20.0).child(Text::new(""))),
                            
                            // User Profile
                            Box::new(Container::new()
                                .padding(16.0)
                                .margin(0.0)
                                .child(Row::new()
                                    .spacing(12.0)
                                    .children(vec![
                                        // Avatar
                                        Box::new(Container::new()
                                            .width(32.0).height(32.0).border_radius(4.0).background(Color::WHITE)
                                            .child(Image::from_url("https://avatars.githubusercontent.com/u/109359355?v=4")
                                                .border_radius(4.0)
                                            )),
                                        Box::new(Text::new("SeregonWar").color(col_text()).size(14.0))
                                    ])
                                )
                            ),
                            
                            // TIMELINES
                            section_header("TIMELINES"),
                            sidebar_item("Classic Timeline", "📅", false),
                            sidebar_item("Local", "👥", false),
                            sidebar_item("Federated", "🌐", false),

                            // ACCOUNT
                            Box::new(Container::new().height(20.0).child(Text::new(""))), 
                            section_header("ACCOUNT"),
                            sidebar_item("Your Posts", "📝", false),
                            sidebar_item("Followers", "👥", false),
                            sidebar_item("Following", "👤", true), 
                            sidebar_item("Bookmarks", "🔖", false),
                            sidebar_item("Favorites", "⭐️", false),
                        
                            // Spacer (Pushes content down)
                            Box::new(Flex::new(
                                Box::new(Container::new().height(1.0).child(Text::new("")))
                            ).flex(1.0)),

                            // Theme Switcher (At bottom)
                            Box::new(Container::new()
                                .background(Color::rgba(1.0, 1.0, 1.0, 0.05))
                                .padding(10.0)
                                .border_radius(6.0)
                                .width(240.0)
                                .child(Row::new()
                                    .spacing(12.0)
                                    .children(vec![
                                        Box::new(Text::new("sun").size(14.0)), 
                                        Box::new(Text::new("Toggle Theme").color(col_text_dim()).size(14.0))
                                    ])
                                )
                                .on_click(|| {
                                    println!("Theme toggle clicked - Dynamic switching pending");
                                })
                            )
                        ])
                    )
                ),
                
                // Main Content Area
                Box::new(Container::new()
                    .padding(0.0)
                    .width(900.0)
                    .child(Column::new()
                        .spacing(0.0)
                        .children(vec![
                            // TopBar
                            Box::new(TopBar::new("Following".to_string())
                                .with_background(col_bg())
                                .height(60.0)),

                            // Feed
                            Box::new(Container::new()
                                .padding(24.0)
                                .background(col_bg()) // Main bg
                                .border_radius(12.0)
                                .margin(10.0)
                                .child(Column::new()
                                    .spacing(16.0)
                                    .children(vec![
                                        card_item(
                                            "NDR", 
                                            "@NDR", 
                                            "Moin! Hier gibt's pro Tag 3-5 Nachrichten aus Hamburg...", 
                                            "146 Posts  7 Following  5k Followers",
                                            "https://avatars.githubusercontent.com/u/1?v=4"
                                        ),
                                        card_item(
                                            "tagesschau", 
                                            "@tagesschau", 
                                            "Hier trötet die tagesschau Nachrichten von https://www.tagesschau.de/", 
                                            "683 Posts  4 Following  20k Followers",
                                            "https://avatars.githubusercontent.com/u/2?v=4"
                                        ),
                                        card_item(
                                            "Simon Willison", 
                                            "@simon", 
                                            "Open source developer building tools to help journalists...", 
                                            "3k Posts  1k Following  17k Followers",
                                            "https://avatars.githubusercontent.com/u/3?v=4"
                                        ),
                                    ])
                                )
                            )
                        ])
                    )
                )
            ])
        )
}

// --- Components ---

fn section_header(title: &str) -> Box<dyn Widget> {
    Box::new(Container::new()
        .padding(16.0) // Left padding alignment
        .margin(0.0)
        .child(Text::new(title).color(col_header()).size(11.0))
    )
}

fn sidebar_item(label: &str, icon: &str, active: bool) -> Box<dyn Widget> {
    let bg_color = if active { col_accent() } else { Color::TRANSPARENT };
    let text_color = if active { Color::WHITE } else { col_subtext() };
    let label_clone = label.to_string();
    
    Box::new(Container::new()
        .background(bg_color)
        .margin(0.0)
        .padding(10.0)
        .border_radius(6.0) // Rounded active item
        .width(240.0) // Slight inset from full width
        .child(Row::new()
            .spacing(12.0)
            .children(vec![
                Box::new(Text::new(icon).size(14.0)), // Smaller icons
                Box::new(Text::new(label).color(text_color).size(14.0))
            ])
        )
        .on_click(move || {
            println!("Clicked sidebar item: {}", label_clone);
        })
    )
}

fn card_item(name: &str, handle: &str, content: &str, stats: &str, avatar_url: &str) -> Box<dyn Widget> {
    Box::new(Container::new()
        .background(col_card_bg())
        .padding(16.0)
        .border_radius(10.0) // Smooth card rounding
        .width(600.0) // Fixed card width for specific look
        .child(Column::new()
            .spacing(12.0)
            .children(vec![
                // Header Row
                Box::new(Row::new()
                    .spacing(12.0)
                    .children(vec![
                        // Avatar
                        Box::new(Container::new()
                            .width(44.0).height(44.0).border_radius(4.0)
                            .child(Image::from_url(avatar_url).border_radius(4.0))
                        ),
                        // Name & Handle
                        Box::new(Column::new()
                            .spacing(2.0)
                            .children(vec![
                                Box::new(Text::new(name).color(col_text()).size(15.0)),
                                Box::new(Text::new(handle).color(col_subtext()).size(13.0))
                            ])
                        ),
                        Box::new(Container::new().width(180.0).child(Text::new(""))), // Flex Spacer
                        // Button
                        Box::new(Container::new()
                            .background(col_accent())
                            .border_radius(14.0) // Pill shape
                            .padding(6.0)
                            .width(60.0)
                            .child(Container::new()
                                 // Centering hack (via padding)
                                 .padding(0.0).margin(0.0)
                                 .child(Text::new("Open").color(Color::WHITE).size(12.0))
                            )
                            .on_click(|| {
                                println!("Clicked Open button");
                            })
                        )
                    ])
                ),
                // Content Body
                Box::new(Container::new()
                     .padding(0.0)
                     .child(Text::new(content).color(col_subtext()).size(14.0))
                ),
                // Footer Stats
                Box::new(Container::new()
                    .padding(0.0)
                    .child(Text::new(stats).color(col_header()).size(12.0))
                )
            ])
        )
    )
}
