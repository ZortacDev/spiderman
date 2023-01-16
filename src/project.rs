use std::collections::HashMap;
use std::fs::{DirEntry, File, ReadDir};
use std::io::{BufRead, BufReader, Write};
use std::iter::FilterMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use anyhow::{anyhow, Result};
use uuid::fmt::Hyphenated;
use crate::Environment;
use crate::file_utils::open_in_editor;

#[derive(Debug, Clone)]
pub struct Project {
    pub uuid: Uuid,
    pub name: String,
    pub tags: HashMap<String, String>
}

const SPIDERMAN_PROJECT_INFO_FILE_NAME: &'static str = "spiderman.tags";

impl Project {
    pub fn new(name: &str) -> Result<Self> {
        let uuid = Uuid::new_v4();

        let env = Environment::get();
        let mut path = env.raw_storage_dir.clone();
        path.push(uuid.hyphenated().to_string());
        std::fs::create_dir(&path)?;
        path.push(name);
        std::fs::create_dir(&path)?;
        path.pop();
        path.push(SPIDERMAN_PROJECT_INFO_FILE_NAME);
        File::create(&path)?;
        // TODO: fill with tags extracted from current directory, if we're in a view
        open_in_editor(&path);
        let tags = Self::read_tags(&path)?;

        Ok(Self {
            uuid,
            name: name.to_owned(),
            tags
        })
    }

    pub fn open(path: &Path) -> Result<Self> {
        let uuid_str = path.file_name().ok_or(anyhow!("Invalid project path"))?.to_string_lossy();
        let uuid = Uuid::parse_str(uuid_str.as_ref())?;

        let mut dir = path.to_path_buf();
        dir.push(SPIDERMAN_PROJECT_INFO_FILE_NAME);
        return if dir.metadata().is_ok_and(|m| m.is_file()) {
            let tags = Self::read_tags(&dir)?;

            dir.pop();
            let directory_contents: Vec<_> = dir.read_dir()?
                .filter_map(|e| e.ok())
                .filter(|d| d.file_name().to_string_lossy() != SPIDERMAN_PROJECT_INFO_FILE_NAME)
                .collect();

            if directory_contents.len() == 1 {
                let name = directory_contents[0].file_name().to_string_lossy().to_string();
                Ok(Self {
                    uuid,
                    name,
                    tags
                })
            } else {
                Err(anyhow!("More than one subdirectory in project UUID directory"))
            }
        } else {
            Err(anyhow!("No spiderman tags file in UUID directory: {}", dir.to_string_lossy()))
        }
    }

    fn read_tags(path: &Path) -> Result<HashMap<String, String>> {
        // TODO: Allow multiple values for any tag
        let tags_file = File::open(&path)?;
        let tags_reader = BufReader::new(tags_file);
        let mut tags = HashMap::new();
        for line in tags_reader.lines() {
            if let Some((tag, value)) = line?.as_str().split_once(':') {
                tags.insert(tag.to_string(), value.trim().to_string());
            }
        }

        return Ok(tags);
    }

    pub fn list() -> Result<ProjectIterator> {
        let map_to_option = Box::new(|e: std::io::Result<DirEntry>| e.ok())
            as Box<dyn Fn(std::io::Result<DirEntry>) -> Option<DirEntry>>;
        let map_to_project = Box::new(|d: DirEntry| {
            let path = d.path();
            match Self::open(&path) {
                Ok(p) => {Some(p)}
                Err(e) => {
                    eprintln!("WARNING: Ignoring project at {}: {}", path.to_string_lossy(), e);
                    None
                }
            }
        }) as Box<dyn Fn(DirEntry) -> Option<Project>>;

        let env = Environment::get();
        Ok(env.raw_storage_dir.read_dir()?
            .filter_map(map_to_option)
            .filter_map(map_to_project))
    }

    pub fn get_project_raw_data_path(&self) -> PathBuf {
        let env = Environment::get();
        let mut path = env.raw_storage_dir.clone();

        path.push(self.uuid.hyphenated().to_string());
        path.push(&self.name);

        return path;
    }
}

pub type ProjectIterator = FilterMap<FilterMap<ReadDir, Box<dyn Fn(std::io::Result<DirEntry>) -> Option<DirEntry>>>, Box<dyn Fn(DirEntry) -> Option<Project>>>;