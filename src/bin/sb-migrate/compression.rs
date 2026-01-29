//! Gzip decompression for storyboard files

use flate2::read::GzDecoder;
use std::io::Read;

/// Check if data is gzip compressed (magic bytes: 0x1f 0x8b)
pub fn is_gzipped(data: &[u8]) -> bool {
    data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b
}

/// Decompress gzip data if compressed, otherwise return as-is
pub fn maybe_decompress(data: Vec<u8>) -> Result<Vec<u8>, std::io::Error> {
    if is_gzipped(&data) {
        let mut decoder = GzDecoder::new(&data[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    } else {
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    #[test]
    fn test_is_gzipped_true() {
        let data = [0x1f, 0x8b, 0x08, 0x00]; // gzip magic + compression method
        assert!(is_gzipped(&data));
    }

    #[test]
    fn test_is_gzipped_false() {
        let data = b"Hello, World!";
        assert!(!is_gzipped(data));
    }

    #[test]
    fn test_is_gzipped_empty() {
        let data: &[u8] = &[];
        assert!(!is_gzipped(data));
    }

    #[test]
    fn test_maybe_decompress_plain() {
        let data = b"Hello, World!".to_vec();
        let result = maybe_decompress(data.clone()).unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn test_maybe_decompress_gzipped() {
        // Create gzipped data
        let original = b"Hello, World!";
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(original).unwrap();
        let compressed = encoder.finish().unwrap();

        // Decompress
        let result = maybe_decompress(compressed).unwrap();
        assert_eq!(result, original);
    }
}
