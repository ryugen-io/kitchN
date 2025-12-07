use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use hcore_lib::config::HyprConfig;
use hcore_lib::db::FragmentsDb;
use hcore_lib::{fragment::Fragment, packager, processor};

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Log via corelog preset
fn log(preset: &str) {
    let _ = Command::new("corelog").arg(preset).status();
}

#[derive(Parser)]
#[command(name = "hyprcore", version, about = "Hyprcore Fragment Manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install a .frag fragment or .fpkg package to the database
    Install { path: PathBuf },
    /// Pack .frag files from a directory into a .fpkg
    Pack {
        /// Directory containing .frag files
        input: PathBuf,
        /// Output .fpkg file (optional, defaults to <dirname>.fpkg)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Sync all fragments from database
    Sync,
    /// List fragments in database
    List,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let dirs = ProjectDirs::from("", "", "hyprcore").context("Could not determine project dirs")?;
    let data_dir = dirs.data_dir(); // ~/.local/share/hyprcore
    let db_path = data_dir.join("fragments.bin");
    let mut db = FragmentsDb::load(&db_path)?;

    match cli.command {
        Commands::Install { path } => {
            install_to_db(&path, &mut db)?;
            db.save()?;
            log("install_ok");
            // Auto-sync after install
            sync_db(&db)?;
        }
        Commands::Pack { input, output } => {
            let out = output.unwrap_or_else(|| {
                let name = input.file_name().unwrap_or_default().to_string_lossy();
                PathBuf::from(format!("{}.fpkg", name))
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
    }

    Ok(())
}

fn install_to_db(path: &Path, db: &mut FragmentsDb) -> Result<()> {
    if !path.exists() {
        return Err(anyhow!("File not found: {:?}", path));
    }

    // Check if it's a .fpkg package
    if path.extension().is_some_and(|ext| ext == "fpkg") {
        // Read zip
        let file = fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            if file.name().ends_with(".frag") {
                let mut content = String::new();
                std::io::Read::read_to_string(&mut file, &mut content)?;
                
                let pkg: Fragment = toml::from_str(&content)
                    .with_context(|| format!("Failed to parse fragment inside zip: {}", file.name()))?;
                
                println!("Ingesting from package: {}", pkg.meta.name);
                db.install(pkg)?;
            }
        }
    } else {
        // Single .frag file
        let content = fs::read_to_string(path)?;
        let pkg: Fragment = toml::from_str(&content)
            .with_context(|| format!("Failed to parse fragment: {:?}", path))?;
            
        println!("Ingesting fragment: {}", pkg.meta.name);
        db.install(pkg)?;
    }
    Ok(())
}

fn list_db(db: &FragmentsDb) {
    use colored::*;
    println!("{}", "\nInstalled Fragments (Database):\n".bold().underline());

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

fn sync_db(db: &FragmentsDb) -> Result<()> {
    log("sync_start");

    // 1. Load Config
    let config = HyprConfig::load().context("Failed to load Hyprcore config")?;

    // 2. Process Fragments
    let fragments = db.list();
    if fragments.is_empty() {
        log("sync_empty");
        return Ok(());
    }

    for pkg in fragments {
         processor::apply(pkg, &config)?;
    }

    log("sync_ok");
    Ok(())
}
