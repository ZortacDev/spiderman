#![feature(absolute_path)]
#![feature(is_some_and)]

mod environment;
mod schema;
mod config;
mod file_utils;
mod project;
mod weave;

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use anyhow::{anyhow, Context, Result};
use fs_extra::dir::{CopyOptions, move_dir};
use crate::environment::Environment;
use crate::file_utils::open_in_editor;
use crate::project::Project;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand)]
enum Commands {
    Weave,
    Init {
        dir: Option<PathBuf>
    },
    New {
        name: String
    },
    Move {
        source: PathBuf
    },
    Tags
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Weave => {
            weave().context("Failed to weave")?;
        },
        Commands::Init {dir} => {
            init(dir).context("Failed to initialize")?;
        }
        Commands::New {name} => {
            new(name).context("Failed to create project")?;
        },
        Commands::Move {source} => {
            move_project(source).context("Failed to move project")?;
        },
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
        return Err(anyhow!("Chosen base path {} is not empty!", path.to_string_lossy()));
    }

    Environment::create_spiderman_dir(&path);

    Ok(())
}

fn new(name: &str) -> Result<()> {
    Project::new(name)?;

    weave()?;
    Ok(())
}

fn move_project(source: &Path) -> Result<()> {
    if source.exists() && source.is_dir() {
        let project_name = std::path::absolute(source)?.file_name().ok_or(anyhow!("Source directory has no name"))?.to_string_lossy().into_owned();
        let project = Project::new(project_name.as_ref())?;
        let new_path = project.get_project_raw_data_path()?;
        let options = CopyOptions::new();
        move_dir(source, new_path, &options)?;
        weave()?;
        Ok(())
    } else {
        Err(anyhow!("{} does not exist or is not a directory", source.to_string_lossy()))
    }
}

fn tags() -> Result<()> {
    let current_project = Project::get_current_project()?
        .ok_or(anyhow!("Not in a project directory (or subdirectory thereof)!"))?;

    let tags_file = current_project.get_tags_file_path()?;

    if open_in_editor(&tags_file)? {
        weave()?;
    }

    Ok(())
}