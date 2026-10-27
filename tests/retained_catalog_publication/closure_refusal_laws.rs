//! Candidates that lose retained content refuse before any catalog mutation.

use std::error::Error;

use keep::{RetainedCatalogPublicationError, RetentionClosureVerificationError};

use crate::catalog_fixture::{Candidate, dropped_identity, with_candidate};
use crate::model_storage::{EvidenceRead, ModelStorage};
use crate::publish;
use crate::retention_fixture::{RetentionRecords, RootShape};
use crate::support::require_error;

#[test]
fn candidate_dropping_a_retained_layout_refuses_before_any_stage() -> Result<(), Box<dyn Error>> {
    require_loss_refused(Candidate::DroppedLayout)
}

#[test]
fn candidate_substituting_a_retained_chunk_refuses_before_any_stage() -> Result<(), Box<dyn Error>>
{
    require_loss_refused(Candidate::SubstitutedChunk)
}

#[test]
fn every_manifest_root_is_verified_wherever_it_sorts() -> Result<(), Box<dyn Error>> {
    let mut positions = Vec::new();
    for names in [[b"alpha".as_slice(), b"omega"], [b"omega", b"alpha"]] {
        let [retaining, released] = names;
        let records = RetentionRecords::built(&[
            (retaining, RootShape::Retaining),
            (released, RootShape::Released),
        ])?;
        let entries = records.entries()?;
        let retained = retained_entry(&records, &entries)?;
        positions.push(entries.iter().position(|entry| *entry == retained));
        let mut storage = ModelStorage::new(records.clone());

        let result = with_candidate(Candidate::DroppedLayout, |call| {
            publish(&mut storage, &call)
        })?;
        let error = require_error(result, "candidate dropped a retained root's layout")?;

        assert!(matches!(
            error,
            RetainedCatalogPublicationError::Closure { entry, .. } if entry == retained
        ));
        assert!(storage.stages().is_empty());
        let mut storage = ModelStorage::new(records);
        let receipt = with_candidate(Candidate::Preserving, |call| publish(&mut storage, &call))??;
        assert_eq!(
            receipt
                .preserved_manifest()
                .map(keep::PreservedRetentionManifest::root_count),
            Some(2)
        );
    }
    positions.sort_unstable();
    assert_eq!(
        positions,
        [Some(0), Some(1)],
        "both manifest orders are exercised"
    );
    Ok(())
}

fn require_loss_refused(candidate: Candidate) -> Result<(), Box<dyn Error>> {
    let records = RetentionRecords::golden()?;
    let golden_entry = *records
        .entries()?
        .first()
        .ok_or("golden manifest is empty")?;
    let expected = dropped_identity(candidate)?;
    let mut storage = ModelStorage::new(records);

    let result = with_candidate(candidate, |call| publish(&mut storage, &call))?;
    let error = require_error(result, "candidate that loses retained content published")?;

    let RetainedCatalogPublicationError::Closure { entry, source } = &error else {
        return Err(format!("unexpected refusal: {error:?}").into());
    };
    assert_eq!(*entry, golden_entry);
    assert!(matches!(
        source.as_ref(),
        RetentionClosureVerificationError::MissingMember { identity } if *identity == expected
    ));
    assert!(std::error::Error::source(&error).is_some());
    assert!(storage.stages().is_empty(), "no catalog stage may run");
    assert_eq!(
        storage.reads(),
        [
            (EvidenceRead::Head, 0),
            (EvidenceRead::Manifest, 0),
            (EvidenceRead::Root(golden_entry), 0),
        ]
    );
    Ok(())
}

fn retained_entry(
    records: &RetentionRecords,
    entries: &[keep::RetentionManifestEntry],
) -> Result<keep::RetentionManifestEntry, Box<dyn Error>> {
    for entry in entries {
        let (_, bytes) = records
            .roots
            .iter()
            .find(|(digest, _)| *digest == entry.root_digest())
            .ok_or("built records omit a root")?;
        if keep::AdmittedRetentionRoot::decode(bytes)?
            .root()
            .anchor_count()
            > 0
        {
            return Ok(*entry);
        }
    }
    Err("built records retain nothing".into())
}
