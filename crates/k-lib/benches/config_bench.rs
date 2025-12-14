use criterion::{Criterion, black_box, criterion_group, criterion_main};
use k_lib::config::Cookbook;
use std::fs;
use tempfile::tempdir;

fn benchmark_config_load(c: &mut Criterion) {
    // Setup a dummy config directory
    let dir = tempdir().unwrap();
    let config_dir = dir.path();

    fs::write(
        config_dir.join("theme.toml"),
        r##"
    [meta]
    name = "bench_theme"
    [settings]
    active_icons = "ascii"
    [colors]
    bg = "#282a36"
    fg = "#f8f8f2"
    primary = "#ff79c6"
    [fonts]
    ui = "Sans"
    "##,
    )
    .unwrap();

    fs::write(
        config_dir.join("icons.toml"),
        r##"
    [nerdfont]
    [ascii]
    "##,
    )
    .unwrap();

    fs::write(
        config_dir.join("layout.toml"),
        r##"
    [tag]
    prefix = "["
    suffix = "]"
    transform = "uppercase"
    min_width = 10
    alignment = "left"
    [labels]
    [structure]
    terminal = "{tag} {msg}"
    file = "{tag} {msg}"
    [logging]
    base_dir = "logs"
    path_structure = "{app}.log"
    filename_structure = "log"
    timestamp_format = "%Y"
    write_by_default = false
    app_name = "benchmark"
    "##,
    )
    .unwrap();

    c.bench_function("config_load_from_dir", |b| {
        b.iter(|| {
            let _ = Cookbook::load_from_dir(black_box(config_dir));
        })
    });
}

fn benchmark_config_serialization(c: &mut Criterion) {
    // We can't use default() as it's not derived.
    // Let's create a dummy one by loading from an empty dir (or our dummy dir)
    let dir = tempdir().unwrap();
    let config_dir = dir.path();
    fs::create_dir_all(config_dir).unwrap();

    // Mock files to make it valid
    fs::write(
        config_dir.join("theme.toml"),
        r##"
    [meta]
    name = "bench_theme"
    [settings]
    active_icons = "ascii"
    [colors]
    bg = "#282a36"
    fg = "#f8f8f2"
    primary = "#ff79c6"
    [fonts]
    ui = "Sans"
    "##,
    )
    .unwrap();

    fs::write(
        config_dir.join("icons.toml"),
        r##"
    [nerdfont]
    [ascii]
    "##,
    )
    .unwrap();

    fs::write(
        config_dir.join("layout.toml"),
        r##"
    [tag]
    prefix = "["
    suffix = "]"
    transform = "uppercase"
    min_width = 10
    alignment = "left"
    [labels]
    [structure]
    terminal = "{tag} {msg}"
    file = "{tag} {msg}"
    [logging]
    base_dir = "logs"
    path_structure = "{app}.log"
    filename_structure = "log"
    timestamp_format = "%Y"
    write_by_default = false
    app_name = "benchmark"
    "##,
    )
    .unwrap();
    fs::write(
        config_dir.join("dictionary.toml"),
        r##"
    [presets.bench_ok]
    level = "success"
    msg = "benchmark passed"
    "##,
    )
    .unwrap();

    let config = Cookbook::load_from_dir(config_dir).expect("Failed to create dummy config");

    let bin_path = dir.path().join("bench.bin");

    c.bench_function("config_save_binary", |b| {
        b.iter(|| {
            let _ = config.save_binary(black_box(&bin_path));
        })
    });
}

criterion_group!(
    benches,
    benchmark_config_load,
    benchmark_config_serialization
);
criterion_main!(benches);
