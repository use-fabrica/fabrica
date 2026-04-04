//! Path encoding module for filesystem-safe project path representation.
//!
//! Converts absolute project paths to Base64URL-encoded strings for safe storage
//! as directory names. Handles long paths via hash fallback.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};

/// Encodes an absolute project path to a filesystem-safe Base64URL string.
///
/// For paths where encoded result would exceed 200 characters, uses a hash
/// prefix instead (`h` + 16-char hex hash).
pub(crate) fn encode_project_path(path: &Path) -> String {
    let absolute_path = absolutize(path);
    let bytes = absolute_path.as_os_str().as_bytes();
    let encoded = URL_SAFE_NO_PAD.encode(bytes);

    if encoded.len() > 200 {
        let mut hasher = DefaultHasher::new();
        bytes.hash(&mut hasher);
        let hash = hasher.finish();
        format!("h{:016x}", hash)
    } else {
        encoded
    }
}

/// Decodes a Base64URL-encoded path back to the original path.
///
/// Returns an error if the encoded string starts with `h` (hashed path).
pub(crate) fn decode_project_path(encoded: &str) -> anyhow::Result<PathBuf> {
    if encoded.starts_with('h') {
        return Err(anyhow::anyhow!(
            "Cannot decode hashed path (starts with 'h')"
        ));
    }

    let bytes = URL_SAFE_NO_PAD.decode(encoded)?;
    let os_string = std::ffi::OsString::from_vec(bytes);
    Ok(PathBuf::from(os_string))
}

/// Converts a relative path to an absolute path using the current directory.
///
/// Does NOT resolve symlinks (unlike `std::fs::canonicalize()`).
fn absolutize(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .expect("Failed to get current directory")
            .join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let original = PathBuf::from("/home/user/my-project");
        let encoded = encode_project_path(&original);
        let decoded = decode_project_path(&encoded).expect("Failed to decode");
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_encode_unicode() {
        let original = PathBuf::from("/home/用户/项目");
        let encoded = encode_project_path(&original);
        let decoded = decode_project_path(&encoded).expect("Failed to decode");
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_encode_spaces() {
        let original = PathBuf::from("/home/user/my project");
        let encoded = encode_project_path(&original);
        let decoded = decode_project_path(&encoded).expect("Failed to decode");
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_encode_no_padding() {
        let original = PathBuf::from("/home/user/my-project");
        let encoded = encode_project_path(&original);
        assert!(
            !encoded.contains('='),
            "Encoded output should not contain padding"
        );
    }

    #[test]
    fn test_encode_relative_path() {
        let relative = PathBuf::from("./some/path");
        let current_dir = std::env::current_dir().expect("Failed to get current dir");
        let expected = current_dir.join("some/path");

        let encoded = encode_project_path(&relative);
        let decoded = decode_project_path(&encoded).expect("Failed to decode");
        assert_eq!(decoded, expected);
    }

    #[test]
    fn test_encode_long_path() {
        let mut long_path = PathBuf::from("/home/user");
        for i in 0..100 {
            long_path.push(format!("very_long_directory_name_{}", i));
        }

        let encoded = encode_project_path(&long_path);
        assert!(encoded.starts_with('h'), "Long paths should start with 'h'");
        assert_eq!(
            encoded.len(),
            17,
            "Hashed paths should be 17 chars: 'h' + 16 hex digits"
        );
    }

    #[test]
    fn test_decode_hashed_path_fails() {
        let result = decode_project_path("h1234567890abcdef");
        assert!(result.is_err(), "Decoding hashed paths should fail");
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("hashed"),
            "Error should mention hashed paths"
        );
    }
}
