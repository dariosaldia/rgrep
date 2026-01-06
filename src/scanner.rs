use crate::sniff::is_text_file;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;
use std::io::{BufRead, Seek};
use std::path::{Path, PathBuf};
use std::{fs, io};

pub fn print_matches<F>(test_match: &F, files: &[PathBuf])
where
    F: Fn(&str) -> bool + Send + Sync + ?Sized,
{
    files.par_iter().for_each(|path| {
        scan_one_file(test_match, path.as_path());
    });
}

fn scan_one_file<F>(test_match: &F, path: &Path)
where
    F: Fn(&str) -> bool + Send + Sync + ?Sized,
{
    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening file {}. {}", path.display(), e);
            return;
        }
    };

    match is_text_file(&mut file) {
        Err(e) => {
            eprintln!("Error sniffing file {}. {}", path.display(), e);
            return;
        }
        Ok(false) => return,
        Ok(true) => {
            if let Err(e) = file.rewind() {
                eprintln!("Error on file rewind {}. {}", path.display(), e);
                return;
            }
        }
    }

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
                continue;
            }
        };

        if test_match(&line) {
            println!("{}:{}:{}", path.display(), number, line);
        }
    }
}
