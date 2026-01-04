use std::io::Read;
use std::path::Path;
use std::{fs, io, str};

const NUMBER_OF_BYTES_TO_SNIFF: usize = 4096;
const NUL_BYTE: u8 = b'\x00';

pub fn is_text_file(file_path: &Path) -> io::Result<bool> {
    // Try to open the file
    let mut file = fs::File::open(file_path)?;

    // Instantiate the sized byte array filled with zeros to hold the file-sniffed-data
    let mut buf = [0; NUMBER_OF_BYTES_TO_SNIFF];

    // Read some bytes from the file
    let bytes_read = file.read(&mut buf)?;

    // When the number of bytes read is zero, the file is empty
    if bytes_read == 0 {
        return Ok(true);
    }

    // Take the relevant slice of bytes that were read from the file
    let b = &buf[..bytes_read];

    // Check for NUL byte or try converting to str
    if b.iter().any(|b| b == &NUL_BYTE) || str::from_utf8(b).is_err() {
        return Ok(false);
    }

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

        assert!(is_text_file(temp_file.path()).unwrap());
    }

    #[test]
    fn is_text_file_long_file_with_valid_prefix_returns_true() {
        let mut temp_file = NamedTempFile::new().unwrap();

        // Write a valid header
        temp_file.write_all(b"Hello\nWorld\n").unwrap();

        // Write at least the first chunk that we want to inspect for utf8 validity
        let buf = [b'a'; NUMBER_OF_BYTES_TO_SNIFF];
        temp_file.write_all(&buf).unwrap();

        // Write an invalid utf8 byte
        temp_file.write_all(b"\xFF").unwrap();

        assert!(is_text_file(temp_file.path()).unwrap());
    }

    #[test]
    fn is_text_file_contains_nul_returns_false() {
        let mut temp_file = NamedTempFile::new().unwrap();

        // File with NUL 0x00
        temp_file.write_all(b"Hello\n\x00World\n").unwrap();

        assert!(!is_text_file(temp_file.path()).unwrap());
    }

    #[test]
    fn is_text_file_invalid_utf8_returns_false() {
        let mut temp_file = NamedTempFile::new().unwrap();

        // File with invalid utf-8 0xFF
        temp_file.write_all(b"Hello\n\xFFWorld\n").unwrap();

        assert!(!is_text_file(temp_file.path()).unwrap());
    }

    #[test]
    fn is_text_file_contains_lone_continuation_byte_returns_false() {
        let mut temp_file = NamedTempFile::new().unwrap();

        // File with lone continuation byte 0x80
        temp_file.write_all(b"Hello\n\x80World\n").unwrap();

        assert!(!is_text_file(temp_file.path()).unwrap());
    }

    #[test]
    fn is_text_file_empty_file_returns_true() {
        // Empty file
        let temp_file = NamedTempFile::new().unwrap();

        assert!(is_text_file(temp_file.path()).unwrap());
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
