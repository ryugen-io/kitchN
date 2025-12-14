use crate::logging::{log, log_msg};
use anyhow::Result;
use k_lib::config::Cookbook;
use k_lib::db::Pantry;
use k_lib::processor;

pub fn execute(db: &Pantry, config: &Cookbook, force: bool) -> Result<()> {
    let ingredients = db.list();
    if ingredients.is_empty() {
        log(config, "cook_empty");
        return Ok(());
    }

    let count = ingredients.len();
    let mut hook_failures = 0;

    for pkg in ingredients {
        log_msg(
            config,
            "cook_start",
            &format!("simmering <primary>{}</primary>", pkg.meta.name),
        );
        if !processor::apply(pkg, config, force)? {
            hook_failures += 1;
        }
    }

    if hook_failures > 0 {
        log_msg(
            config,
            "cook_ok",
            &format!(
                "cooked {} ingredients successfully but {} hooks failed",
                count, hook_failures
            ),
        );
    } else {
        log_msg(
            config,
            "cook_ok",
            &format!("cooked {} ingredients successfully", count),
        );
    }

    Ok(())
}
