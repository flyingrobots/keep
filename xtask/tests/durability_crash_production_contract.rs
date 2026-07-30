//! Production-protocol ownership laws for the process-death crash matrix.

#![cfg(feature = "repository-tasks")]

use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[test]
fn crash_children_execute_every_claimed_production_protocol() -> Result<(), Box<dyn Error>> {
    let source_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let child = fs::read_to_string(source_root.join("durability_crash_matrix/child.rs"))?;
    let protocol = [
        "durability_crash_matrix/production_protocol.rs",
        "durability_crash_matrix/production_protocol/initialization.rs",
        "durability_crash_matrix/production_protocol/publication.rs",
        "durability_crash_matrix/production_protocol/recovery.rs",
    ]
    .into_iter()
    .map(|path| fs::read_to_string(source_root.join(path)))
    .collect::<Result<String, _>>()?;

    assert!(child.contains("production_protocol::run("));
    assert!(!child.contains("state::prepare("));
    for required in [
        "StagedSegment::begin(",
        "publish_catalog_generation(",
        "initialize_store(",
        "execute_recovery_stage_discard(",
    ] {
        assert!(
            protocol.contains(required),
            "crash protocol does not execute {required}"
        );
    }
    Ok(())
}
