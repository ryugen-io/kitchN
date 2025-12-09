use anyhow::{Context, Result};
use log::debug;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

/// Pack all .ing files from a source directory into a .bag archive.
pub fn pack(source_dir: &Path, output_file: &Path) -> Result<()> {
    debug!("Packing {:?} into {:?}", source_dir, output_file);
    let file = File::create(output_file).context("Failed to create output file")?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default();

    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "ing") {
            let filename = path.file_name().unwrap().to_string_lossy();
            debug!("Adding file: {}", filename);
            zip.start_file(filename.as_ref(), options)?;
            let content = fs::read_to_string(&path)?;
            zip.write_all(content.as_bytes())?;
        }
    }

    zip.finish()?;
    Ok(())
}

/// Unpack a .bag archive into the target directory.
pub fn unpack(package_file: &Path, target_dir: &Path) -> Result<()> {
    debug!("Unpacking {:?} to {:?}", package_file, target_dir);
    let file = File::open(package_file).context("Failed to open package file")?;
    let mut archive = ZipArchive::new(file)?;

    fs::create_dir_all(target_dir)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = target_dir.join(file.name());
        debug!("Extracting file: {}", file.name());

        let mut content = Vec::new();
        file.read_to_end(&mut content)?;
        fs::write(outpath, content)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_pack_and_unpack() {
        let dir = tempdir().unwrap();
        let source_dir = dir.path().join("source");
        let output_file = dir.path().join("test.bag");
        let unpack_dir = dir.path().join("unpacked");

        fs::create_dir_all(&source_dir).unwrap();
        let frag_content = r#"
[package]
name = "test"
version = "0.1"
authors = ["Test"]
description = "Test frag"
"#;
        fs::write(source_dir.join("test.ing"), frag_content).unwrap();

        pack(&source_dir, &output_file).unwrap();
        assert!(output_file.exists());

        unpack(&output_file, &unpack_dir).unwrap();
        assert!(unpack_dir.join("test.ing").exists());

        let content = fs::read_to_string(unpack_dir.join("test.ing")).unwrap();
        assert!(content.contains("name = \"test\""));
    }
}
