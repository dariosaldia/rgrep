use rgrep::{Config, run};
use std::env;
use std::path::PathBuf;
use std::process::exit;

fn parse_args() -> Result<Config, String> {
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

    let config = Config {
        regex_mode,
        pattern,
        path,
    };

    Ok(config)
}

fn main() {
    let config = match parse_args() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{e}");
            exit(2);
        }
    };
    exit(run(config));
}
