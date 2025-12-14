use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct ThemeConfig {
    pub meta: ThemeMeta,
    pub settings: ThemeSettings,
    pub colors: HashMap<String, String>,
    pub fonts: HashMap<String, String>, // Using HashMap to allow dynamic font keys like "mono", "ui", "size_mono"
    pub include: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ThemeMeta {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ThemeSettings {
    pub active_icons: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IconsConfig {
    pub nerdfont: HashMap<String, String>,
    pub ascii: HashMap<String, String>,
    pub include: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LayoutConfig {
    pub tag: TagConfig,
    pub labels: HashMap<String, String>,
    pub structure: StructureConfig,
    pub logging: LoggingConfig,
    pub include: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TagConfig {
    pub prefix: String,
    pub suffix: String,
    pub transform: String,
    pub min_width: usize,
    pub alignment: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StructureConfig {
    pub terminal: String,
    pub file: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub base_dir: String,
    pub path_structure: String,
    pub filename_structure: String,
    pub timestamp_format: String,
    pub write_by_default: bool,
    #[serde(default = "default_app_name")]
    pub app_name: String,
}

fn default_app_name() -> String {
    "kitchn".to_string()
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DictionaryConfig {
    pub presets: HashMap<String, Preset>,
    pub include: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Preset {
    pub level: String,
    pub scope: Option<String>,
    pub msg: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cookbook {
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

impl Cookbook {
    pub fn load() -> Result<Self, ConfigError> {
        let config_dir = Self::get_config_dir()?;
        // Use cache dir for binary config
        let bin_path = if let Ok(cache_dir) = Self::get_cache_dir() {
            cache_dir.join("pastry.bin")
        } else {
            // Fallback to data dir or config dir if cache not available (unlikely)
            config_dir.join("pastry.bin")
        };

        Self::load_with_cache(&config_dir, &bin_path, false)
    }

    pub fn load_no_cache() -> Result<Self, ConfigError> {
        let config_dir = Self::get_config_dir()?;
        let bin_path = if let Ok(cache_dir) = Self::get_cache_dir() {
            cache_dir.join("pastry.bin")
        } else {
            config_dir.join("pastry.bin")
        };

        Self::load_with_cache(&config_dir, &bin_path, true)
    }

    pub fn load_from_dir(config_dir: &Path) -> Result<Self, ConfigError> {
        // For manual loading, we still check the standard cache location relative to project dirs if possible
        // But if we only have a random dir, we might have to assume cache is there or skip it.
        // For simplicity in CLI/testing, we'll ask for cache dir again.

        let bin_path = if let Ok(cache_dir) = Self::get_cache_dir() {
            cache_dir.join("pastry.bin")
        } else {
            config_dir.join("pastry.bin")
        };

        Self::load_with_cache(config_dir, &bin_path, false)
    }

    pub fn load_with_cache(
        config_dir: &Path,
        bin_path: &Path,
        force: bool,
    ) -> Result<Self, ConfigError> {
        // Try loading from binary cache if it exists and is fresh
        if !force
            && bin_path.exists()
            && Self::is_cache_fresh(bin_path, config_dir)?
            && let Ok(file) = fs::File::open(bin_path)
        {
            let mut reader = std::io::BufReader::new(file);
            // Decode using bincode
            match bincode::serde::decode_from_std_read::<Cookbook, _, _>(
                &mut reader,
                bincode::config::standard(),
            ) {
                Ok(cfg) => {
                    debug!("Loaded configuration from binary cache: {:?}", bin_path);
                    return Ok(cfg);
                }
                Err(e) => {
                    debug!(
                        "Failed to decode binary cache (falling back to TOML): {}",
                        e
                    );
                }
            }
        } else {
            debug!("Binary cache miss or stale (loading from TOMLs)");
        }

        // Fallback: Load from TOML files
        // Fallback: Load from TOML files
        let theme: ThemeConfig = Self::load_with_includes(&config_dir.join("theme.toml"))?;
        let icons: IconsConfig = Self::load_with_includes(&config_dir.join("icons.toml"))?;
        let layout: LayoutConfig = Self::load_with_includes(&config_dir.join("layout.toml"))?;

        // Load System Dictionary (Embedded)
        // This ensures defaults are always available without external files
        const SYSTEM_DICTIONARY: &str = include_str!("defaults.toml");

        // Parse embedded defaults
        let mut dictionary: DictionaryConfig =
            toml::from_str(SYSTEM_DICTIONARY).map_err(ConfigError::Toml)?;

        let user_dict_path = config_dir.join("cookbook.toml");
        if user_dict_path.exists() {
            let user_dict: DictionaryConfig = Self::load_with_includes(&user_dict_path)?;
            // Merge user dict into system dict (user overrides system)
            for (curr_k, curr_v) in user_dict.presets {
                dictionary.presets.insert(curr_k, curr_v);
            }
            // Merge includes if present
            if let Some(user_inc) = user_dict.include {
                dictionary.include = match dictionary.include {
                    Some(mut sys_inc) => {
                        sys_inc.extend(user_inc);
                        Some(sys_inc)
                    }
                    None => Some(user_inc),
                };
            }
        }

        Ok(Cookbook {
            theme,
            icons,
            layout,
            dictionary,
        })
    }

    pub fn save_binary(&self, path: &Path) -> Result<(), ConfigError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(ConfigError::Io)?;
        }
        let file = fs::File::create(path).map_err(ConfigError::Io)?;
        let mut writer = std::io::BufWriter::new(file);

        bincode::serde::encode_into_std_write(self, &mut writer, bincode::config::standard())
            .map_err(|e| ConfigError::Io(std::io::Error::other(e)))?;

        Ok(())
    }

    fn is_cache_fresh(bin_path: &Path, config_dir: &Path) -> Result<bool, ConfigError> {
        let bin_meta = fs::metadata(bin_path)?;
        let bin_mtime = bin_meta.modified()?;

        // Check if the running executable is newer than the cache
        // This ensures that if we update the embedded defaults, the cache is invalidated
        if let Ok(exe_path) = std::env::current_exe()
            && let Ok(exe_meta) = fs::metadata(&exe_path)
            && let Ok(exe_mtime) = exe_meta.modified()
            && exe_mtime > bin_mtime
        {
            return Ok(false); // Executable is newer
        }

        let toml_files = ["theme.toml", "icons.toml", "layout.toml", "cookbook.toml"];
        for file in toml_files {
            let path = config_dir.join(file);
            if path.exists() {
                let meta = fs::metadata(&path)?;
                let mtime = meta.modified()?;
                if mtime > bin_mtime {
                    return Ok(false); // Source is newer
                }
            }
        }
        Ok(true)
    }

    fn get_config_dir() -> Result<PathBuf, ConfigError> {
        // Use XDG_CONFIG_HOME/kitchn or ~/.config/kitchn
        if let Some(proj_dirs) = ProjectDirs::from("", "", "kitchn") {
            return Ok(proj_dirs.config_dir().to_path_buf());
        }
        Err(ConfigError::ConfigDirNotFound)
    }

    fn get_cache_dir() -> Result<PathBuf, ConfigError> {
        // Use XDG_CACHE_HOME/kitchn or ~/.cache/kitchn
        if let Some(proj_dirs) = ProjectDirs::from("", "", "kitchn") {
            return Ok(proj_dirs.cache_dir().to_path_buf());
        }
        Err(ConfigError::ConfigDirNotFound)
    }

    fn load_with_includes<T: for<'a> Deserialize<'a>>(path: &Path) -> Result<T, ConfigError> {
        debug!("Loading config file: {:?}", path);
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_binary_serialization() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("pastry.bin");

        // Create a minimal config for testing
        let config = Cookbook {
            theme: ThemeConfig {
                meta: ThemeMeta {
                    name: "test_theme".to_string(),
                },
                settings: ThemeSettings {
                    active_icons: "nerdfont".to_string(),
                },
                colors: HashMap::new(),
                fonts: HashMap::new(),
                include: None,
            },
            icons: IconsConfig {
                nerdfont: HashMap::new(),
                ascii: HashMap::new(),
                include: None,
            },
            layout: LayoutConfig {
                tag: TagConfig {
                    prefix: "[".to_string(),
                    suffix: "]".to_string(),
                    transform: "none".to_string(),
                    min_width: 0,
                    alignment: "left".to_string(),
                },
                labels: HashMap::new(),
                structure: StructureConfig {
                    terminal: "{msg}".to_string(),
                    file: "{msg}".to_string(),
                },
                logging: LoggingConfig {
                    base_dir: "logs".to_string(),
                    path_structure: "sys.log".to_string(),
                    filename_structure: "log".to_string(),
                    timestamp_format: "%Y".to_string(),
                    write_by_default: false,
                    app_name: "test".to_string(),
                },
                include: None,
            },
            dictionary: DictionaryConfig {
                presets: HashMap::new(),
                include: None,
            },
        };

        // Save
        config
            .save_binary(&config_path)
            .expect("Failed to save binary");
        assert!(config_path.exists());

        // Load (simulating cache hit)
        // Since no TOMLs exist in tempdir, load_with_cache should use bin if fresh check passes (it checks for tomls existence too)
        // In is_cache_fresh, valid if bin exists. If tomls don't exist, mtime check loop skips?
        // "if path.exists() ...". Yes.

        let loaded = Cookbook::load_with_cache(dir.path(), &config_path, false)
            .expect("Failed to load from cache");
        assert_eq!(loaded.theme.meta.name, "test_theme");
    }
}
