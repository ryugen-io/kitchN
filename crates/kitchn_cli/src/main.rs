use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use kitchn_lib::config::Cookbook;
use kitchn_lib::db::Pantry;
use kitchn_lib::{ingredient::Ingredient, packager, processor};

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Log via kitchn-log preset
fn log(preset: &str) {
    let _ = Command::new("kitchn-log").arg(preset).status();
}

#[derive(Parser)]
#[command(name = "kitchn", version, about = "Kitchn Ingredient Manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install a .ing ingredient or .pantry pantry package to the pantry
    Install { path: PathBuf },
    /// Pack .ing ingredients from a directory into a .pantry package
    Pack {
        /// Directory containing .ing files
        input: PathBuf,
        /// Output .pantry file (optional, defaults to <dirname>.pantry)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Sync all ingredients from pantry
    Sync,
    /// List ingredients in pantry
    List,
    /// Bake cookbook into binary recipe for faster startup
    Bake,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let dirs = ProjectDirs::from("", "", "kitchn").context("Could not determine project dirs")?;
    let data_dir = dirs.data_dir(); // ~/.local/share/kitchn
    let db_path = data_dir.join("pantry.db");
    let mut db = Pantry::load(&db_path)?;

    match cli.command {
        Commands::Install { path } => {
            let installed = install_to_pantry(&path, &mut db)?;
            db.save()?;
            log("install_ok");
            
            // Apply only the newly installed ingredients
            let config = Cookbook::load().context("Failed to load Kitchn cookbook")?;
            for pkg in installed {
                println!("Applying: {}", pkg.meta.name);
                processor::apply(&pkg, &config)?;
            }
        }
        Commands::Pack { input, output } => {
            let out = output.unwrap_or_else(|| {
                let name = input.file_name().unwrap_or_default().to_string_lossy();
                PathBuf::from(format!("{}.pantry", name))
            });
            packager::pack(&input, &out)?;
            log("pack_ok");
        }
        Commands::Sync => {
            sync_db(&db)?;
        }
        Commands::List => {
            list_db(&db);
        }
        Commands::Bake => {
            bake_config(&dirs)?;
        }
    }

    Ok(())
}

fn install_to_pantry(path: &Path, db: &mut Pantry) -> Result<Vec<Ingredient>> {
    let mut installed_list = Vec::new();

    if !path.exists() {
        return Err(anyhow!("File not found: {:?}", path));
    }

    // Check if it's a .pantry package
    if path.extension().is_some_and(|ext| ext == "pantry") {
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
                
                println!("Stocking from pantry: {}", pkg.meta.name);
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
            
        println!("Stocking ingredient: {}", pkg.meta.name);
        let pkg_clone = pkg.clone();
        db.store(pkg)?;
        installed_list.push(pkg_clone);
    }
    Ok(installed_list)
}

fn list_db(db: &Pantry) {
    use colored::*;
    println!("{}", "\nInstalled Ingredients (Database):\n".bold().underline());

    let fragments = db.list();
    if fragments.is_empty() {
        log("list_empty");
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

fn sync_db(db: &Pantry) -> Result<()> {
    log("sync_start");

    // 1. Load Cookbook
    let config = Cookbook::load().context("Failed to load Kitchn cookbook")?;

    // 2. Process Ingredients
    let ingredients = db.list();
    if ingredients.is_empty() {
        log("sync_empty");
        return Ok(());
    }

    for pkg in ingredients {
         processor::apply(pkg, &config)?;
    }

    log("sync_ok");
    Ok(())
}

fn bake_config(dirs: &ProjectDirs) -> Result<()> {
    log("bake_start");
    let config_dir = dirs.config_dir();
    let cache_dir = dirs.cache_dir();
    
    // Force load from TOMLs by bypassing load() which might use stale cache
    // We use load_from_dir but strictly speaking load_from_dir will use cache if fresh.
    // We force refresh by deleting the bin first.
    
    let bin_path = cache_dir.join("recipe.bin");
    if bin_path.exists() {
        let _ = fs::remove_file(&bin_path);
    } 

    // Now load will definitely parse TOMLs
    match Cookbook::load_from_dir(config_dir) {
        Ok(config) => {
            if let Err(e) = config.save_binary(&bin_path) {
                log("bake_fail");
                return Err(anyhow!("Failed to save binary config: {}", e));
            }
            log("bake_ok");
        }
        Err(e) => {
             log("bake_fail");
             return Err(anyhow!("Failed to load configuration: {}", e));
        }
    }
    
    Ok(())
}
