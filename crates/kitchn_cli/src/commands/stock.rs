use crate::logging::log_msg;
use anyhow::{Context, Result, anyhow};
use k_lib::config::Cookbook;
use k_lib::db::Pantry;
use k_lib::ingredient::Ingredient;
use std::fs;
use std::io::Read;
use std::path::Path;

pub fn stock_pantry(path: &Path, db: &mut Pantry, config: &Cookbook) -> Result<Vec<Ingredient>> {
    let mut installed_list = Vec::new();

    if !path.exists() {
        return Err(anyhow!("File not found: {:?}", path));
    }

    if path.extension().is_some_and(|ext| ext == "bag") {
        let file = fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            if file.name().ends_with(".ing") {
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                let pkg: Ingredient = toml::from_str(&content).with_context(|| {
                    format!("Failed to parse ingredient inside zip: {}", file.name())
                })?;

                log_msg(
                    config,
                    "stock_ok",
                    &format!("stocked {} v{}", pkg.meta.name, pkg.meta.version),
                );

                let pkg_clone = pkg.clone();
                db.store(pkg)?;
                installed_list.push(pkg_clone);
            }
        }
    } else {
        let content = fs::read_to_string(path)?;
        let pkg: Ingredient = toml::from_str(&content)
            .with_context(|| format!("Failed to parse ingredient: {:?}", path))?;

        log_msg(
            config,
            "stock_ok",
            &format!("stocked {} v{}", pkg.meta.name, pkg.meta.version),
        );
        let pkg_clone = pkg.clone();
        db.store(pkg)?;
        installed_list.push(pkg_clone);
    }
    Ok(installed_list)
}
