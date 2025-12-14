use anyhow::{Context, Result};
use clap::Parser;
use k_lib::config::Cookbook;
use k_lib::logger;

#[derive(Parser)]
#[command(name = "kitchn-log", version, about = "Kitchn Logging Tool")]
struct Cli {
    /// Preset key from dictionary.toml
    preset: String,

    /// Optional message override
    #[arg(trailing_var_arg = true)]
    msg: Option<Vec<String>>,

    /// Optional app name override
    #[arg(long)]
    app: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Cookbook::load().context("Failed to load Kitchn config")?;

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

    logger::log_to_terminal(&config, level, scope, msg);

    if config.layout.logging.write_by_default {
        logger::log_to_file(&config, level, scope, msg, cli.app.as_deref())?;
    }

    Ok(())
}
