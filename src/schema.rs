use crate::schema::SchemaPathComponent::Fixed;
use crate::{Environment, Project};
use anyhow::Result;
use itertools::Itertools;
use serde::de::Visitor;
use serde::{ser, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt::Formatter;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Schemas {
    pub(crate) schemas: Vec<Schema>,
    default_tag_values: HashMap<String, String>,
}

impl Schemas {
    pub fn fill(&self, project: &Project) -> Result<Vec<PathBuf>> {
        let mut error = None;
        let filled_schemas = self
            .schemas
            .iter()
            .filter_map(|s| {
                let filled_schemas = s.fill(project, &self.default_tag_values);
                match filled_schemas {
                    Ok(s) => Some(s),
                    Err(e) => {
                        error = Some(e);
                        None
                    }
                }
            })
            .flatten()
            .collect();

        match error {
            None => Ok(filled_schemas),
            Some(e) => Err(e),
        }
    }
}

#[derive(Debug)]
pub struct Schema {
    components: Vec<SchemaPathComponent>,
}

impl Serialize for Schema {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(string_repr) = self
            .components
            .iter()
            .map(|c| match c {
                SchemaPathComponent::Tag(t) => {
                    format!("{{{}}}", t)
                }
                Fixed(f) => f.to_string(),
            })
            .intersperse("/".to_string())
            .reduce(|s1, s2| s1 + &s2)
        {
            serializer.serialize_str(&string_repr)
        } else {
            Err(ser::Error::custom(
                "empty schema has no valid string representation",
            ))
        }
    }
}

impl<'de> Deserialize<'de> for Schema {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(SchemaVisitor)
    }
}

struct SchemaVisitor;

impl<'de> Visitor<'de> for SchemaVisitor {
    type Value = Schema;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a schema string")
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Schema::from(v))
    }
}

impl From<&str> for Schema {
    fn from(mut value: &str) -> Self {
        if value.starts_with('/') {
            value = &value[1..];
        }

        if value.ends_with('/') {
            value = &value[..value.len() - 1];
        }

        let components: Vec<SchemaPathComponent> = value
            .split('/')
            .map(|s| {
                if s.starts_with('{') && s.ends_with('}') {
                    SchemaPathComponent::Tag(s[1..s.len() - 1].to_string())
                } else {
                    SchemaPathComponent::Fixed(s.to_string())
                }
            })
            .collect();

        Self { components }
    }
}

impl Schema {
    fn fill(
        &self,
        project: &Project,
        default_tags: &HashMap<String, String>,
    ) -> Result<Vec<PathBuf>> {
        let mut paths = vec![];

        // Get all possible combinations of tags
        let tag_combinations = project
            .tags
            .iter()
            .map(|(tag, values)| values.iter().map(move |v| (tag, v)))
            .multi_cartesian_product()
            .map(|p| p.into_iter().collect::<HashMap<&String, &String>>());

        // Resolve path for each of these combinations and push the result to paths
        for tags in tag_combinations {
            let mut path = Environment::get()?.base_path.clone();
            for component in &self.components {
                match component {
                    SchemaPathComponent::Tag(tag) => {
                        let value: &str = tags.get(tag).map(|s| s.as_str())
                            .unwrap_or_else(|| default_tags.get(tag).map(|s| s.as_str()).unwrap_or_else(|| {
                                eprintln!("WARNING: Could not evaluate tag {} for project {:?} with default tags {:?}", tag, project, default_tags);
                                "unknown"
                            }));

                        path.push(value);
                    }
                    SchemaPathComponent::Fixed(fixed_part) => path.push(fixed_part),
                }
            }

            path.push(&project.name);
            paths.push(path);
        }

        if paths.is_empty() {
            eprintln!(
                "WARNING: The project with UUID {} has no tags and could not be linked anywhere.",
                project.uuid.hyphenated().to_string()
            );
        }

        return Ok(paths);
    }

    pub fn match_with_dir(&self, path: &Path) -> Result<Option<HashMap<String, String>>> {
        let env = Environment::get()?;
        let path = std::path::absolute(path)?;
        let mut tags = HashMap::new();
        if let Ok(path) = path.strip_prefix(env.base_path.clone()) {
            if path.components().zip(self.components.iter()).fold(
                true,
                |matches, (path, schema_component)| match schema_component {
                    SchemaPathComponent::Tag(tag_name) => {
                        tags.insert(
                            tag_name.clone(),
                            path.as_os_str().to_string_lossy().to_string(),
                        );
                        matches
                    }
                    Fixed(name) => {
                        matches && (name.as_str() == path.as_os_str().to_string_lossy().as_ref())
                    }
                },
            ) {
                return Ok(Some(tags));
            }
        }

        Ok(None)
    }
}

#[derive(Debug)]
pub enum SchemaPathComponent {
    Tag(String),
    Fixed(String),
}
