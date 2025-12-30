use regex::Regex;
use std::ffi::OsStr;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::{env, fs, io};

fn parse_args() -> Result<(bool, String, PathBuf), String> {
    let usage = "Usage: rgrep [--regex] <pattern> <path>".to_string();

    let mut args = env::args();

    args.next(); // Skip executable path

    // read first argument
    let pattern_or_regex_mode = args.next().ok_or(usage.clone())?;

    // find out if user wants regex search
    let regex_mode = pattern_or_regex_mode == "--regex";

    let pattern = if regex_mode {
        // if user wants regex search we must read the second argument
        args.next().ok_or(usage.clone())?
    } else {
        pattern_or_regex_mode
    };

    // read third argument
    let path = args.next().map(|s| PathBuf::from(s)).ok_or(usage)?;

    Ok((regex_mode, pattern, path))
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

fn print_matches<F>(test_match: F, files: &[PathBuf])
where
    F: Fn(&str) -> bool,
{
    for path in files {
        let file = match fs::File::open(path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error opening file {}. {}", path.display(), e);
                continue;
            }
        };
        for line_attempt in io::BufReader::new(file).lines().enumerate() {
            let (number, line) = match line_attempt {
                (line_number, Ok(line)) => (line_number + 1, line),
                (line_number, Err(e)) => {
                    eprintln!(
                        "Error reading file {} at line {}. {}",
                        path.display(),
                        line_number + 1,
                        e
                    );
                    continue;
                }
            };

            if test_match(&line) {
                println!("{}:{}:{}", path.display(), number, line);
            }
        }
    }
}

fn main() {
    let (regex_mode, pattern, path) = match parse_args() {
        Ok((regex_mode, pattern, path)) => (regex_mode, pattern, path),
        Err(e) => {
            eprintln!("{e}");
            exit(2);
        }
    };

    let files = match collect_files(&path) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("{e}");
            exit(2);
        }
    };

    let matcher: Box<dyn Fn(&str) -> bool> = if regex_mode {
        let regex = match Regex::new(&pattern) {
            Ok(regex) => regex,
            Err(e) => {
                eprintln!("Regex not valid: {}", e);
                exit(2);
            }
        };
        Box::new(move |line: &str| regex.is_match(line))
    } else {
        Box::new(|line: &str| line.contains(&pattern))
    };

    print_matches(matcher.as_ref(), &files);
}
