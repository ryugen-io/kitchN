use crate::args::PantryCommands;
use crate::logging::{log, log_msg};
use anyhow::Result;
use colored::*;
use k_lib::config::Cookbook;
use k_lib::db::Pantry;

pub fn execute(command: Option<PantryCommands>, db: &mut Pantry, config: &Cookbook) -> Result<()> {
    match command {
        Some(PantryCommands::Clean) => {
            let count = db.list().len();
            if count == 0 {
                log_msg(config, "pantry_empty", "pantry is already empty");
            } else {
                db.clean();
                db.save()?;
                log_msg(
                    config,
                    "pantry_clean_ok",
                    &format!("removed {} ingredients", count),
                );
            }
        }
        None => {
            list_pantry(db, config);
        }
    }
    Ok(())
}

fn list_pantry(db: &Pantry, config: &Cookbook) {
    println!("{}", "\nStocked Ingredients (Pantry):\n".bold().underline());

    let fragments = db.list();
    if fragments.is_empty() {
        log(config, "pantry_empty");
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
