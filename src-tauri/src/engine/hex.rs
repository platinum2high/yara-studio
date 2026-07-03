use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::models::HexRegion;

pub const MAX_REGION_BYTES: usize = 4096;

pub fn read_region(path: &Path, start: u64, length: usize) -> Result<HexRegion, String> {
    let metadata = std::fs::metadata(path).map_err(|e| format!("Cannot access file: {e}"))?;
    if metadata.is_dir() {
        return Err("Path is a directory".to_string());
    }

    let file_size = metadata.len();
    let start = start.min(file_size);
    let available = (file_size - start) as usize;
    let length = length.min(MAX_REGION_BYTES).min(available);

    let mut file = File::open(path).map_err(|e| format!("Cannot open file: {e}"))?;
    file.seek(SeekFrom::Start(start))
        .map_err(|e| format!("Cannot seek: {e}"))?;

    let mut bytes = vec![0u8; length];
    file.read_exact(&mut bytes)
        .map_err(|e| format!("Cannot read file: {e}"))?;

    let mut bytes_hex = String::with_capacity(length * 2);
    for b in &bytes {
        bytes_hex.push_str(&format!("{b:02X}"));
    }

    Ok(HexRegion {
        file_size,
        start,
        bytes_hex,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_file(name: &str, content: &[u8]) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("yara-studio-hex-{name}"));
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn reads_exact_region() {
        let path = temp_file("exact.bin", b"0123456789ABCDEF");
        let region = read_region(&path, 4, 8).unwrap();
        assert_eq!(region.start, 4);
        assert_eq!(region.file_size, 16);
        assert_eq!(region.bytes_hex, "3435363738394142"); // "456789AB"
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn clamps_past_end_of_file() {
        let path = temp_file("clamp.bin", b"short");
        let region = read_region(&path, 3, 100).unwrap();
        assert_eq!(region.bytes_hex.len(), 4); // "rt" -> 2 bytes
        let past = read_region(&path, 999, 16).unwrap();
        assert_eq!(past.bytes_hex, "");
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn missing_file_is_an_error() {
        assert!(read_region(Path::new("/no/such/file"), 0, 16).is_err());
    }
}
