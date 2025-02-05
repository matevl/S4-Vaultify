use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

fn read_initial_bytes<P: AsRef<Path>>(file_path: P, num_bytes: usize) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut buffer = vec![0; num_bytes];
    let useful_bytes = file.read(&mut buffer)?;
    buffer.truncate(useful_bytes);
    Ok(buffer)
}

pub fn detect_type<P: AsRef<Path>>(file_path: P) -> io::Result<String> {
    let buffer = read_initial_bytes(file_path, 16)?;

    if buffer.len() >= 8 && &buffer[..8] == b"\x89PNG\r\n\x1A\n" {
        return Ok("PNG image".to_string());
    }
    if buffer.len() >= 4 && &buffer[..4] == b"%PDF" {
        return Ok("PDF document".to_string());
    }
    if buffer.len() >= 3 && buffer[0] == 0xFF && buffer[1] == 0xD8 && buffer[2] == 0xFF {
        return Ok("JPEG image".to_string());
    }
    if buffer.len() >= 4 && &buffer[..4] == b"PK\x03\x04" {
        return Ok("ZIP archive".to_string());
    }
    Ok("Unknown file type".to_string())
}

