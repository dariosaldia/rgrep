use crate::sniff::is_text_file;
use rayon::prelude::*;
use std::fs::File;
use std::io;
use std::io::{BufRead, Seek};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub fn print_matches<F>(test_match: &F, files: &[PathBuf]) -> (bool, bool)
where
    F: Fn(&str) -> bool + Send + Sync + ?Sized,
{
    let print_lock = Mutex::new(());
    files
        .par_iter()
        .map(|path| scan_one_file(test_match, path.as_path(), &print_lock))
        .reduce(
            || (false, false),
            |(had_match_prev, had_error_prev), (had_match_curr, had_error_curr)| {
                (
                    had_match_prev || had_match_curr,
                    had_error_prev || had_error_curr,
                )
            },
        )
}

fn sniff_text_and_rewind(file: &mut File, path: &Path) -> (bool, bool) {
    match is_text_file(file) {
        Err(e) => {
            eprintln!("Error sniffing file {}. {}", path.display(), e);
            (false, true)
        }
        Ok(false) => (false, false),
        Ok(true) => match file.rewind() {
            Err(e) => {
                eprintln!("Error on file rewind {}. {}", path.display(), e);
                (false, true)
            }
            Ok(_) => (true, false),
        },
    }
}

fn scan_one_file<F>(test_match: &F, path: &Path, print_lock: &Mutex<()>) -> (bool, bool)
where
    F: Fn(&str) -> bool + Send + Sync + ?Sized,
{
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening file {}. {}", path.display(), e);
            return (false, true);
        }
    };

    let (is_text, sniff_had_error) = sniff_text_and_rewind(&mut file, path);

    if sniff_had_error {
        return (false, true);
    }
    if !is_text {
        return (false, false);
    }

    let mut had_match = false;
    let mut had_error = false;

    for line_attempt in io::BufReader::new(file).lines().enumerate() {
        let (number, line) = match line_attempt {
            (line_number, Ok(line)) => (line_number + 1, line),
            (line_number, Err(e)) => {
                eprintln!(
                    "Error reading line {} from file {}. {}",
                    line_number + 1,
                    path.display(),
                    e
                );
                had_error = true;
                continue;
            }
        };

        if test_match(&line) {
            had_match = true;
            let _lock = match print_lock.lock() {
                Ok(lock) => lock,
                Err(e) => {
                    eprintln!(
                        "Error acquiring lock to print line. File {}. {}",
                        path.display(),
                        e
                    );
                    return (true, true);
                }
            };
            println!("{}:{}:{}", path.display(), number, line);
        }
    }

    (had_match, had_error)
}
