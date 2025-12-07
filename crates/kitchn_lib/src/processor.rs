use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tera::{Context as TeraContext, Tera};

use crate::config::Cookbook;
use crate::ingredient::Ingredient;
use crate::logger;

use std::collections::HashMap;
use tera::{Value, to_value, try_get_value};

pub fn apply(ingredient: &Ingredient, config: &Cookbook) -> Result<()> {
    let mut tera = Tera::default();
    tera.register_filter("hex_to_rgb", hex_to_rgb);

    let mut ctx = TeraContext::new();

    // Context Setup
    ctx.insert("colors", &config.theme.colors);
    ctx.insert("fonts", &config.theme.fonts);

    let active_icons = if config.theme.settings.active_icons == "nerdfont" {
        &config.icons.nerdfont
    } else {
        &config.icons.ascii
    };
    ctx.insert("icons", active_icons);

    process_ingredient(ingredient, &mut tera, &mut ctx, config)
}

/// Tera filter: hex_to_rgb
fn hex_to_rgb(value: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let s = try_get_value!("hex_to_rgb", "value", String, value);
    let hex = s.trim_start_matches('#');

    if hex.len() != 6 {
        return Err(tera::Error::msg(format!("Invalid hex color: {}", s)));
    }

    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| tera::Error::msg("Invalid hex"))?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| tera::Error::msg("Invalid hex"))?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| tera::Error::msg("Invalid hex"))?;

    Ok(to_value(vec![r, g, b]).unwrap())
}

fn process_ingredient(
    pkg: &Ingredient,
    tera: &mut Tera,
    ctx: &mut TeraContext,
    config: &Cookbook,
) -> Result<()> {
    // Render Templates
    for tpl in &pkg.templates {
        render_and_write(&tpl.target, &tpl.content, tera, ctx)?;
    }

    // Render Files
    for file in &pkg.files {
        render_and_write(&file.target, &file.content, tera, ctx)?;
    }

    // Run Hooks
    if let Some(cmd) = &pkg.hooks.reload {
        logger::log_to_terminal(config, "info", "HOOK", "running hooks");

        let output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .context("Failed to execute hook")?;

        if !output.stdout.is_empty() {
            let s = String::from_utf8_lossy(&output.stdout);
            for line in s.lines() {
                logger::log_to_terminal(config, "info", "HOOK", line);
            }
        }

        if !output.stderr.is_empty() {
            let s = String::from_utf8_lossy(&output.stderr);
            for line in s.lines() {
                logger::log_to_terminal(config, "error", "HOOK", line);
            }
        }

        if output.status.success() {
            logger::log_to_terminal(config, "success", "HOOK", "hooks executed");
        } else {
            logger::log_to_terminal(config, "error", "HOOK", "hooks failed");
        }
    }

    Ok(())
}

fn render_and_write(target: &str, content: &str, tera: &mut Tera, ctx: &TeraContext) -> Result<()> {
    // Basic expansion of ~
    let target_expanded = if target.starts_with("~") {
        let home = directories::UserDirs::new()
            .context("Could not determine home directory")?
            .home_dir()
            .to_string_lossy()
            .to_string();
        target.replace("~", &home)
    } else {
        target.to_string()
    };

    let path = PathBuf::from(&target_expanded);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Render content
    // We create a one-off template due to dynamic content
    let rendered = tera
        .render_str(content, ctx)
        .context("Failed to render template")?;

    fs::write(path, rendered)?;
    Ok(())
}
