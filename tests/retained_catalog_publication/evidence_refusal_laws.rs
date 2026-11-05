//! Absent, corrupt, or contradictory retention evidence refuses before any stage.

use std::error::Error;
use std::io;

use keep::{
    LivenessGeneration, RetainedCatalogPublicationError as Refusal, RetentionEvidencePhase,
    RetentionHead, RetentionHeadDecodeError, RetentionManifestDecodeError, RetentionManifestLength,
    RetentionRootDecodeError,
};

use crate::catalog_fixture::{Candidate, with_candidate};
use crate::model_storage::ModelStorage;
use crate::publish;
use crate::retention_fixture::{RetentionRecords, RootShape, encode_head};
use crate::support::require_error;

/// Publishes the preserving candidate and requires a refusal with no stage.
fn refuse(records: RetentionRecords) -> Result<Refusal, Box<dyn Error>> {
    refuse_with(ModelStorage::new(records))
}

fn refuse_with(mut storage: ModelStorage) -> Result<Refusal, Box<dyn Error>> {
    let result = with_candidate(Candidate::Preserving, |call| publish(&mut storage, &call))?;
    let error = require_error(result, "unadmitted retention evidence published")?;
    assert!(storage.stages().is_empty(), "no catalog stage may run");
    Ok(error)
}

fn flip_last(bytes: &mut [u8]) -> Result<(), Box<dyn Error>> {
    let last = bytes.last_mut().ok_or("record is empty")?;
    *last ^= 1;
    Ok(())
}

#[test]
fn absent_manifest_refuses_with_its_exact_read() -> Result<(), Box<dyn Error>> {
    let mut records = RetentionRecords::golden()?;
    records.manifest = None;

    let error = refuse(records)?;

    assert!(matches!(
        error,
        Refusal::RetentionStorage { phase: RetentionEvidencePhase::ReadManifest, ref source }
            if source.kind() == io::ErrorKind::NotFound
    ));
    Ok(())
}

#[test]
fn absent_root_refuses_with_its_manifest_entry() -> Result<(), Box<dyn Error>> {
    let mut records = RetentionRecords::golden()?;
    let expected = *records
        .entries()?
        .first()
        .ok_or("golden manifest is empty")?;
    records.roots.clear();

    let error = refuse(records)?;

    assert!(matches!(
        error,
        Refusal::RootStorage { entry, ref source }
            if entry == expected && source.kind() == io::ErrorKind::NotFound
    ));
    Ok(())
}

#[test]
fn failed_head_read_refuses_at_admission() -> Result<(), Box<dyn Error>> {
    let storage = ModelStorage::new(RetentionRecords::golden()?).failing_head_read(0);

    let error = refuse_with(storage)?;

    assert!(matches!(
        error,
        Refusal::RetentionStorage {
            phase: RetentionEvidencePhase::AdmitHead,
            ..
        }
    ));
    Ok(())
}

#[test]
fn corrupt_head_refuses_with_its_checksum() -> Result<(), Box<dyn Error>> {
    let mut records = RetentionRecords::golden()?;
    flip_last(records.head.as_mut().ok_or("golden head is absent")?)?;

    let error = refuse(records)?;

    assert!(matches!(
        error,
        Refusal::HeadDecode {
            phase: RetentionEvidencePhase::AdmitHead,
            source: RetentionHeadDecodeError::ChecksumMismatch { .. }
        }
    ));
    Ok(())
}

#[test]
fn corrupt_manifest_refuses_with_its_checksum() -> Result<(), Box<dyn Error>> {
    let mut records = RetentionRecords::golden()?;
    let (_, manifest) = records
        .manifest
        .as_mut()
        .ok_or("golden manifest is absent")?;
    flip_last(manifest)?;

    let error = refuse(records)?;

    assert!(matches!(
        error,
        Refusal::ManifestDecode { ref source }
            if matches!(**source, RetentionManifestDecodeError::ChecksumMismatch { .. })
    ));
    Ok(())
}

#[test]
fn corrupt_root_refuses_with_its_entry_and_checksum() -> Result<(), Box<dyn Error>> {
    let mut records = RetentionRecords::golden()?;
    let expected = *records
        .entries()?
        .first()
        .ok_or("golden manifest is empty")?;
    let (_, root) = records.roots.first_mut().ok_or("golden root is absent")?;
    flip_last(root)?;

    let error = refuse(records)?;

    assert!(matches!(
        error,
        Refusal::RootDecode { entry, ref source }
            if entry == expected
                && matches!(**source, RetentionRootDecodeError::ChecksumMismatch { .. })
    ));
    Ok(())
}

