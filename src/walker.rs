use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};

pub fn collect_files(root: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files_result: Vec<PathBuf> = Vec::new();

    if root.is_file() {
        files_result.push(root.to_path_buf());
    } else if root.is_dir() {
        let root_read_dir = root.read_dir();

        let iter = match root_read_dir {
            Ok(iter) => iter,
            Err(e) => {
                eprintln!("Error reading dir: {}", e);
                return Ok(files_result);
            }
        };

        for entry in iter {
            let child_entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    continue;
                }
            };

            let child_file_type = match child_entry.file_type() {
                Ok(file_type) => file_type,
                Err(e) => {
                    eprintln!("Error reading file type: {}", e);
                    continue;
                }
            };

            let child_path = child_entry.path();
            if child_file_type.is_file() {
                files_result.push(child_path);
            } else if child_file_type.is_dir() {
                if child_entry.file_name() == OsStr::new(".git") {
                    continue;
                }
                files_result.extend(collect_files(&child_path)?);
            } else {
                // Not printing on purpose for now, to avoid spamming
                // println!("{:?} is not a file or directory", child_path);
            }
        }
    }

    Ok(files_result)
}
