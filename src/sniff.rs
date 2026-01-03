use std::path::Path;
use std::{fs, io};

pub fn is_text_file(file_path: &Path) -> io::Result<bool> {
    let file = match fs::File::open(file_path) {
        Ok(f) => f,
        Err(e) => {
            // eprintln!("Error opening file {}. {}", path.display(), e);
            return Err(e);
        }
    };

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{ErrorKind, Write};
    use tempfile::NamedTempFile;

    #[test]
    fn is_text_file_utf8_returns_true() {
        let mut temp_file = NamedTempFile::new().unwrap();

        temp_file.write_all(b"Hello\nWorld\n").unwrap();

        assert_eq!(true, is_text_file(temp_file.path()).unwrap());
    }

    #[test]
    fn is_text_file_contains_nul_returns_false() {
        let mut temp_file = NamedTempFile::new().unwrap();

        // File with NUL 0x00
        temp_file.write_all(b"Hello\n\x00World\n").unwrap();

        assert_eq!(false, is_text_file(temp_file.path()).unwrap());
    }

    #[test]
    fn is_text_file_invalid_utf8_returns_false() {
        let mut temp_file = NamedTempFile::new().unwrap();

        // File with invalid utf-8 0xFF
        temp_file.write_all(b"Hello\n\xFFWorld\n").unwrap();

        assert_eq!(false, is_text_file(temp_file.path()).unwrap());
    }

    #[test]
    fn is_text_file_empty_file_returns_true() {
        // Empty file
        let temp_file = NamedTempFile::new().unwrap();

        assert_eq!(true, is_text_file(temp_file.path()).unwrap());
    }

    #[test]
    fn is_text_file_missing_file_returns_error() {
        assert_eq!(
            ErrorKind::NotFound,
            is_text_file(Path::new("not_a_file.txt"))
                .unwrap_err()
                .kind()
        );
    }
}
