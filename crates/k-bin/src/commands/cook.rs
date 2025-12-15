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

    let total = ingredients.len();
    let mut hook_failures = 0;
    let mut skipped = 0;

    for pkg in ingredients {
        if pkg.meta.ignored {
            log_msg(
                config,
                "cook_skip",
                &format!("ignoring <secondary>{}</secondary> (disabled)", pkg.meta.name),
            );
            skipped += 1;
            continue;
        }

        log_msg(
            config,
            "cook_start",
            &format!("simmering <primary>{}</primary>", pkg.meta.name),
        );
        if !processor::apply(pkg, config, force)? {
            hook_failures += 1;
        }
    }

    let cooked = total - skipped;

    if hook_failures > 0 {
        log_msg(
            config,
            "cook_ok",
            &format!(
                "cooked {} ingredients successfully ({} skipped) but {} hooks failed",
                cooked, skipped, hook_failures
            ),
        );
    } else {
        log_msg(
            config,
            "cook_ok",
            &format!(
                "cooked {} ingredients successfully ({} skipped)",
                cooked, skipped
            ),
        );
    }

    Ok(())
}
