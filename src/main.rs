#![feature(absolute_path)]
#![feature(is_some_and)]

mod environment;
mod schema;
mod config;
mod file_utils;
mod project;
mod weave;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use anyhow::Result;
use crate::environment::Environment;
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
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Weave => {
            weave().expect("Failed to weave");
        },
        Commands::Init {dir} => {
            init(dir).expect("Failed to initialize");
        }
        Commands::New {name} => {
            Project::new(name).expect("Failed to create project");
        },
        Commands::Move {source} => {
            todo!()
        }
    }
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
        panic!("Chosen base path {} is not empty!", path.to_string_lossy());
    }

    Environment::create_spiderman_dir(&path);

    Ok(())
}