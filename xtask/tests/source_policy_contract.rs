//! Written-policy regression evidence for the executable source-size law.

const RUST_STANDARDS: &str = include_str!("../../docs/Rust Standards.md");
const SOURCE_FILE: &str = include_str!("../src/source_structure/source_file.rs");
const SOURCE_STRUCTURE: &str = include_str!("../src/source_structure.rs");

#[test]
fn written_source_limit_matches_the_executable_law() {
    assert!(SOURCE_STRUCTURE.contains("SOURCE_MODULE_HARD_LIMIT_LINES: u64 = 500"));
    assert!(
        RUST_STANDARDS
            .contains("**Tests:** same 500-line absolute maximum; prefer scenario subdivision")
    );
    assert!(RUST_STANDARDS.contains("Count physical lines for deterministic enforcement."));
    assert!(!RUST_STANDARDS.contains("Count nonblank, non-comment lines for enforcement"));
}

#[test]
fn source_root_identity_declares_its_unix_scope() {
    assert!(SOURCE_FILE.contains("intentionally supported only on Unix hosts"));
    assert!(SOURCE_FILE.contains("Unix device and inode identity"));
}
