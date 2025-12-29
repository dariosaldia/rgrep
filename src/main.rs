use std::env;
use std::path::PathBuf;
use std::process::exit;

fn parse_args() -> Result<(String, PathBuf), String> {
    let usage = "Usage: rgrep <pattern> <path>".to_string();

    let mut args = env::args();

    args.next(); // Skip executable path

    let pattern = args.next().ok_or(usage.clone())?;

    let path = args.next().map(|s| PathBuf::from(s)).ok_or(usage)?;

    Ok((pattern, path))
}

fn main() {
    match parse_args() {
        Ok((pattern, path)) => {
            println!("pattern: {} - path: {}", pattern, path.as_path().display());
        }
        Err(e) => {
            eprintln!("{e}");
            exit(2);
        }
    }
}
