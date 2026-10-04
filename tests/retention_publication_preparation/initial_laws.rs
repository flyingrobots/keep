//! Initial retention publication preparation laws.

use std::error::Error;

use keep::{
    AdmittedRetentionRoot, RetentionGenerationExpectation, preflight_retention_transition,
    prepare_retention_publication,
};

use super::fixture::{HEAD_HEX, MANIFEST_HEX, fixture, root_bytes, with_snapshot};

#[test]
fn initial_preparation_reproduces_frozen_manifest_and_head() -> Result<(), Box<dyn Error>> {
    let root_bytes = root_bytes()?;
    let candidate = AdmittedRetentionRoot::decode(&root_bytes)?;
    let preflight = with_snapshot(|snapshot| {
        preflight_retention_transition(
            RetentionGenerationExpectation::Absent,
            None,
            candidate,
            snapshot,
        )
    })??;

    let preparation = prepare_retention_publication(preflight, None)?;
    let publication = preparation
        .publication()
        .ok_or("initial transition did not prepare publication")?;

    assert_eq!(preparation.candidate().encoded(), root_bytes);
    assert_eq!(publication.manifest().encoded(), fixture(MANIFEST_HEX)?);
    assert_eq!(publication.head().encoded().as_slice(), fixture(HEAD_HEX)?);
    assert_eq!(preparation.closure().usage().node_count(), 2);
    Ok(())
}
