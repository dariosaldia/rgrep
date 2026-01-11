use crate::scanner::print_matches;
use crate::walker::collect_files;
use regex::Regex;
use std::path::PathBuf;

pub mod scanner;
pub mod sniff;
pub mod walker;

pub struct Config {
    pub regex_mode: bool,
    pub pattern: String,
    pub path: PathBuf,
}

pub fn run(config: Config) -> i32 {
    let files = match collect_files(&config.path) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };

    let matcher: Box<dyn Fn(&str) -> bool + Send + Sync> = if config.regex_mode {
        let regex = match Regex::new(&config.pattern) {
            Ok(regex) => regex,
            Err(e) => {
                eprintln!("Regex not valid: {}", e);
                return 2;
            }
        };
        Box::new(move |line| regex.is_match(line))
    } else {
        Box::new(move |line| line.contains(&config.pattern))
    };

    let (had_match, had_error) = print_matches(matcher.as_ref(), &files);
    if had_error {
        2
    } else if had_match {
        0
    } else {
        1
    }
}
