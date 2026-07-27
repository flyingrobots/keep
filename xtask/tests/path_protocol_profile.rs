//! Regression laws for the durable corpus path profile.

use xtask::protocol_admission::{POSIX_RELATIVE_PATH_PROFILE, posix_relative_path};

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
    for path in [
        "",
        "/absolute",
        "inputs/",
        "inputs//source",
        "inputs/./source",
        "inputs/../source",
        r"inputs\source",
        "inputs/source:stream",
        "inputs/source\0tail",
    ] {
        assert!(posix_relative_path(path).is_err());
    }
    assert_eq!(
        posix_relative_path("inputs/source.txt").ok(),
        Some("inputs/source.txt".into())
    );
}
