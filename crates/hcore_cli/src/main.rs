use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use hcore_lib::config::HyprConfig;
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
    /// Install a .frag fragment or .fpkg package
    Install { path: PathBuf },
    /// Pack .frag files from a directory into a .fpkg
    Pack {
        /// Directory containing .frag files
        input: PathBuf,
        /// Output .fpkg file (optional, defaults to <dirname>.fpkg)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Sync all fragments (render templates)
    Sync,
    /// List installed fragments
    List,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let dirs = ProjectDirs::from("", "", "hyprcore").context("Could not determine project dirs")?;
    let data_dir = dirs.data_dir(); // ~/.local/share/hyprcore
    let fragments_dir = data_dir.join("fragments");

    match cli.command {
        Commands::Install { path } => {
            install_fragment_or_package(&path, &fragments_dir)?;
            log("install_ok");
            sync_fragments(&fragments_dir)?;
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
            sync_fragments(&fragments_dir)?;
        }
        Commands::List => {
            list_fragments(&fragments_dir)?;
        }
    }

    Ok(())
}

fn install_fragment_or_package(path: &Path, fragments_dir: &Path) -> Result<()> {
    if !path.exists() {
        return Err(anyhow!("File not found: {:?}", path));
    }

    fs::create_dir_all(fragments_dir)?;

    // Check if it's a .fpkg package
    if path.extension().is_some_and(|ext| ext == "fpkg") {
        packager::unpack(path, fragments_dir)?;
    } else {
        // Single .frag file
        let filename = path.file_name().context("Invalid filename")?;
        let target = fragments_dir.join(filename);
        fs::copy(path, target)?;
    }
    Ok(())
}

fn list_fragments(fragments_dir: &Path) -> Result<()> {
    if !fragments_dir.exists() {
        log("list_empty");
        return Ok(());
    }

    for entry in fs::read_dir(fragments_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "frag") {
            // Try to parse to get ID
            let content = fs::read_to_string(&path)?;
            if let Ok(pkg) = toml::from_str::<Fragment>(&content) {
                println!(
                    "  {} ({})",
                    pkg.meta.name,
                    path.file_name().unwrap().to_string_lossy()
                );
            } else {
                eprintln!(
                    "  {} (Invalid)",
                    path.file_name().unwrap().to_string_lossy()
                );
            }
        }
    }
    Ok(())
}

fn sync_fragments(fragments_dir: &Path) -> Result<()> {
    log("sync_start");

    // 1. Load Config
    let config = HyprConfig::load().context("Failed to load Hyprcore config")?;

    // 4. Process Fragments
    if !fragments_dir.exists() {
        log("sync_empty");
        return Ok(());
    }

    for entry in fs::read_dir(fragments_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "frag") {
            // Use Library Processor
            processor::install(&path, &config)?;
        }
    }

    log("sync_ok");
    Ok(())
}
