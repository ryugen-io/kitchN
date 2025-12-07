use crate::fragment::Fragment;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct FragmentsDb {
    path: PathBuf,
    fragments: HashMap<String, Fragment>,
}

impl FragmentsDb {
    pub fn load(path: &Path) -> Result<Self> {
        let mut db = FragmentsDb {
            path: path.to_path_buf(),
            fragments: HashMap::new(),
        };

        if path.exists() {
            let file = File::open(path).context("Failed to open fragments database")?;
            let mut reader = BufReader::new(file);
            
            // Using bincode 2.0 serde integration
            let data: HashMap<String, Fragment> = bincode::serde::decode_from_std_read(
                &mut reader, 
                bincode::config::standard()
            ).context("Failed to decode fragments database")?;
            
            db.fragments = data;
        }
        Ok(db)
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = File::create(&self.path).context("Failed to create fragments database file")?;
        let mut writer = BufWriter::new(file);
        
        bincode::serde::encode_into_std_write(
            &self.fragments,
            &mut writer,
            bincode::config::standard()
        ).context("Failed to encode fragments database")?;
        
        Ok(())
    }

    pub fn install(&mut self, fragment: Fragment) -> Result<()> {
        // Validation could happen here
        self.fragments.insert(fragment.meta.name.clone(), fragment);
        Ok(())
    }
    
    pub fn remove(&mut self, name: &str) -> Option<Fragment> {
        self.fragments.remove(name)
    }

    pub fn list(&self) -> Vec<&Fragment> {
        let mut list: Vec<&Fragment> = self.fragments.values().collect();
        list.sort_by_key(|f| &f.meta.name);
        list
    }
    
    pub fn iter(&self) -> std::collections::hash_map::Values<'_, String, Fragment> {
        self.fragments.values()
    }
}
