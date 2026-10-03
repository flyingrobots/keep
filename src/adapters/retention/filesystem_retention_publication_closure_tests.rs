//! These laws own live transitive admission after pure publication preflight.

use std::error::Error;
use std::fs;
use std::io;

use super::RetentionPublicationError;
use super::filesystem_retention_test_fixture::{
    ROOT_HEX, SEGMENT_NAME, fixture, initial_preparation, open_authority, retention_witness,
};
use crate::adapters::{
    CatalogRestartError, CatalogRestartPhase, SegmentReadError, SegmentSealError,
};
use crate::execute_retention_publication;

#[derive(Clone, Copy, Debug)]
enum Damage {
    Missing,
    Corrupt,
}

// Size: medium. Oracle: preflight is not proof of live reconstructability at publication.
// Delete only with this protocol or a stronger runtime-boundary replacement.
#[test]
fn publication_refuses_a_segment_removed_after_preflight() -> Result<(), Box<dyn Error>> {
    require_refusal(Damage::Missing)
}
#[test]
fn publication_refuses_a_segment_corrupted_after_preflight() -> Result<(), Box<dyn Error>> {
    require_refusal(Damage::Corrupt)
}

fn require_refusal(damage: Damage) -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority(&format!("publication-live-closure-{damage:?}"))?;
    let bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&bytes)?;
    let path = sandbox.path().join("segments").join(SEGMENT_NAME);
    match damage {
        Damage::Missing => fs::remove_file(path)?,
        Damage::Corrupt => {
            let mut bytes = fs::read(&path)?;
            *bytes.last_mut().ok_or("empty segment")? ^= 1;
            fs::write(path, bytes)?;
        }
    }
    let before = retention_witness(sandbox.path())?;
    let result = execute_retention_publication(&mut authority, &preparation);
    assert_eq!(
        retention_witness(sandbox.path())?,
        before,
        "live closure refusal must preserve retained bytes for {damage:?}: {result:?}"
    );
    let error = match &result {
        Err(RetentionPublicationError::CurrentVerification { source }) => source,
        other => {
            return Err(format!("live closure must refuse current verification: {other:?}").into());
        }
    };
    let restart = error
        .get_ref()
        .and_then(|source| source.downcast_ref::<CatalogRestartError>());
    let exact = match (damage, restart) {
        (Damage::Missing, Some(CatalogRestartError::SegmentIo { phase, source, .. })) => {
            *phase == CatalogRestartPhase::OpenSegment && source.kind() == io::ErrorKind::NotFound
        }
        (Damage::Corrupt, Some(CatalogRestartError::Segment { source, .. })) => matches!(
            source.as_ref(),
            SegmentReadError::Seal {
                source: SegmentSealError::SealChecksumMismatch { .. }
            }
        ),
        _ => false,
    };
    assert!(
        exact,
        "live closure must retain exact {damage:?} refusal: {result:?}"
    );
    Ok(())
}
