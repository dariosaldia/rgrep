use std::io::Read;
use std::{io, str};

const NUMBER_OF_BYTES_TO_SNIFF: usize = 4096;
const NUL_BYTE: u8 = b'\x00';

pub fn is_text_file(file: &mut impl Read) -> io::Result<bool> {
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
    use std::io::{Seek, Write};
    use tempfile::NamedTempFile;

    #[test]
    fn is_text_file_utf8_returns_true() {
        let mut temp_file = NamedTempFile::new().unwrap();

        temp_file.write_all(b"Hello\nWorld\n").unwrap();

        temp_file.rewind().unwrap();

        assert!(is_text_file(temp_file.as_file_mut()).unwrap());
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

        temp_file.rewind().unwrap();

        assert!(is_text_file(temp_file.as_file_mut()).unwrap());
    }

    #[test]
    fn is_text_file_contains_nul_returns_false() {
        let mut temp_file = NamedTempFile::new().unwrap();

        // File with NUL 0x00
        temp_file.write_all(b"Hello\n\x00World\n").unwrap();

        temp_file.rewind().unwrap();

        assert!(!is_text_file(temp_file.as_file_mut()).unwrap());
    }

    #[test]
    fn is_text_file_invalid_utf8_returns_false() {
        let mut temp_file = NamedTempFile::new().unwrap();

        // File with invalid utf-8 0xFF
        temp_file.write_all(b"Hello\n\xFFWorld\n").unwrap();

        temp_file.rewind().unwrap();

        assert!(!is_text_file(temp_file.as_file_mut()).unwrap());
    }

    #[test]
    fn is_text_file_contains_lone_continuation_byte_returns_false() {
        let mut temp_file = NamedTempFile::new().unwrap();

        // File with lone continuation byte 0x80
        temp_file.write_all(b"Hello\n\x80World\n").unwrap();

        temp_file.rewind().unwrap();

        assert!(!is_text_file(temp_file.as_file_mut()).unwrap());
    }

    #[test]
    fn is_text_file_empty_file_returns_true() {
        // Empty file
        let mut temp_file = NamedTempFile::new().unwrap();

        assert!(is_text_file(temp_file.as_file_mut()).unwrap());
    }
}
