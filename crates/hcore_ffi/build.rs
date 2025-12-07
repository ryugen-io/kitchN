extern crate cbindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // Output to project root include/ directory
    let mut output_file = PathBuf::from(&crate_dir);
    output_file.pop(); // crates/
    output_file.pop(); // root
    output_file.push("include");
    output_file.push("hcore.h");

    if let Some(parent) = output_file.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_config(cbindgen::Config::from_file("cbindgen.toml").unwrap_or_default())
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(output_file);
}
