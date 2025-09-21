//! Hello World example for OxideUI with improved logging

// Import standard del framework
use oxide_ui::prelude::*;
use oxide_ui::{InitBuilder, InitConfig};
use oxide_ui::oxide_core::Result;
use oxide_core::{
    config::{OxideConfig, LoggingConfig},
    logging::{LogLevel, LogCategory},
    oxide_info, oxide_debug, oxide_error, oxide_text_debug,
};
use std::collections::HashMap;

fn main() -> Result<()> {
    // Configure logging to reduce noise and improve debugging
    setup_logging()?;

    // Use the new granular initialization system with font optimization
    let config = InitConfig {
        enable_logging: true,
        skip_problematic_fonts: true, // Avoid mstmc.ttf and other problematic fonts
        max_font_faces: Some(30),     // Limit font loading for better performance
        ..Default::default()
    };

    oxide_info!(LogCategory::Core, "OxideUI Hello World starting with optimized configuration");

    let mut builder = InitBuilder::new()
        .with_config(config.clone());
    
    match builder.init_all() {
        Ok(_) => {
            oxide_info!(LogCategory::Core, "OxideUI initialization completed successfully");
        }
        Err(e) => {
            oxide_error!(LogCategory::Core, "Failed to initialize OxideUI: error={}, config_hash={:?}", e, config.skip_problematic_fonts);
            return Err(e);
        }
    }

    println!("Hello World - OxideUI initialized with optimized font loading!");

    // Print configuration info before running
    println!("Font loading optimizations applied:");
    println!("- Skipped problematic fonts: {}", config.skip_problematic_fonts);
    println!("- Max font faces: {:?}", config.max_font_faces);
    println!("- Custom font directories: {:?}", config.custom_font_dirs);
    println!("- Preferred fonts: {:?}", config.preferred_fonts);

    oxide_info!(LogCategory::Platform, "Starting application window with title='Hello OxideUI', size=400x300");

    // Run the application
    ApplicationBuilder::new()
        .title("Hello OxideUI")
        .window(WindowBuilder::new().with_size(400.0, 300.0).resizable(true))
        .run(build_ui())
}

fn setup_logging() -> Result<()> {
    // Create a logging configuration that reduces noise
    let mut category_levels = HashMap::new();
    category_levels.insert(LogCategory::Core, LogLevel::Info);
    category_levels.insert(LogCategory::Renderer, LogLevel::Info);
    category_levels.insert(LogCategory::Platform, LogLevel::Info);
    category_levels.insert(LogCategory::Event, LogLevel::Info);
    category_levels.insert(LogCategory::Performance, LogLevel::Info);
    category_levels.insert(LogCategory::Animation, LogLevel::Info);
    // Vulkan errors only at WARN level to reduce validation spam
    category_levels.insert(LogCategory::Vulkan, LogLevel::Warn);
    // Text and Layout debug disabled by default to prevent spam
    category_levels.insert(LogCategory::Text, LogLevel::Error);
    category_levels.insert(LogCategory::Layout, LogLevel::Error);

    let logging_config = LoggingConfig {
        global_level: LogLevel::Info,
        category_levels,
        enable_text_debug: false,
        enable_layout_debug: false,
        rate_limit_interval_ms: 5000, // 5 seconds
        max_logs_per_interval: 3,     // Max 3 messages per interval
    };

    // Initialize logging with the configuration
    oxide_core::logging::init_with_config(logging_config)?;
    
    oxide_info!(LogCategory::Core, "Logging system initialized with noise reduction: text_debug=false, layout_debug=false, vulkan_level=warn");
    Ok(())
}

fn build_ui() -> impl Widget {
    Container::new()
        .background(Color::rgb(0.2, 0.2, 0.8))  
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
                            .color(Color::rgb(1.0, 1.0, 0.0))  
                    ),
                    Box::new(
                        Text::new("Welcome to the future of UI development!")
                            .size(20.0)
                            .color(Color::rgb(0.0, 1.0, 0.0))  
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