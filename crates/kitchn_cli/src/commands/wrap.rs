use crate::logging::log_msg;
use anyhow::Result;
use k_lib::config::Cookbook;
use k_lib::packager;
use std::path::PathBuf;

pub fn execute(input: PathBuf, output: Option<PathBuf>, config: &Cookbook) -> Result<()> {
    let out = output.unwrap_or_else(|| {
        let name = input.file_name().unwrap_or_default().to_string_lossy();
        PathBuf::from(format!("{}.bag", name))
    });
    packager::pack(&input, &out)?;
    log_msg(
        config,
        "wrap_ok",
        &format!("wrapped package to {}", out.display()),
    );
    Ok(())
}
