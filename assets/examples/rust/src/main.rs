use kitchn_lib::config::Cookbook;
use kitchn_lib::logger;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    println!("Initializing Kitchn (Rust Native)...");

    // 1. Load Configuration
    let mut config = Cookbook::load()?;
    
    // Set App Name Programmatically for this config instance
    config.layout.logging.app_name = "RustNative".to_string();

    // 2. Logging Example
    // Directly using the Rust API gives us type safety and direct struct access
    logger::log_to_terminal(&config, "info", "rust_example", "Hello from Native Rust!");
    
    // Explicitly write to file with override option (although config.app_name is already set)
    logger::log_to_file(&config, "success", "rust_example", "This consumes the crate directly.", Some("RustOverride"))?;

    // 3. Library Usage Example
    // We can use generic paths directly
    let src = Path::new("non_existent_dir");
    let dest = Path::new("output.bag");

    println!("\nAttempting to pack (safe Rust API)...");
    if let Err(e) = kitchn_lib::packager::pack(src, dest) {
        println!("Caught expected error: {:#}", e);
    }

    Ok(())
}
