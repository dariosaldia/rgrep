use regex::Regex;
use rgrep::scanner::print_matches;
use rgrep::walker::collect_files;
use std::env;
use std::path::PathBuf;
use std::process::exit;

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

    let matcher: Box<dyn Fn(&str) -> bool + Send + Sync> = if regex_mode {
        let regex = match Regex::new(&pattern) {
            Ok(regex) => regex,
            Err(e) => {
                eprintln!("Regex not valid: {}", e);
                exit(2);
            }
        };
        Box::new(move |line| regex.is_match(line))
    } else {
        Box::new(move |line| line.contains(&pattern))
    };

    print_matches(matcher.as_ref(), &files);
}
