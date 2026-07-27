//! Regression laws for the durable corpus path profile.

use xtask::protocol_admission::{
    POSIX_RELATIVE_PATH_PROFILE, RelativePathError, posix_relative_path,
};

const SPECIFICATION: &str = include_str!("../../docs/conformance/golden-file-worldline.md");
const RATIONALE: &str = include_str!("../../conformance/golden-file-worldline/v1/rationale.md");

#[test]
fn path_profile_is_named_specified_and_rationalized() {
    assert_eq!(
        POSIX_RELATIVE_PATH_PROFILE,
        "keep.golden-file-worldline.path/v1"
    );
    assert!(SPECIFICATION.contains(POSIX_RELATIVE_PATH_PROFILE));
    assert!(SPECIFICATION.contains("empty path segments"));
    assert!(SPECIFICATION.contains("backslash, colon, or NUL"));
    assert!(RATIONALE.contains(POSIX_RELATIVE_PATH_PROFILE));
    assert!(RATIONALE.contains("Reject rather than normalize"));
}

#[test]
fn path_profile_refuses_every_documented_ambiguous_spelling() {
    for (path, expected) in [
        ("", RelativePathError::Empty),
        ("/absolute", RelativePathError::Absolute),
        ("inputs/", RelativePathError::EmptySegment),
        ("inputs//source", RelativePathError::EmptySegment),
        ("inputs/./source", RelativePathError::DotSegment),
        ("inputs/../source", RelativePathError::ParentSegment),
        (r"inputs\source", RelativePathError::Backslash),
        ("inputs/source:stream", RelativePathError::Colon),
        ("inputs/source\0tail", RelativePathError::Nul),
    ] {
        assert_eq!(posix_relative_path(path), Err(expected));
    }
    assert_eq!(
        posix_relative_path("inputs/source.txt").ok(),
        Some("inputs/source.txt".into())
    );
}
