use std::fs::DirEntry;
use std::path::Path;
use anyhow::Result;
use crate::{Environment, Project};

pub fn remove_symlinks() -> Result<()> {
    let env = Environment::get();

    let base_path = &env.base_path;
    let spiderman_dir = &env.spiderman_dir;

    for directory in base_path.read_dir()?
        .filter_map(|d| d.ok())
        .filter(|d| d.path() != *spiderman_dir) {
        if directory.metadata().is_ok_and(|m| m.is_dir()) {
            remove_symlinks_impl(&directory)?;
        }
    }

    Ok(())
}

fn remove_symlinks_impl(dir: &DirEntry) -> Result<()> {
    for entry in dir.path().read_dir()?.filter_map(|d| d.ok()) {
        let path = entry.path();
        if path.is_symlink() {
            let canonical_path = entry.path().canonicalize().unwrap();
            let canonical_raw_data_dir_path = Environment::get().raw_storage_dir.canonicalize().unwrap();

            if canonical_path > canonical_raw_data_dir_path {
                // Entry is managed by spiderman (points into raw data directory)
                //println!("DEBUG: Would remove file/symlink {}", path.to_string_lossy());
                remove_symlink_dir(&path)?;
            }
        } else if path.is_dir() {
            remove_symlinks_impl(&entry)?;

        }
    }

    Ok(())
}

pub fn remove_empty_directories() -> Result<()> {
    let env = Environment::get();

    let base_path = &env.base_path;
    let spiderman_dir = &env.spiderman_dir;

    for directory in base_path.read_dir()?
        .filter_map(|d| d.ok())
        .filter(|d| d.path() != *spiderman_dir) {
        if directory.metadata().is_ok_and(|m| m.is_dir()) {
            remove_empty_directories_impl(&directory)?;
            let path = directory.path();
            if path.read_dir()?.next().is_none() {
                std::fs::remove_dir(path)?;
            } else {
                eprintln!("Warning: directory {} is not empty after removing all the symlinks!", path.to_string_lossy());
            }
        }
    }

    Ok(())
}

fn remove_empty_directories_impl(dir: &DirEntry) -> Result<()> {
    for entry in dir.path().read_dir()?.filter_map(|d| d.ok()) {
        let path = entry.path();
        if path.is_dir() {
            remove_empty_directories_impl(&entry)?;
            // path should now be empty
            if path.read_dir()?.next().is_none() {
                std::fs::remove_dir(path)?;
            } else {
                eprintln!("Warning: directory {} is not empty after removing all the symlinks!", path.to_string_lossy());
            }
        } else {
            eprintln!("Warning: {} is not a directory or symlink and should not be here!", path.to_string_lossy());
        }
    }

    Ok(())
}



pub fn construct_view_tree() -> Result<()> {
    let env = Environment::get();

    for project in Project::list()? {
        let raw_data_path = project.get_project_raw_data_path();

        for link_target in env.schema.fill(&project) {
            std::fs::create_dir_all(link_target.parent().unwrap());
            symlink_dir(&raw_data_path, &link_target)?;
        }
    }

    Ok(())
}

#[cfg(unix)]
fn symlink_dir(source: &Path, dest: &Path) -> Result<()> {
    std::os::unix::fs::symlink(raw_data_path, link_target)?;

    Ok(())
}

#[cfg(windows)]
fn symlink_dir(source: &Path, dest: &Path) -> Result<()> {
    std::os::windows::fs::symlink_dir(source, dest)?;

    Ok(())
}

#[cfg(unix)]
fn remove_symlink_dir(path: &Path) -> Result<()> {
    std::fs::remove_file(path)?;

    Ok(())
}

#[cfg(windows)]
fn remove_symlink_dir(path: &Path) -> Result<()> {
    std::fs::remove_dir(path)?;

    Ok(())
}