use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct ThemeConfig {
    pub meta: ThemeMeta,
    pub settings: ThemeSettings,
    pub colors: HashMap<String, String>,
    pub fonts: HashMap<String, String>, // Using HashMap to allow dynamic font keys like "mono", "ui", "size_mono"
    pub include: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ThemeMeta {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct ThemeSettings {
    pub active_icons: String,
}

#[derive(Debug, Deserialize)]
pub struct IconsConfig {
    pub nerdfont: HashMap<String, String>,
    pub ascii: HashMap<String, String>,
    pub include: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct LayoutConfig {
    pub tag: TagConfig,
    pub labels: HashMap<String, String>,
    pub structure: StructureConfig,
    pub logging: LoggingConfig,
    pub include: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct TagConfig {
    pub prefix: String,
    pub suffix: String,
    pub transform: String,
    pub min_width: usize,
    pub alignment: String,
}

#[derive(Debug, Deserialize)]
pub struct StructureConfig {
    pub terminal: String,
    pub file: String,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    pub base_dir: String,
    pub path_structure: String,
    pub filename_structure: String,
    pub timestamp_format: String,
    pub write_by_default: bool,
}

#[derive(Debug, Deserialize)]
pub struct DictionaryConfig {
    pub presets: HashMap<String, Preset>,
    pub include: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct Preset {
    pub level: String,
    pub scope: Option<String>,
    pub msg: String,
}

#[derive(Debug)]
pub struct HyprConfig {
    pub theme: ThemeConfig,
    pub icons: IconsConfig,
    pub layout: LayoutConfig,
    pub dictionary: DictionaryConfig,
}

use directories::ProjectDirs;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Could not determine config directory")]
    ConfigDirNotFound,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),
}

impl HyprConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let config_dir = Self::get_config_dir()?;
        Self::load_from_dir(&config_dir)
    }

    pub fn load_from_dir(config_dir: &Path) -> Result<Self, ConfigError> {
        let theme: ThemeConfig = Self::load_with_includes(&config_dir.join("theme.toml"))?;
        let icons: IconsConfig = Self::load_with_includes(&config_dir.join("icons.toml"))?;
        let layout: LayoutConfig = Self::load_with_includes(&config_dir.join("layout.toml"))?;
        let dictionary: DictionaryConfig =
            Self::load_with_includes(&config_dir.join("dictionary.toml"))?;

        Ok(HyprConfig {
            theme,
            icons,
            layout,
            dictionary,
        })
    }

    fn get_config_dir() -> Result<PathBuf, ConfigError> {
        // Use XDG_CONFIG_HOME/hyprcore or ~/.config/hyprcore
        if let Some(proj_dirs) = ProjectDirs::from("", "", "hyprcore") {
            return Ok(proj_dirs.config_dir().to_path_buf());
        }
        Err(ConfigError::ConfigDirNotFound)
    }

    fn load_with_includes<T: for<'a> Deserialize<'a>>(path: &Path) -> Result<T, ConfigError> {
        let value = Self::load_value_recursive(path)?;
        let config: T = value.try_into()?;
        Ok(config)
    }

    fn load_value_recursive(path: &Path) -> Result<toml::Value, ConfigError> {
        let content = fs::read_to_string(path)?;
        let current_value: toml::Value = toml::from_str(&content)?;

        // Check for "include" array of strings
        let mut bases = Vec::new();

        if let Some(includes) = current_value.get("include").and_then(|v| v.as_array()) {
            for inc in includes {
                if let Some(inc_str) = inc.as_str() {
                    // Resolve path relative to current config file
                    let inc_path = if let Some(parent) = path.parent() {
                        parent.join(inc_str)
                    } else {
                        PathBuf::from(inc_str)
                    };

                    // Recursive load
                    let base_value = Self::load_value_recursive(&inc_path)?;
                    bases.push(base_value);
                }
            }
        }

        // Merge bases first (in order), then current on top
        let mut final_value = if bases.is_empty() {
            toml::Value::Table(toml::map::Map::new())
        } else {
            let mut acc = bases.remove(0);
            for b in bases {
                Self::deep_merge(&mut acc, b);
            }
            acc
        };

        Self::deep_merge(&mut final_value, current_value);
        Ok(final_value)
    }

    fn deep_merge(target: &mut toml::Value, source: toml::Value) {
        match (target, source) {
            (toml::Value::Table(t), toml::Value::Table(s)) => {
                for (k, v) in s {
                    if let Some(existing) = t.get_mut(&k) {
                        Self::deep_merge(existing, v);
                    } else {
                        t.insert(k, v);
                    }
                }
            }
            (t, s) => *t = s,
        }
    }
}
