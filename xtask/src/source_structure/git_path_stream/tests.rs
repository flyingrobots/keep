//! This module owns adversarial Git path-stream tests.

use std::io::Cursor;

use super::{GitPathLimits, SourceStructureError, read_paths_with};

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
        Err(SourceStructureError::GitOutputBound {
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
        Err(SourceStructureError::GitOutputBound {
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
        Err(SourceStructureError::GitOutputBound {
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
        Err(SourceStructureError::GitOutputFraming {
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
            Err(SourceStructureError::GitOutputFraming {
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
        Err(SourceStructureError::DuplicatePath(ref path)) if path == "a"
    ));
}

#[test]
fn git_path_stream_preserves_records_before_source_selection() {
    let result = read_paths_with(Cursor::new(b"..\0"), "test paths", TEST_LIMITS);
    let paths = result.map(|paths| {
        paths
            .iter()
            .map(|path| path.as_str().to_owned())
            .collect::<Vec<_>>()
    });
    assert_eq!(paths.ok(), Some(vec![String::from("..")]));
}

#[test]
fn git_path_stream_admits_non_source_repository_spellings() {
    let result = read_paths_with(Cursor::new(b"x:y\0"), "test paths", TEST_LIMITS);
    let paths = result.map(|paths| {
        paths
            .iter()
            .map(|path| path.as_str().to_owned())
            .collect::<Vec<_>>()
    });
    assert_eq!(paths.ok(), Some(vec![String::from("x:y")]));
}

#[test]
fn git_path_encoding_failure_names_the_path_stream() {
    let result = read_paths_with(Cursor::new([u8::MAX, 0]), "test paths", TEST_LIMITS);
    assert!(matches!(
        result,
        Err(SourceStructureError::GitPathEncoding {
            operation: "test paths",
            ..
        })
    ));
}
