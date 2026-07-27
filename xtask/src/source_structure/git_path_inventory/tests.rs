//! This module owns adversarial Git stream and diagnostic-bound tests.

use std::io::Cursor;

use super::{GitOutputLimits, SourceStructureError, bounded_bytes, git_failure, read_paths};

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
fn git_path_encoding_failure_names_the_path_stream() {
    let result = read_paths(Cursor::new([u8::MAX, 0]), "test paths", TEST_LIMITS);
    assert!(matches!(
        result,
        Err(SourceStructureError::GitPathEncoding {
            operation: "test paths",
            ..
        })
    ));
}

#[test]
fn git_diagnostic_encoding_failure_retains_exit_status() {
    let error = git_failure("test diagnostics", Some(9), vec![u8::MAX]);
    assert!(matches!(
        error,
        SourceStructureError::GitDiagnosticEncoding {
            operation: "test diagnostics",
            code: Some(9),
            ..
        }
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
