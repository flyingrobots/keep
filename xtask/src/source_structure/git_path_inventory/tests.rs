use std::io::Cursor;

use super::{GitOutputLimits, SourceStructureError, bounded_bytes, read_paths};

const TEST_LIMITS: GitOutputLimits = GitOutputLimits {
    diagnostic_bytes: 3,
    path_bytes: 3,
    path_stream_bytes: 6,
    paths: 2,
};

#[test]
fn git_path_stream_refuses_an_oversized_path_without_buffering_the_tail() {
    let result = read_paths(Cursor::new(b"abcd\0"), "test paths", TEST_LIMITS);
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
    let result = read_paths(Cursor::new(b"a\0b\0c\0"), "test paths", TEST_LIMITS);
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
    let result = read_paths(Cursor::new(b"\0\0\0\0\0\0\0"), "test paths", TEST_LIMITS);
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
    let result = read_paths(Cursor::new(b"abc"), "test paths", TEST_LIMITS);
    assert!(matches!(
        result,
        Err(SourceStructureError::GitOutputFraming { .. })
    ));
}

#[test]
fn git_diagnostics_are_drained_but_only_the_bound_is_retained() {
    let result = bounded_bytes(Cursor::new(b"abcdef"), TEST_LIMITS.diagnostic_bytes);
    assert!(matches!(
        result,
        Ok(ref diagnostic) if diagnostic.bytes == b"abc" && diagnostic.exceeded
    ));
}
