use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CliConfig {
    pub force_cooking: bool,
}

impl CliConfig {
    pub fn load() -> Result<Self> {
        let path = Self::get_config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path).context("Failed to read CLI config file")?;

        let config: Self = toml::from_str(&content).context("Failed to parse CLI config")?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::get_config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create CLI config directory")?;
        }

        let content = toml::to_string_pretty(self).context("Failed to serialize CLI config")?;

        fs::write(&path, content).context("Failed to write CLI config file")?;

        Ok(())
    }

    fn get_config_path() -> Result<PathBuf> {
        let dirs = ProjectDirs::from("", "", "kitchn")
            .context("Could not determine project directories")?;
        Ok(dirs.config_dir().join("cli.toml"))
    }
}
