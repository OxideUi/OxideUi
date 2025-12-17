use strato_core::inspector::{inspector, InspectorConfig};
use strato_core::Result;
use strato_ui::{InitBuilder, InitConfig};

fn main() -> Result<()> {
    println!("StratoUI Custom Initialization Example");
    println!("=====================================");

    // Example 1: Basic initialization with default config
    println!("\n1. Basic initialization with default config:");
    let mut builder = InitBuilder::new();
    builder.init_all()?;

    inspector().configure(InspectorConfig {
        enabled: true,
        ..Default::default()
    });

    println!("   Core: ✓");
    println!("   Widgets: ✓");
    println!("   Platform: ✓");
    println!("   Inspector: ✓ Enabled for runtime overlays");
    println!("   Complete: ✓");

    // Example 2: Check global text renderer
    println!("\n2. Global text renderer status:");
    match strato_ui::get_text_renderer() {
        Some(_renderer) => {
            println!("   Global text renderer: ✓ Available");
        }
        None => {
            println!("   Global text renderer: ✗ Not available");
        }
    }

    println!("\nExample completed successfully!");
    println!("The new initialization system provides:");
    println!("  • Granular control over initialization steps");
    println!("  • Better error handling and reporting");
    println!("  • Improved text rendering capabilities");
    println!("  • Font management and filtering");

    Ok(())
}
