use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use kitchn_lib::config::Cookbook;
use kitchn_lib::db::Pantry;
use kitchn_lib::{ingredient::Ingredient, packager, processor};
use log::{debug, warn};
use simplelog::{Config, LevelFilter, WriteLogger};

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::env;

/// Log via kitchn-log preset
fn log(preset: &str) {
    let _ = Command::new("kitchn-log")
        .arg("--app")
        .arg("kitchn")
        .arg(preset)
        .status();
}

/// Log via kitchn-log preset with custom message
fn log_msg(preset: &str, msg: &str) {
    let _ = Command::new("kitchn-log")
        .arg("--app")
        .arg("kitchn")
        .arg(preset)
        .arg(msg)
        .status();
}

#[derive(Parser)]
#[command(name = "kitchn", version, about = "Kitchn Ingredient Manager")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Enable debug mode with verbose logging in a separate terminal
    #[arg(long, global = true)]
    debug: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Stock .ing ingredients or .bag packages into the pantry
    Stock { path: PathBuf },
    /// Wrap .ing ingredients from a directory into a .bag package
    Wrap {
        /// Directory containing .ing files
        input: PathBuf,
        /// Output .bag file (optional, defaults to <dirname>.bag)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Cook all ingredients from pantry into the system
    Cook,
    /// List stocked ingredients
    Pantry,
    /// Bake cookbook into binary pastry for faster startup
    Bake,
}

fn get_log_path() -> PathBuf {
    env::temp_dir().join("kitchn-debug.log")
}

fn init_logging(force_enable: bool) -> Result<bool> {
    let log_path = get_log_path();
    let should_log = force_enable || log_path.exists();

    if should_log {
        // Create file if it doesn't exist, open for append if it does
        let file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .context("Failed to open debug log file")?;

        // Initialize logger
        let _ = WriteLogger::init(LevelFilter::Debug, Config::default(), file);
        debug!("Logging initialized to {:?}", log_path);
    }
    Ok(should_log)
}

