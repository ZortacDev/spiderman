use crate::{Environment, Project};
use anyhow::Result;
use std::fs::DirEntry;
use std::path::Path;

pub fn remove_symlinks() -> Result<()> {
    let env = Environment::get()?;

    let base_path = &env.base_path;
    let spiderman_dir = &env.spiderman_dir;

    for directory in base_path
        .read_dir()?
        .filter_map(|d| d.ok())
        .filter(|d| d.path() != *spiderman_dir)
    {
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
            let canonical_raw_data_dir_path =
                Environment::get()?.raw_storage_dir.canonicalize().unwrap();

            if canonical_path.starts_with(canonical_raw_data_dir_path) {
                // Entry is managed by spiderman (points into raw data directory)
                remove_symlink_dir(&path)?;
            }
        } else if path.is_dir() {
            remove_symlinks_impl(&entry)?;
        }
    }

    Ok(())
}

pub fn remove_empty_directories() -> Result<()> {
    let env = Environment::get()?;

    let base_path = &env.base_path;
    let spiderman_dir = &env.spiderman_dir;

    for directory in base_path
        .read_dir()?
        .filter_map(|d| d.ok())
        .filter(|d| d.path() != *spiderman_dir)
    {
        if directory.metadata().is_ok_and(|m| m.is_dir()) {
            remove_empty_directories_impl(&directory)?;
            let path = directory.path();
            if path.read_dir()?.next().is_none() {
                std::fs::remove_dir(path)?;
            } else {
                eprintln!(
                    "Warning: directory {} is not empty after removing all the symlinks!",
                    path.to_string_lossy()
                );
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
                eprintln!(
                    "Warning: directory {} is not empty after removing all the symlinks!",
                    path.to_string_lossy()
                );
            }
        } else {
            eprintln!(
                "Warning: {} is not a directory or symlink and should not be here!",
                path.to_string_lossy()
            );
        }
    }

    Ok(())
}

pub fn construct_view_tree() -> Result<()> {
    let env = Environment::get()?;

    for project in Project::list()? {
        let raw_data_path = project.get_project_raw_data_path()?;

        for mut link_target in env.schema.fill(&project)? {
            std::fs::create_dir_all(link_target.parent().unwrap())?;

            // Add a counter for duplicate link targets
            let mut counter = 1;
            while link_target.exists() {
                if counter == 1 && link_target.extension().is_some() {
                    link_target.set_extension(format!(
                        "{}.{}",
                        link_target.extension().unwrap().to_string_lossy(),
                        counter
                    ));
                } else {
                    link_target.set_extension(format!("{}", counter));
                }
                counter += 1;
            }

            if counter != 1 {
                eprintln!("Warning: Could not link project with UUID {} to proper target, as that target already exists. Linked to {} instead.",
                          project.uuid.hyphenated().to_string(),
                          link_target.to_string_lossy());
            }

            symlink_dir(&raw_data_path, &link_target)?;
        }
    }

    Ok(())
}

#[cfg(unix)]
fn symlink_dir(source: &Path, dest: &Path) -> Result<()> {
    std::os::unix::fs::symlink(source, dest)?;

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
