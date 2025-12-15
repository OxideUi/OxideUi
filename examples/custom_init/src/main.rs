use strato_ui::{InitBuilder, InitConfig};
use strato_core::Result;

fn main() -> Result<()> {
    println!("StratoUI Custom Initialization Example");
    println!("=====================================");

    // Example 1: Basic initialization with default config
    println!("\n1. Basic initialization with default config:");
    let mut builder = InitBuilder::new();
    builder.init_all()?;
    
    println!("   Core: ✓");
    println!("   Widgets: ✓");
    println!("   Platform: ✓");
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