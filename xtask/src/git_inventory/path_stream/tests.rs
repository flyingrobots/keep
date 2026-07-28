//! This module owns adversarial Git path-stream tests.

use std::io::Cursor;

use super::{GitInventoryError, GitPathLimits, read_paths_with};

const TEST_LIMITS: GitPathLimits = GitPathLimits {
    path_bytes: 3,
    stream_bytes: 6,
    paths: 2,
};

#[test]
fn git_path_stream_refuses_an_oversized_path_without_buffering_the_tail() {
    let result = read_paths_with(Cursor::new(b"abcd\0"), "test paths", TEST_LIMITS);
    assert!(matches!(
        result,
        Err(GitInventoryError::OutputBound {
            stream: "path bytes",
            maximum: 3,
            ..
        })
    ));
}

#[test]
fn git_path_stream_refuses_an_oversized_inventory() {
    let result = read_paths_with(Cursor::new(b"a\0b\0c\0"), "test paths", TEST_LIMITS);
    assert!(matches!(
        result,
        Err(GitInventoryError::OutputBound {
            stream: "path count",
            maximum: 2,
            ..
        })
    ));
}

#[test]
fn git_path_stream_refuses_excess_framing_bytes() {
    let result = read_paths_with(Cursor::new(b"\0\0\0\0\0\0\0"), "test paths", TEST_LIMITS);
    assert!(matches!(
        result,
        Err(GitInventoryError::OutputBound {
            stream: "path stream bytes",
            maximum: 6,
            ..
        })
    ));
}

#[test]
fn git_path_stream_requires_nul_framing() {
    let result = read_paths_with(Cursor::new(b"abc"), "test paths", TEST_LIMITS);
    assert!(matches!(
        result,
        Err(GitInventoryError::OutputFraming {
            operation: "test paths"
        })
    ));
}

#[test]
fn git_path_stream_refuses_empty_records() {
    for stream in [&b"\0"[..], &b"a\0\0"[..]] {
        let result = read_paths_with(Cursor::new(stream), "test paths", TEST_LIMITS);
        assert!(matches!(
            result,
            Err(GitInventoryError::OutputFraming {
                operation: "test paths"
            })
        ));
    }
}

#[test]
fn git_path_stream_refuses_duplicate_records() {
    let result = read_paths_with(Cursor::new(b"a\0a\0"), "test paths", TEST_LIMITS);
    assert!(matches!(
        result,
        Err(GitInventoryError::DuplicatePath(ref path)) if path == b"a"
    ));
}

#[test]
fn duplicate_non_utf8_path_diagnostics_preserve_bytes() {
    let result = read_paths_with(
        Cursor::new([u8::MAX, 0, u8::MAX, 0]),
        "test paths",
        TEST_LIMITS,
    );
    let diagnostic = result.err().map(|error| error.to_string());
    assert_eq!(
        diagnostic,
        Some(String::from("git returned duplicate path `\\xff`"))
    );
}

#[test]
fn git_path_stream_preserves_records_before_source_selection() {
    let result = read_paths_with(Cursor::new(b"..\0"), "test paths", TEST_LIMITS);
    let paths = result.map(|paths| {
        paths
            .iter()
            .map(|path| path.as_bytes().to_vec())
            .collect::<Vec<_>>()
    });
    assert_eq!(paths.ok(), Some(vec![b"..".to_vec()]));
}

#[test]
fn git_path_stream_admits_non_source_repository_spellings() {
    let result = read_paths_with(Cursor::new(b"x:y\0"), "test paths", TEST_LIMITS);
    let paths = result.map(|paths| {
        paths
            .iter()
            .map(|path| path.as_bytes().to_vec())
            .collect::<Vec<_>>()
    });
    assert_eq!(paths.ok(), Some(vec![b"x:y".to_vec()]));
}

#[test]
fn git_path_stream_preserves_non_utf8_non_source_records() {
    let result = read_paths_with(Cursor::new([u8::MAX, 0]), "test paths", TEST_LIMITS);
    assert!(matches!(
        result,
        Ok(ref paths) if paths.iter().any(|path| path.as_bytes() == [u8::MAX])
    ));
}
