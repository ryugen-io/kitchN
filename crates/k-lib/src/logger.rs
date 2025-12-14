use crate::config::Cookbook;
use crate::factory::{ColorResolver, TagFactory};
use anyhow::{Context, Result};
use chrono::Local;
use colored::Colorize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

pub fn log_to_terminal(config: &Cookbook, level: &str, scope: &str, msg: &str) {
    let icon_set_key = &config.theme.settings.active_icons;
    let icon = if icon_set_key == "nerdfont" {
        config
            .icons
            .nerdfont
            .get(level)
            .map(|s| s.as_str())
            .unwrap_or("?")
    } else {
        config
            .icons
            .ascii
            .get(level)
            .map(|s| s.as_str())
            .unwrap_or("?")
    };

    let tag = TagFactory::create_tag(config, level);

    let level_color_hex = config
        .theme
        .colors
        .get(level)
        .or_else(|| config.theme.colors.get("fg"))
        .map(|s| s.as_str())
        .unwrap_or("#ffffff");
    let level_color = ColorResolver::hex_to_color(level_color_hex);

    let structure = &config.layout.structure.terminal;

    let parts = parse_structure(structure);

    for part in parts {
        match part.as_str() {
            "{tag}" => print!("{}", tag.custom_color(level_color)),
            "{icon}" => print!("{}", icon.custom_color(level_color)),
            "{scope}" => print!("{}", scope.white().dimmed()),
            "{msg}" => print_formatted_msg(msg, config),
            _ => print!("{}", part),
        }
    }
    println!();
}

