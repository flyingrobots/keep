//! Closure-member re-verification laws: publication under filesystem
//! authority re-reads every closure member from this store's own pools, and
//! the exact admission error a corrupt member produces travels as the
//! `source` of the publication refusal.

use std::error::Error;
use std::fs;
use std::path::Path;

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, SEGMENT_NAME, fixture, initial_preparation, migrated_store, refusal,
    reopen_authority, retention_witness,
};
use super::{
    RetentionCurrentStateRefusal, RetentionPublicationStorage, RetentionTransitionDisposition,
};
use crate::adapters::{CatalogRestartError, SegmentRecordAdmissionError};

/// The one-zero bundle segment: a chunk record at 64 whose payload byte is at
/// 176 and checksum at 177, and a layout record at 209 whose payload starts
/// at 321 and checksum at 541.
const CHUNK_PAYLOAD_OFFSET: usize = 176;
const CHUNK_CHECKSUM_OFFSET: usize = 177;
const CHUNK_RECORD_OFFSET: usize = 64;
const LAYOUT_PAYLOAD_OFFSET: usize = 321;
const LAYOUT_CHECKSUM_OFFSET: usize = 541;
const LAYOUT_RECORD_OFFSET: usize = 209;

#[test]
fn an_intact_store_reverifies_every_member_and_admits_publication() -> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("retention-member-intact")?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    let mut authority = reopen_authority(sandbox.path())?;

    let disposition = authority.verify_current(&preparation)?;

    assert_eq!(disposition, RetentionTransitionDisposition::Publish);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn a_corrupt_chunk_member_refuses_with_its_admission_error_as_source() -> Result<(), Box<dyn Error>>
{
    let sandbox = migrated_store("retention-member-corrupt-chunk")?;
    corrupt_record(
        sandbox.path(),
        CHUNK_RECORD_OFFSET,
        CHUNK_PAYLOAD_OFFSET,
        CHUNK_CHECKSUM_OFFSET,
    )?;

    let error = refuse_publication(sandbox.path())?;

    assert!(matches!(
        refusal(&error),
        Some(RetentionCurrentStateRefusal::ClosureMemberRefused { .. })
    ));
    let admission = find_source::<SegmentRecordAdmissionError>(&error)
        .ok_or("the chunk admission error was erased from the source chain")?;
    assert!(matches!(
        admission,
        SegmentRecordAdmissionError::ChunkIdentityMismatch { .. }
    ));
    sandbox.remove()?;
    Ok(())
}

#[test]
fn a_corrupt_layout_member_refuses_with_its_layout_admission_error_as_source()
-> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("retention-member-corrupt-layout")?;
    corrupt_record(
        sandbox.path(),
        LAYOUT_RECORD_OFFSET,
        LAYOUT_PAYLOAD_OFFSET,
        LAYOUT_CHECKSUM_OFFSET,
    )?;

    let error = refuse_publication(sandbox.path())?;

    assert!(matches!(
        refusal(&error),
        Some(RetentionCurrentStateRefusal::ClosureMemberRefused { .. })
    ));
    let admission = find_source::<SegmentRecordAdmissionError>(&error)
        .ok_or("the layout admission error was erased from the source chain")?;
    assert!(matches!(
        admission,
        SegmentRecordAdmissionError::Layout { .. }
    ));
    sandbox.remove()?;
    Ok(())
}

#[test]
fn a_missing_member_segment_refuses_with_the_restart_error_as_source() -> Result<(), Box<dyn Error>>
{
    let sandbox = migrated_store("retention-member-missing")?;
    fs::remove_file(sandbox.path().join("segments").join(SEGMENT_NAME))?;

    let error = refuse_publication(sandbox.path())?;

    assert!(matches!(
        refusal(&error),
        Some(RetentionCurrentStateRefusal::ClosureMemberRefused { .. })
    ));
    assert!(find_source::<CatalogRestartError>(&error).is_some());
    sandbox.remove()?;
    Ok(())
}

/// Runs current-state verification against a store whose pool is damaged
/// and proves it refuses before touching the retention namespace.
fn refuse_publication(root: &Path) -> Result<std::io::Error, Box<dyn Error>> {
    let before = retention_witness(root)?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    let mut authority = reopen_authority(root)?;
    let error = authority
        .verify_current(&preparation)
        .err()
        .ok_or("a damaged closure member was unexpectedly admitted")?;
    drop(authority);
    assert_eq!(retention_witness(root)?, before);
    Ok(error)
}

/// Flips one payload byte of the record at `record` and reseals that record's
/// checksum, so the record is content-valid framing whose payload no longer
/// hashes to its declared identity. The segment seal is left stale: record
/// admission refuses before the outer digest is compared.
fn corrupt_record(
    root: &Path,
    record: usize,
    payload: usize,
    checksum: usize,
) -> Result<(), Box<dyn Error>> {
    let path = root.join("segments").join(SEGMENT_NAME);
    let mut bytes = fs::read(&path)?;
    let byte = bytes.get_mut(payload).ok_or("payload offset")?;
    *byte ^= 1;
    let covered = bytes.get(record..checksum).ok_or("record coverage")?;
    let covered_length = u64::try_from(covered.len())?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"KEEP:SEG:RECORD:SUM\0");
    hasher.update(&1_u16.to_be_bytes());
    hasher.update(&[1]);
    hasher.update(covered);
    hasher.update(&covered_length.to_be_bytes());
    let resealed = *hasher.finalize().as_bytes();
    let end = checksum.checked_add(32).ok_or("checksum end")?;
    bytes
        .get_mut(checksum..end)
        .ok_or("checksum slot")?
        .copy_from_slice(&resealed);
    fs::write(path, bytes)?;
    Ok(())
}

/// Walks the `source` chain from an `io::Error`'s payload downward.
fn find_source<T: Error + 'static>(error: &std::io::Error) -> Option<&T> {
    let mut cursor: &(dyn Error + 'static) = error.get_ref()?;
    loop {
        if let Some(found) = cursor.downcast_ref::<T>() {
            return Some(found);
        }
        cursor = cursor.source()?;
    }
}
