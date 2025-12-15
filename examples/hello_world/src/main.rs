//! Hello World example for StratoUI with improved logging

// Import standard del framework
use strato_ui::prelude::*;
use strato_ui::{InitBuilder, InitConfig};
use strato_ui::strato_core::Result;
use strato_ui::strato_core::{
    config::{StratoConfig, LoggingConfig},
    logging::{LogLevel, LogCategory},
    strato_info, strato_debug, strato_error, strato_text_debug,
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

    strato_info!(LogCategory::Core, "StratoUI Hello World starting with optimized configuration");

    let mut builder = InitBuilder::new()
        .with_config(config.clone());
    
    match builder.init_all() {
        Ok(_) => {
            strato_info!(LogCategory::Core, "StratoUI initialization completed successfully");
        }
        Err(e) => {
            strato_error!(LogCategory::Core, "Failed to initialize StratoUI: error={}, config_hash={:?}", e, config.skip_problematic_fonts);
            return Err(e);
        }
    }

    println!("Hello World - StratoUI initialized with optimized font loading!");

    // Print configuration info before running
    println!("Font loading optimizations applied:");
    println!("- Skipped problematic fonts: {}", config.skip_problematic_fonts);
    println!("- Max font faces: {:?}", config.max_font_faces);
    println!("- Custom font directories: {:?}", config.custom_font_dirs);
    println!("- Preferred fonts: {:?}", config.preferred_fonts);

    strato_info!(LogCategory::Platform, "Starting application window with title='Hello StratoUI', size=400x300");

    // Run the application
    ApplicationBuilder::new()
        .title("Hello StratoUI")
        .window(WindowBuilder::new().with_size(400.0, 300.0).resizable(true))
        .run(build_ui())
}

fn setup_logging() -> Result<()> {
    // Create a logging configuration that reduces noise
    let mut category_levels = HashMap::new();
    category_levels.insert(LogCategory::Core.to_string(), LogLevel::Info.to_string());
    category_levels.insert(LogCategory::Renderer.to_string(), LogLevel::Info.to_string());
    category_levels.insert(LogCategory::Platform.to_string(), LogLevel::Info.to_string());
    // category_levels.insert(LogCategory::Event.to_string(), LogLevel::Info.to_string()); // Event category missing
    // category_levels.insert(LogCategory::Performance.to_string(), LogLevel::Info.to_string()); // Performance category missing
    // category_levels.insert(LogCategory::Animation.to_string(), LogLevel::Info.to_string()); // Animation category missing
    
    // Vulkan errors only at WARN level to reduce validation spam
    category_levels.insert(LogCategory::Vulkan.to_string(), LogLevel::Warn.to_string());
    
    // Text and Layout debug disabled by default to prevent spam
    category_levels.insert(LogCategory::Text.to_string(), LogLevel::Error.to_string());
    category_levels.insert("layout".to_string(), LogLevel::Error.to_string()); // Layout is a string key in config defaults

    let logging_config = LoggingConfig {
        category_levels,
        enable_text_debug: false,
        enable_layout_debug: false,
        rate_limit_seconds: 5,
        max_rate_limit_count: 3,
    };

    // Initialize logging with the configuration
    strato_ui::strato_core::logging::init(&logging_config)
        .map_err(|e| strato_ui::strato_core::StratoError::other(format!("Logging init failed: {}", e)))?;
    
    strato_info!(LogCategory::Core, "Logging system initialized with noise reduction: text_debug=false, layout_debug=false, vulkan_level=warn");
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
                        Text::new("Hello, StratoUI!")
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