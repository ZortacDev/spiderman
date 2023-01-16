use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::{Environment, Project};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SchemaFile {
    default_tag_values: HashMap<String, String>,
    schemas: Vec<String>
}

pub struct Schemas {
    pub default_tag_values: HashMap<String, String>,
    pub schemas: Vec<Schema>
}

impl From<SchemaFile> for Schemas {
    fn from(file: SchemaFile) -> Self {
        Self {
            default_tag_values: file.default_tag_values,
            schemas: file.schemas.iter().map(|s| s.as_str().into()).collect()
        }
    }
}

impl Schemas {
    pub fn fill(&self, project: &Project) -> Vec<PathBuf> {
        self.schemas.iter().map(|s| s.fill(project, &self.default_tag_values)).collect()
    }
}

pub struct Schema {
    components: Vec<SchemaPathComponent>
}

impl From<&str> for Schema {
    fn from(value: &str) -> Self {
        let components: Vec<SchemaPathComponent> = value.split('/').map(|s| {
            if s.starts_with('{') && s.ends_with('}') {
                SchemaPathComponent::Tag(s[1..s.len()-1].to_string())
            } else {
                SchemaPathComponent::Fixed(s.to_string())
            }
        }).collect();

        Self {
            components
        }
    }
}

impl Schema {
    fn fill(&self, project: &Project, default_tags: &HashMap<String, String>) -> PathBuf {
        let mut path = Environment::get().base_path.clone();
        for component in &self.components {
            match component {
                SchemaPathComponent::Tag(tag) => {
                    let value: &str = project.tags.get(tag).map(|s| s.as_str())
                        .unwrap_or_else(|| default_tags.get(tag).map(|s| s.as_str()).unwrap_or_else(|| {
                            eprintln!("WARNING: Could not evaluate tag {} for project {:?} with default tags {:?}", tag, project, default_tags);
                            "unknown"
                        }));

                    path.push(value);
                }
                SchemaPathComponent::Fixed(fixed_part) => {path.push(fixed_part)}
            }
        }

        path.push(&project.name);
        return path;
    }
}

pub enum SchemaPathComponent {
    Tag(String),
    Fixed(String)
}