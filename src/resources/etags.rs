use std::fmt::Write;

use sha2::{Digest, Sha256};

/// Computes a strong-looking, cheap ETag: the first 8 bytes of the SHA-256 of
/// the content, hex encoded and quoted.
///
/// Every cacheable response uses this function, so pre-rendered pages and
/// cached static files tag their bodies identically.
pub fn of(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    let mut out = String::with_capacity(18);
    out.push('"');
    for byte in &digest[..8] {
        // Writing into a String cannot fail.
        let _ = write!(out, "{byte:02x}");
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::of;

    #[test]
    fn produces_quoted_16_hex_digits() {
        let etag = of(b"hello");
        assert!(etag.starts_with('"'));
        assert!(etag.ends_with('"'));
        assert_eq!(etag.len(), 18);
        assert!(etag[1..17].chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn distinguishes_different_content() {
        assert_ne!(of(b"a"), of(b"b"));
    }
}
