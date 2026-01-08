use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};

fn is_hidden(file_name: &OsStr) -> bool {
    file_name.to_string_lossy().starts_with('.')
}

pub fn collect_files(root: &Path) -> io::Result<Vec<PathBuf>> {
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

            if is_hidden(&child_entry.file_name()) {
                continue;
            }

            let child_path = child_entry.path();
            if child_file_type.is_file() {
                files_result.push(child_path);
            } else if child_file_type.is_dir() {
                files_result.extend(collect_files(&child_path)?);
            } else {
                // Not printing on purpose for now, to avoid spamming
                // println!("{:?} is not a file or directory", child_path);
            }
        }
    }

    Ok(files_result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{File, create_dir};

    #[test]
    fn is_hidden_dotfile_returns_true() {
        assert!(is_hidden(OsStr::new(".a")));
    }

    #[test]
    fn is_hidden_dotgit_returns_true() {
        assert!(is_hidden(OsStr::new(".git")));
    }

    #[test]
    fn is_hidden_dotenv_returns_true() {
        assert!(is_hidden(OsStr::new(".env")));
    }

    #[test]
    fn is_hidden_single_dot_returns_true() {
        assert!(is_hidden(OsStr::new(".")));
    }

    #[test]
    fn is_hidden_double_dot_returns_true() {
        assert!(is_hidden(OsStr::new("..")));
    }

    #[test]
    fn is_hidden_plain_filename_returns_false() {
        assert!(!is_hidden(OsStr::new("git")));
    }

    #[test]
    fn is_hidden_dot_in_middle_returns_false() {
        assert!(!is_hidden(OsStr::new("a.env")));
    }

    #[test]
    fn is_hidden_empty_name_returns_false() {
        assert!(!is_hidden(OsStr::new("")));
    }

    #[test]
    fn collect_files_skips_hidden_file_in_visible_directory() {
        // File structure:
        // root/
        // |- a.txt
        // |- .secret

        // Create root folder
        let root = tempfile::tempdir().unwrap();
        let root_path = root.path();

        // Create visible a file under root
        File::create(root_path.join("a.txt")).unwrap();

        // Create hidden secret file under root
        File::create(root_path.join(".secret")).unwrap();

        // When passing "root"
        let collected_paths = collect_files(root_path).unwrap();

        assert_eq!(collected_paths.len(), 1);

        let result = collected_paths.first().unwrap();
        let expect = &root_path.join("a.txt");
        assert_eq!(result, expect);
    }

    #[test]
    fn collect_files_skips_hidden_directory() {
        // File structure:
        // root/
        // |- .git/
        //  |- config
        // |- b.txt

        // Create root folder
        let root = tempfile::tempdir().unwrap();
        let root_path = root.path();

        // Create hidden git folder under root
        let dir_git_path = root_path.join(".git");
        create_dir(&dir_git_path).unwrap();

        // Create visible file inside hidden git folder
        File::create(dir_git_path.join("config")).unwrap();

        // Create visible file under root
        File::create(root_path.join("b.txt")).unwrap();

        // When passing "root"
        let collected_paths = collect_files(root_path).unwrap();

        assert_eq!(collected_paths.len(), 1);

        let result = collected_paths.first().unwrap();
        let expect = &PathBuf::from(root_path.join("b.txt"));

        assert_eq!(result, expect);
    }

    #[test]
    fn collect_files_allows_explicit_hidden_file_root() {
        // File structure:
        // root/
        // |- .env

        // Create root folder
        let root = tempfile::tempdir().unwrap();
        let root_path = root.path();

        // Create hidden env file under root
        let env_path = root_path.join(".env");
        File::create(env_path.as_path()).unwrap();

        // When passing "root/.env"
        let collected_paths = collect_files(env_path.as_path()).unwrap();

        assert_eq!(collected_paths.len(), 1);

        let result = collected_paths.first().unwrap();

        assert_eq!(result, &env_path);
    }

    #[test]
    fn collect_files_allows_explicit_hidden_directory_root() {
        // File structure:
        // root/
        // |- .git/
        //  |- config

        // Create root folder
        let root = tempfile::tempdir().unwrap();
        let root_path = root.path();

        // Create hidden git folder under root
        let dir_git_path = root_path.join(".git");
        create_dir(&dir_git_path).unwrap();

        // Create visible file inside hidden git folder
        let git_config_path = dir_git_path.join("config");
        File::create(&git_config_path).unwrap();

        // When passing "root/.git/"
        let collected_paths = collect_files(dir_git_path.as_path()).unwrap();

        assert_eq!(collected_paths.len(), 1);

        let result = collected_paths.first().unwrap();

        assert_eq!(result, &git_config_path);
    }

    #[test]
    fn collect_files_skips_nested_hidden_directory() {
        // File structure:
        // root/
        // |- visible/
        //  |- .hidden/
        //   |- x.txt

        // Create root folder
        let root = tempfile::tempdir().unwrap();
        let root_path = root.path();

        // Create visible folder under root
        let visible_folder_path = root_path.join("visible");
        create_dir(&visible_folder_path).unwrap();

        // Create hidden folder under visible
        let hidden_folder_path = visible_folder_path.join(".hidden");
        create_dir(&hidden_folder_path).unwrap();

        // Create visible file inside hidden folder
        let visible_file_path = hidden_folder_path.join("x.txt");
        File::create(&visible_file_path).unwrap();

        // When passing "root"
        let collected_paths = collect_files(root_path).unwrap();

        assert_eq!(collected_paths.len(), 0);
    }
}
