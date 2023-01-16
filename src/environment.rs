use std::env::current_dir;
use std::ffi::OsString;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use once_cell::sync::OnceCell;
use crate::config::Config;
use crate::schema::{SchemaFile, Schemas};
use anyhow::Result;

const SPIDERMAN_DIR_NAME: &'static str = ".spiderman";
const RAW_STORAGE_DIR_NAME: &'static str = "raw";
const SCHEMA_FILE_NAME: &'static str = "schema.toml";

pub struct Environment {
    pub base_path: PathBuf,
    pub spiderman_dir: PathBuf,
    pub raw_storage_dir: PathBuf,
    pub schema: Schemas,
    pub config: Config
}

static ENVIRONMENT: OnceCell<Environment> = OnceCell::new();

impl Environment {
    fn new() -> Self {
        let config = Config::load().expect("Unable to load or create configuration file");

        let base_path = Self::get_parent_base_dir()
            .expect("Could not check if a parent directory is a spiderman base directory")
            .unwrap_or(config.default_project_dir.clone());

        if !Self::is_valid_spiderman_dir(&base_path) {
            if !base_path.exists() {
                std::fs::create_dir_all(base_path.clone()).expect("Failed to create project directory");
            } else {
                if !base_path.read_dir().is_ok_and(|mut d| d.next().is_none()) {
                    // directory is not empty
                    panic!("Chosen base path {} is not empty!", base_path.to_string_lossy());
                }
            }
            Self::create_spiderman_dir(&base_path);
        }

        let spiderman_dir = {let mut d = base_path.clone(); d.push(SPIDERMAN_DIR_NAME); d};
        let raw_storage_dir = {let mut d = spiderman_dir.clone(); d.push(RAW_STORAGE_DIR_NAME); d};
        let schema_file_path = {let mut d = spiderman_dir.clone(); d.push(SCHEMA_FILE_NAME); d};
        let mut schema_data = String::new();
        File::open(schema_file_path).expect("Failed to open schema file.").read_to_string(&mut schema_data);
        
        let schema_file: SchemaFile = toml::de::from_str(&schema_data).expect("Failed to parse schema data.");
        
        Self {
            base_path,
            spiderman_dir,
            raw_storage_dir,
            schema: schema_file.into(),
            config
        }
    }

    fn get_parent_base_dir() -> Result<Option<PathBuf>> {
        let current_dir = current_dir()?;
        assert!(current_dir.has_root());

        Ok(current_dir.ancestors().find(|d| Self::is_valid_spiderman_dir(d)).map(|p| p.to_path_buf()))
    }

    pub fn is_valid_spiderman_dir(dir: &Path) -> bool {
        let mut path = dir.to_path_buf();
        return {
            path.push(SPIDERMAN_DIR_NAME);
            path.metadata().is_ok_and(|m| m.is_dir())
        } && {
            path.push(RAW_STORAGE_DIR_NAME);
            path.metadata().is_ok_and(|m| m.is_dir())
        } && {
            path.pop();
            path.push(SCHEMA_FILE_NAME);
            path.metadata().is_ok_and(|m| m.is_file())
        };
    }

    pub fn create_spiderman_dir(dir: &Path) -> Result<()> {
        let mut path = dir.to_path_buf();
        path.push(SPIDERMAN_DIR_NAME);
        std::fs::create_dir(&path)?;
        path.push(RAW_STORAGE_DIR_NAME);
        std::fs::create_dir(&path)?;
        path.pop();
        path.push(SCHEMA_FILE_NAME);
        let mut schema_file = File::create(&path)?;
        schema_file.write_all(&toml::ser::to_vec(&SchemaFile::default()).unwrap())?;
        Ok(())
    }

    pub fn get() -> &'static Self {
        ENVIRONMENT.get_or_init(Environment::new)
    }
}

