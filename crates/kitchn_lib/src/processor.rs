use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tera::{Context as TeraContext, Tera};
use log::debug;

use crate::config::Cookbook;
use crate::ingredient::Ingredient;
use crate::logger;

use std::collections::HashMap;
use tera::{Value, to_value, try_get_value};

pub fn apply(ingredient: &Ingredient, config: &Cookbook) -> Result<()> {
    debug!("Applying ingredient: {}", ingredient.meta.name);
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
    debug!("Processing ingredient templates and hooks for: {}", pkg.meta.name);
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
        debug!("Found reload hook: {}", cmd);
        // Retrieve presets or fall back to defaults
        let (run_lvl, run_scope, run_msg) = config.dictionary.presets.get("hook_run")
            .map(|p| (p.level.as_str(), p.scope.as_deref().unwrap_or("HOOK"), p.msg.as_str()))
            .unwrap_or(("secondary", "HOOK", "running hooks"));

        let (ok_lvl, ok_scope, ok_msg) = config.dictionary.presets.get("hook_ok")
            .map(|p| (p.level.as_str(), p.scope.as_deref().unwrap_or("HOOK"), p.msg.as_str()))
            .unwrap_or(("success", "HOOK", "hooks executed"));
            
        let (err_lvl, err_scope, err_msg) = config.dictionary.presets.get("hook_fail")
            .map(|p| (p.level.as_str(), p.scope.as_deref().unwrap_or("HOOK"), p.msg.as_str()))
            .unwrap_or(("error", "HOOK", "hooks failed"));

        logger::log_to_terminal(config, run_lvl, run_scope, run_msg);

        let output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .context("Failed to execute hook")?;

        if !output.stdout.is_empty() {
            let s = String::from_utf8_lossy(&output.stdout);
            debug!("Hook stdout: {}", s.trim());
            for line in s.lines() {
                // For output lines, we stick with 'info' or maybe reuse run_lvl? 
                // Let's stick with 'info' to keep it distinct from the header buffer
                logger::log_to_terminal(config, "info", run_scope, line);
            }
        }

        if !output.stderr.is_empty() {
            let s = String::from_utf8_lossy(&output.stderr);
            debug!("Hook stderr: {}", s.trim());
            for line in s.lines() {
                logger::log_to_terminal(config, "error", run_scope, line);
            }
        }

        if output.status.success() {
            logger::log_to_terminal(config, ok_lvl, ok_scope, ok_msg);
        } else {
            logger::log_to_terminal(config, err_lvl, err_scope, err_msg);
        }
    }

    Ok(())
}

fn render_and_write(target: &str, content: &str, tera: &mut Tera, ctx: &TeraContext) -> Result<()> {
    debug!("Rendering target: {}", target);
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
    
    debug!("Expanded target path: {}", target_expanded);

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
