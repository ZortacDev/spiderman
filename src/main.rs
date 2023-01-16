#![feature(absolute_path)]
#![feature(is_some_and)]

mod config;
mod environment;
mod file_utils;
mod project;
mod schema;
mod weave;

use std::path::{Path, PathBuf};

use crate::environment::Environment;
use crate::file_utils::open_in_editor;
use crate::project::Project;
use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use fs_extra::dir::{move_dir, CopyOptions};

#[derive(Parser)]
#[command(author, version)]
/// The Weaving Project Manager
///
/// Spiderman manages your projects by associating each project with a set of _tags_ and dynamically generating
/// set of _views_ based on these tags.
///
/// All projects live within a spiderman project root. All subcommands (except **init** which creates a new root)
/// operate with respect to the _current_ project root. The project root is either the next upstream
/// directory (from the current working directory) that qualifies as a project root, or the default project root,
/// which can be set in `~/.config/spiderman/config.toml`.
///
/// A spiderman project root contains a `.spiderman` directory, which, in turn, contains the `raw` directory,
/// holding all the projects managed by spiderman, and the `schema.toml` configuration file, describing how
/// spiderman should build view trees based on tags.
///
/// The `schema.toml` file contains a list of schemas and a set of default tag values to use when a
/// project does not have a tag, but that tag is used in a schema. Schemas consist of `/` separated components
/// that are either plain strings, in which case they will be used in the view tree verbatim, or strings contained
/// in curly braces (`{` and `}`) in which case the string is interpreted as the name of a tag and substituted
/// with the appropriate tag value when building the view tree.
///
/// A project's tags are specified in its `spiderman.tags` file, which can be edited using the **tags** subcommand.
/// Each line in this file consists of colon (`:`) separated values. The first of these is the name of the tag, while the
/// later ones are values for that tag. A project may have multiple values for one tag.
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Re-creates the view tree
    Weave,
    /// Initializes a new spiderman project root
    Init {
        /// Directory to use instead of the current directory as the project root
        dir: Option<PathBuf>,
    },
    /// Creates a new project
    New {
        /// Name of the project to be created
        name: String,
    },
    /// Moves an existing project into the current spiderman project root
    Move {
        /// Path to the project to be moved, must be a directory
        source: PathBuf,
    },
    /// Edit tags of the current project (the current working directory must be a project directory)
    Tags,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Weave => {
            weave().context("Failed to weave")?;
        }
        Commands::Init { dir } => {
            init(dir).context("Failed to initialize")?;
        }
        Commands::New { name } => {
            new(name).context("Failed to create project")?;
        }
        Commands::Move { source } => {
            move_project(source).context("Failed to move project")?;
        }
        Commands::Tags => {
            tags().context("Failed to edit tags")?;
        }
    }

    Ok(())
}

fn weave() -> Result<()> {
    weave::remove_symlinks().expect("Failed to remove symlinks in view tree");
    weave::remove_empty_directories().expect("Failed to remove empty directories in view tree");
    weave::construct_view_tree().expect("Failed to construct view tree");

    Ok(())
}

fn init(dir: &Option<PathBuf>) -> Result<()> {
    let path = dir.clone().unwrap_or(std::env::current_dir()?);
    if !path.read_dir().is_ok_and(|mut d| d.next().is_none()) {
        // directory is not empty
        return Err(anyhow!(
            "Chosen base path {} is not empty!",
            path.to_string_lossy()
        ));
    }

    Environment::create_spiderman_dir(&path)?;

    Ok(())
}

fn new(name: &str) -> Result<()> {
    Project::new(name)?;

    weave()?;
    Ok(())
}

fn move_project(source: &Path) -> Result<()> {
    if source.exists() && source.is_dir() {
        let project_name = std::path::absolute(source)?
            .file_name()
            .ok_or(anyhow!("Source directory has no name"))?
            .to_string_lossy()
            .into_owned();
        let project = Project::new(project_name.as_ref())?;
        let new_path = project.get_project_raw_data_path()?;
        let options = CopyOptions::new();
        move_dir(source, new_path, &options)?;
        weave()?;
        Ok(())
    } else {
        Err(anyhow!(
            "{} does not exist or is not a directory",
            source.to_string_lossy()
        ))
    }
}

fn tags() -> Result<()> {
    let current_project = Project::get_current_project()?.ok_or(anyhow!(
        "Not in a project directory (or subdirectory thereof)!"
    ))?;

    let tags_file = current_project.get_tags_file_path()?;

    if open_in_editor(&tags_file)? {
        weave()?;
    }

    Ok(())
}
