use strato_platform::{
    application::Application,
    window::WindowBuilder,
};
use tracing::info;
use tracing_subscriber::prelude::*;

mod app;
mod components;
mod views;
mod theme;

fn main() -> anyhow::Result<()> {
    // Setup logging
    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer()
            .with_writer(std::io::stdout)
            .with_ansi(true)
        );
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");

    info!("Starting Modern Dashboard Example");

    // Create application
    let window_builder = WindowBuilder::new()
        .with_title("StratoUI Modern Dashboard")
        .with_size(1440.0, 900.0)
        .resizable(true);
        
    let mut app = Application::new("Modern Dashboard", window_builder);

    // Build the UI
    let dashboard = app::ModernDashboardApp::new();
    app.set_root(dashboard.build());

    info!("Application initialized, entering event loop");
    app.run()
}
