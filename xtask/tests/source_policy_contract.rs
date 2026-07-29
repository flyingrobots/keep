//! Written-policy regression evidence for the executable source-size law.

const RUST_STANDARDS: &str = include_str!("../../docs/Rust Standards.md");
const GIT_PROCESS: &str = include_str!("../src/git_inventory/process.rs");
const REPOSITORY_FILE: &str = include_str!("../src/repository_file.rs");
const SOURCE_STRUCTURE: &str = include_str!("../src/source_structure.rs");
const SOURCE_INVENTORY: &str = include_str!("../src/source_structure/source_inventory.rs");

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
fn repository_file_admission_declares_its_unix_scope() {
    assert!(REPOSITORY_FILE.contains("intentionally supported only on Unix hosts"));
    assert!(REPOSITORY_FILE.contains("Unix device and inode identity"));
}

#[test]
fn source_scan_revalidates_repository_identity_after_reading() {
    assert_eq!(
        SOURCE_STRUCTURE
            .matches("verify_source_root(&source_root, repository_root)?;")
            .count(),
        3
    );
}

#[test]
fn source_inventory_uses_the_admitted_repository_directory() {
    assert!(SOURCE_STRUCTURE.contains(".process_directory()"));
    assert!(SOURCE_INVENTORY.contains("paths_with("));
    assert!(!SOURCE_INVENTORY.contains("paths as git_paths"));
    assert!(!GIT_PROCESS.contains("current_dir("));
}

#[test]
fn regular_source_admission_has_one_refusal_boundary() {
    assert_eq!(
        SOURCE_STRUCTURE
            .matches("SourceFileAdmission::NonRegular")
            .count(),
        1
    );
    assert_eq!(SOURCE_STRUCTURE.matches("fn admit_regular(").count(), 1);
}
