use super::{FuzzTarget, TargetError, harnesses, parse_list};
use std::error::Error;
use std::path::Path;
use std::path::PathBuf;

#[test]
fn cargo_registry_is_sorted_before_reconciliation() -> Result<(), TargetError> {
    let targets = parse_list(b"segment_format\nblob_hasher\n".to_vec())?;
    assert_eq!(
        targets.iter().map(FuzzTarget::as_str).collect::<Vec<_>>(),
        ["blob_hasher", "segment_format"]
    );
    Ok(())
}

#[test]
fn checked_in_harness_set_is_exact_and_sorted() -> Result<(), Box<dyn Error>> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repository_root = manifest
        .parent()
        .ok_or("xtask manifest has no repository parent")?;
    let targets = harnesses(repository_root)?;
    assert_eq!(
        targets.iter().map(FuzzTarget::as_str).collect::<Vec<_>>(),
        [
            "blob_hasher",
            "blob_id_binary",
            "blob_id_text",
            "fast_cdc",
            "golden_protocol",
            "layout_record",
            "repository_json",
            "segment_format",
        ]
    );
    Ok(())
}

#[test]
fn empty_duplicate_and_malformed_registries_are_refused() {
    assert!(matches!(
        parse_list(Vec::new()),
        Err(TargetError::EmptyRegistry)
    ));
    assert!(matches!(
        parse_list(b"blob_hasher\nblob_hasher\n".to_vec()),
        Err(TargetError::Duplicate)
    ));
    assert!(matches!(
        parse_list(b"BlobHasher\n".to_vec()),
        Err(TargetError::Malformed(_))
    ));
}

#[test]
fn target_paths_cannot_forge_diagnostic_lines() {
    let error = TargetError::NonRegular(PathBuf::from("first\nError: forged\rrewrite"));
    assert_eq!(
        error.to_string(),
        "fuzz harness is not a regular file: first\\nError: forged\\rrewrite"
    );
}