fn parse_structure(structure: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let placeholders = vec!["{tag}", "{scope}", "{icon}", "{msg}"];

    let mut i = 0;
    while i < structure.len() {
        let remainder = &structure[i..];
        let mut matched = false;

        for ph in &placeholders {
            if remainder.starts_with(ph) {
                if !current.is_empty() {
                    parts.push(current.clone());
                    current.clear();
                }
                parts.push(ph.to_string());
                i += ph.len();
                matched = true;
                break;
            }
        }

        if !matched {
            current.push(structure.chars().nth(i).unwrap());
            i += 1;
        }
    }
    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

fn print_formatted_msg(msg: &str, config: &Cookbook) {
    let mut i = 0;
    while i < msg.len() {
        if let Some(start_tag_open) = msg[i..].find('<') {
            print!("{}", &msg[i..i + start_tag_open]);
            i += start_tag_open;

            if let Some(tag_close_idx) = msg[i..].find('>') {
                let tag_name = &msg[i + 1..i + tag_close_idx];
                let content_start = i + tag_close_idx + 1;
                let close_tag = format!("</{}>", tag_name);

                if let Some(content_end_rel) = msg[content_start..].find(&close_tag) {
                    let content_end = content_start + content_end_rel;
                    let inner_text = &msg[content_start..content_end];
                    apply_style(inner_text, tag_name, config);
                    i = content_end + close_tag.len();
                } else {
                    print!("<");
                    i += 1;
                }
            } else {
                print!("<");
                i += 1;
            }
        } else {
            print!("{}", &msg[i..]);
            break;
        }
    }
}

fn apply_style(text: &str, style: &str, config: &Cookbook) {
    if style == "bold" {
        print!("{}", text.bold());
    } else if let Some(hex) = config.theme.colors.get(style) {
        let color = ColorResolver::hex_to_color(hex);
        print!("{}", text.custom_color(color));
    } else {
        print!("{}", text);
    }
}

pub fn log_to_file(
    config: &Cookbook,
    level: &str,
    scope: &str,
    msg: &str,
    app_override: Option<&str>,
) -> Result<()> {
    let clean_msg = strip_tags(msg);
    let now = Local::now();
    let tag = TagFactory::create_tag(config, level);
    let timestamp = now
        .format(&config.layout.logging.timestamp_format)
        .to_string();

    let app_name = app_override.unwrap_or(&config.layout.logging.app_name);

    let mut content = config.layout.structure.file.clone();
    content = content.replace("{timestamp}", &timestamp);
    content = content.replace("{tag}", &tag);
    content = content.replace("{msg}", &clean_msg);
    content = content.replace("{scope}", scope);

    let base_dir_str = &config.layout.logging.base_dir;
    let base_dir = if base_dir_str.starts_with("~") {
        let home = directories::UserDirs::new().context("Could not find home dir")?;
        PathBuf::from(base_dir_str.replace("~", home.home_dir().to_str().unwrap()))
    } else {
        PathBuf::from(base_dir_str)
    };

    let year = now.format("%Y").to_string();
    let month = now.format("%m").to_string();
    let day = now.format("%d").to_string();

    let mut rel_path = config.layout.logging.path_structure.clone();
    rel_path = rel_path.replace("{year}", &year);
    rel_path = rel_path.replace("{month}", &month);
    rel_path = rel_path.replace("{scope}", scope);
    rel_path = rel_path.replace("{app}", app_name);

    let mut filename = config.layout.logging.filename_structure.clone();
    filename = filename.replace("{level}", level);
    filename = filename.replace("{year}", &year);
    filename = filename.replace("{month}", &month);
    filename = filename.replace("{day}", &day);
    filename = filename.replace("{app}", app_name);

    let full_dir = base_dir.join(rel_path);
    fs::create_dir_all(&full_dir)?;

    let file_path = full_dir.join(filename);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;

    writeln!(file, "{}", content)?;

    Ok(())
}

fn strip_tags(msg: &str) -> String {
    let mut result = String::new();
    let mut i = 0;
    while i < msg.len() {
        if let Some(start) = msg[i..].find('<') {
            result.push_str(&msg[i..i + start]);
            i += start;
            if let Some(end) = msg[i..].find('>') {
                i += end + 1;
            } else {
                result.push('<');
                i += 1;
            }
        } else {
            result.push_str(&msg[i..]);
            break;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        DictionaryConfig, IconsConfig, LayoutConfig, LoggingConfig, StructureConfig, TagConfig,
        ThemeConfig, ThemeMeta, ThemeSettings,
    };
    use std::collections::HashMap;
    use tempfile::tempdir;

    fn create_mock_config() -> Cookbook {
        Cookbook {
            theme: ThemeConfig {
                meta: ThemeMeta {
                    name: "Test".to_string(),
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
                    transform: "uppercase".to_string(),
                    min_width: 10,
                    alignment: "center".to_string(),
                },
                labels: HashMap::new(),
                structure: StructureConfig {
                    terminal: "".to_string(),
                    file: "{msg}".to_string(),
                },
                logging: LoggingConfig {
                    base_dir: "".to_string(), // will be set
                    path_structure: "{app}/{scope}".to_string(),
                    filename_structure: "log.txt".to_string(),
                    timestamp_format: "%Y".to_string(),
                    write_by_default: true,
                    app_name: "default_app".to_string(),
                },
                include: None,
            },
            dictionary: DictionaryConfig {
                presets: HashMap::new(),
                include: None,
            },
        }
    }

    #[test]
    fn test_log_to_file_default_app() {
        let dir = tempdir().unwrap();
        let mut config = create_mock_config();
        config.layout.logging.base_dir = dir.path().to_str().unwrap().to_string();

        log_to_file(&config, "info", "MAIN", "test message", None).unwrap();

        let expected_path = dir.path().join("default_app/MAIN/log.txt");
        assert!(expected_path.exists());

        let content = fs::read_to_string(expected_path).unwrap();
        assert!(content.contains("test message"));
    }

    #[test]
    fn test_log_to_file_app_override() {
        let dir = tempdir().unwrap();
        let mut config = create_mock_config();
        config.layout.logging.base_dir = dir.path().to_str().unwrap().to_string();

        log_to_file(
            &config,
            "info",
            "MAIN",
            "test message",
            Some("OverriddenApp"),
        )
        .unwrap();

        let expected_path = dir.path().join("OverriddenApp/MAIN/log.txt");
        assert!(expected_path.exists());
    }
}