fn spawn_debug_viewer() -> Result<()> {
    let log_path = get_log_path();

    // Reset log file for fresh run if we are spawning the viewer explicitly
    // This gives a clean state for "Starting debug mode"
    File::create(&log_path).context("Failed to reset log file")?;

    // Detect terminal
    let terminal = env::var("TERMINAL").ok().or_else(|| {
        let terminals = ["alacritty", "kitty", "rio", "gnome-terminal", "xterm"];
        for term in terminals {
            if which::which(term).is_ok() {
                return Some(term.to_string());
            }
        }
        None
    });

    if let Some(term) = terminal {
        debug!("Spawning debug viewer with: {}", term);
        // Spawn terminal tailing the log
         let _ = Command::new(&term)
            .arg("-e")
            .arg("tail")
            .arg("-f")
            .arg(&log_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to spawn debug terminal")?;
            
        println!("Debug Mode Started.");
        println!("Verbose logs are streaming to the new terminal window.");
        println!("Run 'kitchn' commands normally, and they will appear there.");
    } else {
        warn!("No supported terminal emulator found.");
        println!("Logs are being written to: {:?}", log_path);
        println!("You can tail them manually: tail -f {:?}", log_path);
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let logging_enabled = init_logging(cli.debug)?;

    // Handle Standalone Debug Mode
    match cli.command {
        None => {
            if cli.debug {
                // kitchn --debug
                spawn_debug_viewer()?;
            } else {
                // kitchn (no args) -> Print Help
                use clap::CommandFactory;
                Cli::command().print_help()?;
            }
            return Ok(());
        }
        Some(cmd) => {
            if logging_enabled {
                 debug!("Executing command: {:?}", cmd);
            }
            // Proceed to execute command
            process_command(cmd)?;
        }
    }

    Ok(())
}

fn process_command(cmd: Commands) -> Result<()> {
    let dirs = ProjectDirs::from("", "", "kitchn").context("Could not determine project dirs")?;
    let data_dir = dirs.data_dir(); // ~/.local/share/kitchn
    let db_path = data_dir.join("pantry.db");
    let mut db = Pantry::load(&db_path)?;

    match cmd {
        Commands::Stock { path } => {
            let installed = stock_pantry(&path, &mut db)?;
            db.save()?;
            
            // Apply only the newly stocked ingredients
            let config = Cookbook::load().context("Failed to load Kitchn cookbook")?;
            for pkg in installed {
                log_msg("cook_start", &format!("simmering {}", pkg.meta.name));
                processor::apply(&pkg, &config)?;
            }
        }
        Commands::Wrap { input, output } => {
            let out = output.unwrap_or_else(|| {
                let name = input.file_name().unwrap_or_default().to_string_lossy();
                PathBuf::from(format!("{}.bag", name))
            });
            packager::pack(&input, &out)?;
            log_msg("wrap_ok", &format!("wrapped package to {}", out.display()));
        }
        Commands::Cook => {
            cook_db(&db)?;
        }
        Commands::Pantry => {
            list_pantry(&db);
        }
        Commands::Bake => {
            bake_config(&dirs)?;
        }
    }
    Ok(())
}

fn stock_pantry(path: &Path, db: &mut Pantry) -> Result<Vec<Ingredient>> {
    let mut installed_list = Vec::new();

    if !path.exists() {
        return Err(anyhow!("File not found: {:?}", path));
    }

    // Check if it's a .bag package
    if path.extension().is_some_and(|ext| ext == "bag") {
        // Read zip
        let file = fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            if file.name().ends_with(".ing") {
                let mut content = String::new();
                std::io::Read::read_to_string(&mut file, &mut content)?;
                
                let pkg: Ingredient = toml::from_str(&content)
                    .with_context(|| format!("Failed to parse ingredient inside zip: {}", file.name()))?;
                
                log_msg("stock_ok", &format!("stocked {} v{}", pkg.meta.name, pkg.meta.version));
                // We clone here because store takes ownership, but we want to return it too
                // Or proper: store takes ownership. We can clone before storing.
                let pkg_clone = pkg.clone();
                db.store(pkg)?;
                installed_list.push(pkg_clone);
            }
        }
    } else {
        // Single .ing file
        let content = fs::read_to_string(path)?;
        let pkg: Ingredient = toml::from_str(&content)
            .with_context(|| format!("Failed to parse ingredient: {:?}", path))?;
            
        log_msg("stock_ok", &format!("stocked {} v{}", pkg.meta.name, pkg.meta.version));
        let pkg_clone = pkg.clone();
        db.store(pkg)?;
        installed_list.push(pkg_clone);
    }
    Ok(installed_list)
}

fn list_pantry(db: &Pantry) {
    use colored::*;
    println!("{}", "\nStocked Ingredients (Pantry):\n".bold().underline());

    let fragments = db.list();
    if fragments.is_empty() {
        log("pantry_empty");
        return;
    }

    for pkg in fragments {
        println!(
            "  {} {}\n    {}\n    {}",
            pkg.meta.name.blue().bold(),
            format!("v{}", pkg.meta.version).green(),
            pkg.meta.description.italic(),
            format!("by {}", pkg.meta.authors.join(", ")).dimmed()
        );
        println!();
    }
}

fn cook_db(db: &Pantry) -> Result<()> {
    // 1. Load Cookbook
    let config = Cookbook::load().context("Failed to load Kitchn cookbook")?;

    // 2. Process Ingredients
    let ingredients = db.list();
    if ingredients.is_empty() {
        log("cook_empty");
        return Ok(());
    }

    let count = ingredients.len();
    for pkg in ingredients {
         log_msg("cook_start", &format!("simmering <primary>{}</primary>", pkg.meta.name));
         processor::apply(pkg, &config)?;
    }

    log_msg("cook_ok", &format!("cooked {} ingredients successfully", count));
    Ok(())
}

fn bake_config(dirs: &ProjectDirs) -> Result<()> {
    log("bake_start");
    let config_dir = dirs.config_dir();
    let cache_dir = dirs.cache_dir();
    
    log_msg("bake_scan", &config_dir.to_string_lossy());

    // Verbose check for standard files to inform user
    let files = ["theme.toml", "icons.toml", "layout.toml", "cookbook.toml"];
    for f in files {
        let p = config_dir.join(f);
        if p.exists() {
             log_msg("bake_file", f);
        }
    }

    // Force load from TOMLs by bypassing load() which might use stale cache
    // We use load_from_dir but strictly speaking load_from_dir will use cache if fresh.
    // We force refresh by deleting the bin first.
    
    let bin_path = cache_dir.join("pastry.bin");
    if bin_path.exists() {
        let _ = fs::remove_file(&bin_path);
    } 

    // Now load will definitely parse TOMLs
    match Cookbook::load_from_dir(config_dir) {
        Ok(config) => {
            log_msg("bake_save", &bin_path.to_string_lossy());
            if let Err(e) = config.save_binary(&bin_path) {
                log("bake_fail");
                return Err(anyhow!("Failed to save binary config: {}", e));
            }
            log_msg("bake_ok", &format!("baked configuration to {}", bin_path.display()));
        }
        Err(e) => {
             log("bake_fail");
             return Err(anyhow!("Failed to load configuration: {}", e));
        }
    }
    
    Ok(())
}
