use crate::file_utils::{current_dir_with_symlinks, open_in_editor};
use crate::Environment;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::fs::{DirEntry, File, ReadDir};
use std::io::{BufRead, BufReader, Write};
use std::iter::FilterMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Project {
    pub uuid: Uuid,
    pub name: String,
    pub tags: HashMap<String, Vec<String>>,
}

const SPIDERMAN_PROJECT_INFO_FILE_NAME: &'static str = "spiderman.tags";

impl Project {
    pub fn new(name: &str) -> Result<Self> {
        let uuid = Uuid::new_v4();

        let env = Environment::get()?;
        let mut path = env.raw_storage_dir.clone();
        path.push(uuid.hyphenated().to_string());
        std::fs::create_dir(&path)?;
        path.push(name);
        std::fs::create_dir(&path)?;
        path.pop();
        path.push(SPIDERMAN_PROJECT_INFO_FILE_NAME);
        let mut tag_file = File::create(&path)?;
        let current_dir = current_dir_with_symlinks()?;

        // If we're in a directory that matches a schema, pre-populate the tags file
        if let Some(tags) = env
            .schema
            .schemas
            .iter()
            .map(|s| s.match_with_dir(&current_dir))
            .find_map(|t| t.unwrap_or(None))
        {
            for (tag, value) in tags {
                let line = format!("{}:{}\r\n", tag, value);
                tag_file.write_all(line.as_bytes())?;
            }
        }
        tag_file.sync_data()?;
        drop(tag_file); // Close the file

        open_in_editor(&path)?;
        let tags = Self::read_tags(&path)?;

        Ok(Self {
            uuid,
            name: name.to_owned(),
            tags,
        })
    }

    pub fn open(path: &Path) -> Result<Self> {
        let uuid_str = path
            .file_name()
            .ok_or(anyhow!("Invalid project path"))?
            .to_string_lossy();
        let uuid = Uuid::parse_str(uuid_str.as_ref())?;

        let mut dir = path.to_path_buf();
        dir.push(SPIDERMAN_PROJECT_INFO_FILE_NAME);
        return if dir.metadata().is_ok_and(|m| m.is_file()) {
            let tags = Self::read_tags(&dir)?;

            dir.pop();
            let directory_contents: Vec<_> = dir
                .read_dir()?
                .filter_map(|e| e.ok())
                .filter(|d| d.file_name().to_string_lossy() != SPIDERMAN_PROJECT_INFO_FILE_NAME)
                .collect();

            if directory_contents.len() == 1 {
                let name = directory_contents[0]
                    .file_name()
                    .to_string_lossy()
                    .to_string();
                Ok(Self { uuid, name, tags })
            } else {
                Err(anyhow!(
                    "More than one subdirectory in project UUID directory"
                ))
            }
        } else {
            Err(anyhow!(
                "No spiderman tags file in UUID directory: {}",
                dir.to_string_lossy()
            ))
        };
    }

    fn read_tags(path: &Path) -> Result<HashMap<String, Vec<String>>> {
        let tags_file = File::open(&path)?;
        tags_file.sync_data()?;
        let tags_reader = BufReader::new(tags_file);
        let mut tag_map: HashMap<String, Vec<String>> = HashMap::new();
        for line in tags_reader.lines() {
            let line = line?;
            let mut tags = line.split(':');
            if let Some(tag) = tags.next() {
                let tag = tag.trim().to_string();
                if tag.is_empty() {
                    continue;
                }

                let mut values: Vec<_> = tags.map(|v| v.trim().to_string()).collect();
                if values.is_empty() {
                    continue;
                }

                if let Some(tag_vec) = tag_map.get_mut(&tag) {
                    tag_vec.append(&mut values);
                } else {
                    tag_map.insert(tag, values);
                }
            }
        }

        return Ok(tag_map);
    }

    pub fn list() -> Result<ProjectIterator> {
        let map_to_option = Box::new(|e: std::io::Result<DirEntry>| e.ok())
            as Box<dyn Fn(std::io::Result<DirEntry>) -> Option<DirEntry>>;
        let map_to_project = Box::new(|d: DirEntry| {
            let path = d.path();
            match Self::open(&path) {
                Ok(p) => Some(p),
                Err(e) => {
                    eprintln!(
                        "WARNING: Ignoring project at {}: {}",
                        path.to_string_lossy(),
                        e
                    );
                    None
                }
            }
        }) as Box<dyn Fn(DirEntry) -> Option<Project>>;

        let env = Environment::get()?;
        Ok(env
            .raw_storage_dir
            .read_dir()?
            .filter_map(map_to_option)
            .filter_map(map_to_project))
    }

    pub fn get_project_raw_data_path(&self) -> Result<PathBuf> {
        let env = Environment::get()?;
        let mut path = env.raw_storage_dir.clone();

        path.push(self.uuid.hyphenated().to_string());
        path.push(&self.name);

        return Ok(path);
    }

    pub fn get_current_project() -> Result<Option<Self>> {
        let env = Environment::get()?;
        let raw_storage_path = env.raw_storage_dir.canonicalize()?;
        let current_path = current_dir_with_symlinks()?;
        let mut project_canonical_path = None;
        for path in current_path.ancestors() {
            if path.is_symlink() {
                let canonical = path.canonicalize()?;
                if canonical.starts_with(&raw_storage_path) {
                    project_canonical_path = Some(canonical);
                }
            }
        }

        match project_canonical_path {
            None => Ok(None),
            Some(path) => Ok(Some(Self::open(
                path.parent().expect("No parent directory"),
            )?)),
        }
    }

    pub fn get_tags_file_path(&self) -> Result<PathBuf> {
        let mut tags_file = self.get_project_raw_data_path()?;
        tags_file.pop();
        tags_file.push(SPIDERMAN_PROJECT_INFO_FILE_NAME);

        if !(tags_file.exists() && tags_file.is_file()) {
            Err(anyhow!(
                "Malformed project directory, no tag description file in {}",
                tags_file.parent().unwrap().to_string_lossy()
            ))
        } else {
            Ok(tags_file)
        }
    }
}

pub type ProjectIterator = FilterMap<
    FilterMap<ReadDir, Box<dyn Fn(std::io::Result<DirEntry>) -> Option<DirEntry>>>,
    Box<dyn Fn(DirEntry) -> Option<Project>>,
>;