#[test]
fn substituted_root_refuses_with_expected_and_observed_entries() -> Result<(), Box<dyn Error>> {
    let mut records = RetentionRecords::golden()?;
    let expected = *records
        .entries()?
        .first()
        .ok_or("golden manifest is empty")?;
    let other = RetentionRecords::built(&[(b"substitute", RootShape::Retaining)])?;
    let observed = *other.entries()?.first().ok_or("built manifest is empty")?;
    let (_, substitute) = other.roots.first().ok_or("built root is absent")?;
    let (_, root) = records.roots.first_mut().ok_or("golden root is absent")?;
    root.clone_from(substitute);

    let error = refuse(records)?;

    assert!(matches!(
        error,
        Refusal::RootBinding { expected: e, observed: ref o } if e == expected && **o == observed
    ));
    Ok(())
}

#[test]
fn substituted_manifest_refuses_with_both_digests() -> Result<(), Box<dyn Error>> {
    let mut records = RetentionRecords::golden()?;
    let expected = records.current_head()?.manifest_digest();
    let other = RetentionRecords::built(&[(b"contract", RootShape::Released)])?;
    let (observed, substitute) = other.manifest.ok_or("built manifest is absent")?;
    let (_, manifest) = records
        .manifest
        .as_mut()
        .ok_or("golden manifest is absent")?;
    *manifest = substitute;

    let error = refuse(records)?;

    assert!(matches!(
        error,
        Refusal::ManifestDigest { expected: e, observed: o } if e == expected && o == observed
    ));
    Ok(())
}

#[test]
fn head_naming_another_manifest_length_refuses_before_decoding() -> Result<(), Box<dyn Error>> {
    let mut records = RetentionRecords::golden()?;
    let current = records.current_head()?;
    let actual = current.manifest_length().get();
    let forged = RetentionManifestLength::new(actual.checked_add(72).ok_or("length overflow")?)?;
    let head = RetentionHead::new(
        current.generation(),
        forged,
        current.manifest_digest(),
        None,
    )?;
    records.head = Some(encode_head(&head));

    let error = refuse(records)?;

    assert!(matches!(
        error,
        Refusal::ManifestLength { expected, observed }
            if expected == forged && u64::try_from(observed).ok() == Some(actual)
    ));
    Ok(())
}

#[test]
fn head_naming_another_generation_refuses_with_both_generations() -> Result<(), Box<dyn Error>> {
    let mut records = RetentionRecords::golden()?;
    let current = records.current_head()?;
    let forged = LivenessGeneration::new(2)?;
    let head = RetentionHead::new(
        forged,
        current.manifest_length(),
        current.manifest_digest(),
        Some(current.manifest_digest()),
    )?;
    records.head = Some(encode_head(&head));

    let error = refuse(records)?;

    assert!(matches!(
        error,
        Refusal::ManifestGeneration { expected, observed }
            if expected == forged && observed == current.generation()
    ));
    Ok(())
}

#[test]
fn head_naming_another_predecessor_refuses_with_both_predecessors() -> Result<(), Box<dyn Error>> {
    let golden = RetentionRecords::golden()?;
    let golden_digest = golden.current_head()?.manifest_digest();
    let generation = LivenessGeneration::new(2)?;
    let manifest =
        keep::RetentionManifest::new(generation, Some(golden_digest), golden.entries()?)?;
    let canonical = keep::CanonicalRetentionManifest::from_manifest(&manifest)?;
    let length = RetentionManifestLength::new(u64::try_from(canonical.encoded().len())?)?;
    let forged = RetentionRecords::built(&[(b"forged", RootShape::Released)])?.current_head()?;
    let head = RetentionHead::new(
        generation,
        length,
        canonical.digest(),
        Some(forged.manifest_digest()),
    )?;
    let records = RetentionRecords {
        head: Some(encode_head(&head)),
        manifest: Some((canonical.digest(), canonical.encoded().to_vec())),
        roots: golden.roots,
    };

    let error = refuse(records)?;

    assert!(matches!(
        error,
        Refusal::ManifestPredecessor { expected, observed }
            if expected == Some(forged.manifest_digest()) && observed == Some(golden_digest)
    ));
    Ok(())
}
