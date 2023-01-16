use crate::file_utils::create_or_open_file_with_dirs;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub default_project_dir: PathBuf,
}

const CONFIG_PATH: &'static str = "~/.config/spiderman/config.toml";
const DEFAULT_PROJECT_DIR: &'static str = "~/spiderman_projects";

impl Default for Config {
    fn default() -> Self {
        Self {
            default_project_dir: Path::new(
                &shellexpand::full(DEFAULT_PROJECT_DIR).unwrap().to_string(),
            )
            .to_path_buf(),
        }
    }
}

impl Config {
    fn get_expanded_config_path() -> Result<PathBuf> {
        let expanded_path = shellexpand::full(CONFIG_PATH)?.to_string();
        Ok(Path::new(&expanded_path).to_owned())
    }

    pub fn load() -> Result<Self> {
        let config_path = Self::get_expanded_config_path()?;

        let mut config_file = create_or_open_file_with_dirs(&config_path, || {
            toml::ser::to_vec(&Self::default()).unwrap()
        })?;

        let mut config_contents = String::new();
        config_file.read_to_string(&mut config_contents)?;
        Ok(toml::de::from_str(&config_contents)?)
    }
}
