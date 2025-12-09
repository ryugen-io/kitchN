use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_cli_pack_and_install() {
    let dir = tempdir().unwrap();
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
    cargo_bin_cmd!("kitchn")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}
