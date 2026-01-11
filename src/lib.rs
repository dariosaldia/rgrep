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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{File, set_permissions};
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::tempdir;

    #[test]
    fn run_returns_zero_when_any_match_found() {
        // Create root folder
        let root = tempdir().unwrap();
        let root_path = root.path();

        // Create a file containing 'Hello' pattern in it
        let mut file_with_pattern = File::create(root_path.join("with_pattern.txt")).unwrap();
        file_with_pattern
            .write_all(b"This is my\nHello World!\n")
            .unwrap();
        file_with_pattern.flush().unwrap();
        drop(file_with_pattern);

        // Create a second file NOT containing 'Hello' pattern in it
        let mut file_without_pattern = File::create(root_path.join("no_pattern.txt")).unwrap();
        file_without_pattern
            .write_all(b"Just some\nrandom string\n")
            .unwrap();
        file_without_pattern.flush().unwrap();
        drop(file_without_pattern);

        // Create the desired config
        let config = Config {
            regex_mode: false,
            pattern: "Hello".to_string(),
            path: root_path.to_path_buf(),
        };

        let exit_code = run(config);

        // Assert we expect exit_code 0, because there were matches and no errors
        assert_eq!(exit_code, 0);
    }

    #[test]
    fn run_returns_one_when_no_matches_and_no_errors() {
        // Create root folder
        let root = tempdir().unwrap();
        let root_path = root.path();

        // Create a first file NOT containing 'Hello' pattern in it
        let mut file_without_pattern_a = File::create(root_path.join("no_pattern_A.txt")).unwrap();
        file_without_pattern_a
            .write_all(b"This is my\nmy World!\n")
            .unwrap();
        file_without_pattern_a.flush().unwrap();
        drop(file_without_pattern_a);

        // Create a second file NOT containing 'Hello' pattern in it
        let mut file_without_pattern_b = File::create(root_path.join("no_pattern_B.txt")).unwrap();
        file_without_pattern_b
            .write_all(b"Just some\nrandom string\n")
            .unwrap();
        file_without_pattern_b.flush().unwrap();
        drop(file_without_pattern_b);

        // Create the desired config
        let config = Config {
            regex_mode: false,
            pattern: "Hello".to_string(),
            path: root_path.to_path_buf(),
        };

        let exit_code = run(config);

        // Assert we expect exit_code 1, because there were no matches and no errors
        assert_eq!(exit_code, 1);
    }

    #[test]
    fn run_returns_two_when_any_error_occurs() {
        // Create root folder
        let root = tempdir().unwrap();
        let root_path = root.path();

        // Create a file containing 'Hello' pattern in it
        let mut file_with_pattern = File::create(root_path.join("with_pattern.txt")).unwrap();
        file_with_pattern
            .write_all(b"This is my\nHello World!\n")
            .unwrap();
        file_with_pattern.flush().unwrap();
        drop(file_with_pattern);

        // Create another file with no read permissions
        let file_no_permissions_path = root_path.join("no_read_permissions.txt");
        let mut file_no_permissions = File::create(&file_no_permissions_path).unwrap();
        file_no_permissions
            .write_all(b"Just some\nrandom string\n")
            .unwrap();
        file_no_permissions.flush().unwrap();
        drop(file_no_permissions);

        // Remove read permissions from the file to force the error when opening it
        let mut permissions = file_no_permissions_path.metadata().unwrap().permissions();
        permissions.set_mode(permissions.mode() & !0o444);
        set_permissions(file_no_permissions_path, permissions).unwrap();

        // Create the desired config
        let config = Config {
            regex_mode: false,
            pattern: "Hello".to_string(),
            path: root_path.to_path_buf(),
        };

        let exit_code = run(config);

        // Assert we expect exit_code 2, because there was at least 1 error
        assert_eq!(exit_code, 2);
    }
}
