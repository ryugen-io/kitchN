pub mod bake;
pub mod cook;
pub mod pantry;
pub mod stock;
pub mod wrap;

use crate::args::Commands;
use crate::logging::log_msg;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use k_lib::config::Cookbook;
use k_lib::db::Pantry;
use k_lib::processor;

pub fn process_command(cmd: Commands) -> Result<()> {
    let dirs = ProjectDirs::from("", "", "kitchn").context("Could not determine project dirs")?;
    let data_dir = dirs.data_dir();
    let db_path = data_dir.join("pantry.db");
    let mut db = Pantry::load(&db_path)?;
    let config = Cookbook::load().context("Failed to load Kitchn cookbook")?;

    match cmd {
        Commands::Stock { path } => {
            let installed = stock::stock_pantry(&path, &mut db, &config)?;
            db.save()?;

            for pkg in installed {
                log_msg(
                    &config,
                    "cook_start",
                    &format!("simmering {}", pkg.meta.name),
                );
                let _ = processor::apply(&pkg, &config, false)?;
            }
        }
        Commands::Wrap { input, output } => {
            wrap::execute(input, output, &config)?;
        }
        Commands::Cook {
            toggle_force,
            force,
        } => {
            use crate::cli_config::CliConfig;

            let mut current_force = force;

            // Handle Toggle
            if toggle_force {
                let mut cli_conf = CliConfig::load().unwrap_or_default();
                cli_conf.force_cooking = !cli_conf.force_cooking;

                if cli_conf.force_cooking {
                    log_msg(&config, "info", "FORCE MODE ENABLED (persistent)");
                } else {
                    log_msg(&config, "info", "FORCE MODE DISABLED (persistent)");
                }

                if let Err(e) = cli_conf.save() {
                    log_msg(
                        &config,
                        "error",
                        &format!("Failed to save CLI config: {}", e),
                    );
                }

                // If toggled on, we consider this run forced
                if cli_conf.force_cooking {
                    current_force = true;
                }
            } else {
                // If not toggling, checking persistent state
                if CliConfig::load().unwrap_or_default().force_cooking {
                    current_force = true;
                }
            }

            // If force is active, we MUST reload the cookbook ignoring cache
            let final_config = if current_force {
                // Reload config forcing cache bypass
                match Cookbook::load_no_cache() {
                    Ok(c) => c,
                    Err(e) => {
                        log_msg(
                            &config,
                            "warn",
                            &format!("Failed to reload config with no-cache: {}", e),
                        );
                        config
                    }
                }
            } else {
                config
            };

            if current_force {
                log_msg(&final_config, "warn", "COOKING WITH FORCE (Cache bypassed)");
            }

            cook::execute(&db, &final_config, current_force)?;
        }
        Commands::Pantry { command } => {
            pantry::execute(command, &mut db, &config)?;
        }
        Commands::Bake => {
            bake::execute(&dirs, &config)?;
        }
        Commands::InternalWatch { .. } => {}
    }
    Ok(())
}
