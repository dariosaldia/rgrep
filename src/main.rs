use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::{env, io};

fn parse_args() -> Result<(String, PathBuf), String> {
    let usage = "Usage: rgrep <pattern> <path>".to_string();

    let mut args = env::args();

    args.next(); // Skip executable path

    let pattern = args.next().ok_or(usage.clone())?;

    let path = args.next().map(|s| PathBuf::from(s)).ok_or(usage)?;

    Ok((pattern, path))
}

fn collect_files(root: &Path) -> io::Result<Vec<PathBuf>> {
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

fn main() {
    match parse_args() {
        Ok((pattern, path)) => {
            println!("pattern: {} - path: {}", pattern, &path.display());
            if let Ok(files) = collect_files(&path) {
                for file in files.iter().take(5) {
                    println!("file: {}", file.display());
                }
            }
        }
        Err(e) => {
            eprintln!("{e}");
            exit(2);
        }
    }
}
