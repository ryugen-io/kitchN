use hcore_lib::config::HyprConfig;
use hcore_lib::logger;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    println!("Initializing Hyprcore (Rust Native)...");

    // 1. Load Configuration
    let config = HyprConfig::load()?;
    
    // 2. Logging Example
    // Directly using the Rust API gives us type safety and direct struct access
    logger::log_to_terminal(&config, "info", "rust_example", "Hello from Native Rust!");
    logger::log_to_terminal(&config, "success", "rust_example", "This consumes the crate directly.");

    // 3. Library Usage Example
    // We can use generic paths directly
    let src = Path::new("non_existent_dir");
    let dest = Path::new("output.fpkg");

    println!("\nAttempting to pack (safe Rust API)...");
    if let Err(e) = hcore_lib::packager::pack(src, dest) {
        println!("Caught expected error: {:#}", e);
    }

    Ok(())
}
