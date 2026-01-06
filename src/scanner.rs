use crate::sniff::is_text_file;
use rayon::prelude::*;
use std::fs::File;
use std::io;
use std::io::{BufRead, Seek};
use std::path::{Path, PathBuf};

pub fn print_matches<F>(test_match: &F, files: &[PathBuf])
where
    F: Fn(&str) -> bool + Send + Sync + ?Sized,
{
    files.par_iter().for_each(|path| {
        scan_one_file(test_match, path.as_path());
    });
}

fn sniff_text_and_rewind(file: &mut File, path: &Path) -> bool {
    match is_text_file(file) {
        Err(e) => {
            eprintln!("Error sniffing file {}. {}", path.display(), e);
            false
        }
        Ok(false) => false,
        Ok(true) => {
            if let Err(e) = file.rewind() {
                eprintln!("Error on file rewind {}. {}", path.display(), e);
                false
            } else {
                true
            }
        }
    }
}

fn scan_one_file<F>(test_match: &F, path: &Path)
where
    F: Fn(&str) -> bool + Send + Sync + ?Sized,
{
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening file {}. {}", path.display(), e);
            return;
        }
    };

    if !sniff_text_and_rewind(&mut file, path) {
        return;
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
