use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "kitchn", version, about = "Kitchn Ingredient Manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable debug mode with verbose logging in a separate terminal
    #[arg(long, global = true)]
    pub debug: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
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
    Cook {
        /// Persistently toggle force mode (always overwrite)
        #[arg(long)]
        toggle_force: bool,
        /// Force overwrite for this run only
        #[arg(long)]
        force: bool,
    },
    /// List stocked ingredients
    Pantry {
        #[command(subcommand)]
        command: Option<PantryCommands>,
    },
    /// Bake cookbook into binary pastry for faster startup
    Bake,
    /// Internal command to watch logs via socket (Hidden)
    #[command(hide = true)]
    InternalWatch { socket_path: PathBuf },
}

#[derive(Subcommand, Debug)]
pub enum PantryCommands {
    /// Remove all ingredients from the pantry
    Clean,
}
