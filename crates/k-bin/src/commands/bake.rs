use crate::logging::{log, log_msg};
use anyhow::{Result, anyhow};
use directories::ProjectDirs;
use k_lib::config::Cookbook;
use std::fs;

pub fn execute(dirs: &ProjectDirs, config: &Cookbook) -> Result<()> {
    log(config, "bake_start");
    let config_dir = dirs.config_dir();
    let cache_dir = dirs.cache_dir();

    log_msg(config, "bake_scan", &config_dir.to_string_lossy());

    let files = ["theme.toml", "icons.toml", "layout.toml", "cookbook.toml"];
    for f in files {
        let p = config_dir.join(f);
        if p.exists() {
            log_msg(config, "bake_file", f);
        }
    }

    let bin_path = cache_dir.join("pastry.bin");
    if bin_path.exists() {
        let _ = fs::remove_file(&bin_path);
    }

    match Cookbook::load_from_dir(config_dir) {
        Ok(new_config) => {
            log_msg(config, "bake_save", &bin_path.to_string_lossy());
            if let Err(e) = new_config.save_binary(&bin_path) {
                log(config, "bake_fail");
                return Err(anyhow!("Failed to save binary config: {}", e));
            }
            log_msg(
                config,
                "bake_ok",
                &format!("baked configuration to {}", bin_path.display()),
            );
        }
        Err(e) => {
            log(config, "bake_fail");
            return Err(anyhow!("Failed to load configuration: {}", e));
        }
    }

    Ok(())
}
