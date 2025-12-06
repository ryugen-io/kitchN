use anyhow::{Context, Result};
use chrono::Local;
use clap::Parser;
use colored::Colorize;
use core_lib::config::HyprConfig;
use core_lib::factory::{ColorResolver, TagFactory};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "corelog", version, about = "Hyprcore Logging Tool")]
struct Cli {
    /// Preset key from dictionary.toml
    preset: String,

    /// Optional message override
    #[arg(trailing_var_arg = true)]
    msg: Option<Vec<String>>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = HyprConfig::load().context("Failed to load Hyprcore config")?;

    let preset = config.dictionary.presets.get(&cli.preset).context(format!(
        "Preset '{}' not found in dictionary.toml",
        cli.preset
    ))?;

    let level = &preset.level;
    let scope = preset
        .scope
        .as_ref()
        .context("Preset missing 'scope' field")?;
    
    // Join override words or use preset msg
    let msg_string;
    let msg = if let Some(args) = &cli.msg {
        msg_string = args.join(" ");
        &msg_string
    } else {
        &preset.msg
    };

    log_to_terminal(&config, level, scope, msg);

    if config.layout.logging.write_by_default {
        log_to_file(&config, level, scope, msg)?;
    }

    Ok(())
}

fn log_to_terminal(config: &HyprConfig, level: &str, scope: &str, msg: &str) {
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

    // Parse structure template
    // We assume structure is "{tag} {scope} {icon} {msg}" or similar.
    // We split by space to simple tokenize? No, better replace placeholders and then split?
    // Actually, to support granular coloring, we should parse the *structure string*.
    // But for a robust CLI tool, let's keep it simple: we reconstruct assuming the standard layout parts.
    // If the user completely changes structure to "{msg} {tag}", our hardcoded print order fails.
    // Ideally, we'd regex parse the structure string.

    // For now, let's process the structure string and replace parts with colored versions,
    // *except* matching {msg}, which we handle specially.

    let structure = &config.layout.structure.terminal;

    // Naive Tokenization: split chunks by valid placeholders
    let parts = parse_structure(structure);

    for part in parts {
        match part.as_str() {
            "{tag}" => print!("{}", tag.custom_color(level_color)),
            "{icon}" => print!("{}", icon.custom_color(level_color)),
            "{scope}" => print!("{}", scope.white().dimmed()), // Scope dimmed white
            "{msg}" => print_formatted_msg(msg, config),
            _ => print!("{}", part), // Literal text/spaces
        }
    }
    println!(); // End line
}

fn parse_structure(structure: &str) -> Vec<String> {
    // Simple splitter that keeps delimiters.
    // This is a minimal implementation.
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

fn print_formatted_msg(msg: &str, config: &HyprConfig) {
    // Very simple XML-like parser: <bold>...</bold>, <color>...</color>
    // Supports nested tags via recursion? Or flat? Flat is easier and likely enough.
    // Let's do a simple scan.

    let mut i = 0;
    while i < msg.len() {
        if let Some(start_tag_open) = msg[i..].find('<') {
            // Print text before tag
            print!("{}", &msg[i..i + start_tag_open]);
            i += start_tag_open;

            if let Some(tag_close_idx) = msg[i..].find('>') {
                let tag_name = &msg[i + 1..i + tag_close_idx];
                let content_start = i + tag_close_idx + 1;

                // Find closing tag </tag_name>
                let close_tag = format!("</{}>", tag_name);

                if let Some(content_end_rel) = msg[content_start..].find(&close_tag) {
                    let content_end = content_start + content_end_rel;
                    let inner_text = &msg[content_start..content_end];

                    // Apply style
                    apply_style(inner_text, tag_name, config);

                    i = content_end + close_tag.len();
                } else {
                    // No closing tag found, treat opening bracket as literal
                    print!("<");
                    i += 1;
                }
            } else {
                print!("<");
                i += 1;
            }
        } else {
            // No more tags
            print!("{}", &msg[i..]);
            break;
        }
    }
}

fn apply_style(text: &str, style: &str, config: &HyprConfig) {
    if style == "bold" {
        print!("{}", text.bold());
    } else {
        // Check if style matches a color name in theme
        if let Some(hex) = config.theme.colors.get(style) {
            let color = ColorResolver::hex_to_color(hex);
            print!("{}", text.custom_color(color));
        } else {
             // If not in theme, print plain text.
             // This ensures we strictly follow the theme palette.
             print!("{}", text);
        }
    }
}

fn log_to_file(config: &HyprConfig, level: &str, scope: &str, msg: &str) -> Result<()> {
    // Strip tags for file log
    let clean_msg = strip_tags(msg);

    let now = Local::now();

    let tag = TagFactory::create_tag(config, level);
    let timestamp = now
        .format(&config.layout.logging.timestamp_format)
        .to_string();

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

    let mut filename = config.layout.logging.filename_structure.clone();
    filename = filename.replace("{level}", level);
    filename = filename.replace("{year}", &year);
    filename = filename.replace("{month}", &month);
    filename = filename.replace("{day}", &day);

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
                // Check if it's a closing tag or content
                // A simplistic strip: just remove anything between < >
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
