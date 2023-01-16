use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Write};
use std::path::Path;
use std::process::Command;
use anyhow::{anyhow, Result};

pub fn create_or_open_file_with_dirs<B>(path: &Path, default_contents: impl FnOnce() -> B) -> Result<File>
    where B: AsRef<[u8]> {
    let parent_dir = path.parent().unwrap();
    std::fs::create_dir_all(parent_dir)?;

    return match File::open(path) {
        Ok(file) => { Ok(file) }
        Err(e) => {
            match e.kind() {
                ErrorKind::NotFound => {
                    File::create(path)?.write_all(default_contents().as_ref())?;
                    Ok(File::open(path)?)
                }
                _ => {
                    Err(anyhow!(e))
                }
            }
        }
    }
}

pub fn open_in_editor(path: &Path) -> Result<bool> {
    if let Ok(editor) = std::env::var("EDITOR") {
        Command::new(editor).arg(path.to_string_lossy().into_owned()).output()?;
        Ok(true)
    } else {
        eprintln!("EDITOR environment variable not set, can't open project tags file.");
        eprintln!("Please edit {} manually and then run spiderman weave.", path.to_string_lossy());
        Ok(false)
    }
}