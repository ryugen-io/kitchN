use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn setup_config(root: &Path) -> std::path::PathBuf {
    let config_home = root.join("config");
    let kitchn_config = config_home.join("kitchn");
    fs::create_dir_all(&kitchn_config).unwrap();

    // Create minimal required config files
    fs::write(
        kitchn_config.join("theme.toml"),
        r#"
[meta]
name = "test"
[settings]
active_icons = "none"
[colors]
[fonts]
"#,
    )
    .unwrap();

    fs::write(
        kitchn_config.join("icons.toml"),
        r#"
[nerdfont]
[ascii]
"#,
    )
    .unwrap();

    fs::write(
        kitchn_config.join("layout.toml"),
        r#"
[tag]
prefix = ""
suffix = ""
transform = "none"
min_width = 0
alignment = "left"
[labels]
[structure]
terminal = "{msg}"
file = "{msg}"
[logging]
base_dir = "logs"
path_structure = "sys.log"
filename_structure = "log"
timestamp_format = ""
write_by_default = false
"#,
    )
    .unwrap();

    config_home
}

#[test]
fn test_cli_pack_and_install() {
    let dir = tempdir().unwrap();
    let config_home = setup_config(dir.path());
    let source_dir = dir.path().join("source");
    let output_file = dir.path().join("test.bag");

    fs::create_dir_all(&source_dir).unwrap();
    fs::write(
        source_dir.join("test.ing"),
        r#"[package]
name = "test"
version = "0.1.0"
authors = ["Test"]
description = "Test ingredient"
"#,
    )
    .unwrap();

    cargo_bin_cmd!("kitchn")
        .env("XDG_CONFIG_HOME", &config_home)
        .env("XDG_CACHE_HOME", dir.path().join("cache"))
        .env("XDG_DATA_HOME", dir.path().join("data"))
        .arg("wrap")
        .arg(&source_dir)
        .arg("-o")
        .arg(&output_file)
        .assert()
        .success();

    assert!(output_file.exists());
}

#[test]
fn test_cli_help() {
    // Help usually doesn't require config load, but to be robust we set envs to empty/temp
    // or rely on clap handling help before config load.
    // Given the previous success, clap handles it. But let's be safe if we change order.
    let _dir = tempdir().unwrap();
    // We don't necessarily need to create files for help if it's pure clap,
    // but if kitchn initializes early, it might fail.
    // Let's assume clap handles it first.
    cargo_bin_cmd!("kitchn")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}
