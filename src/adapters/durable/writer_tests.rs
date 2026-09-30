//! Durable ingestion laws: one pass commits a blob readable by its layout
//! at once and by identity once anchored; nearby content reuses every
//! unchanged chunk after exact byte comparison; an exact re-ingest
//! publishes nothing; every limit and identity refusal leaves nothing
//! visible; an interrupted source leaves a stage the recovery protocol
//! discards, and staging refuses until it does.

use std::error::Error;
use std::io::{self, Cursor, Read};

use super::recovery::recover_durable_ingestion_unchecked_for_tests;
use super::test_fixture::{durable_store, identify, long_content};
use super::{DurableIngestionError, DurableStore, DurableWriter};
use crate::adapters::retention::filesystem_retention_test_fixture::{
    catalog_policy, migrated_store,
};
use crate::adapters::{FilesystemVersionTwoAdmission, ReaderAttemptLimit};
use crate::{
    BlobHasher, ContentReads, IngestionError, LayoutEntryLimit, StagedByteLimit, StagingLimits,
};

fn open_writer(root: &std::path::Path) -> Result<DurableWriter, Box<dyn Error>> {
    let admission = FilesystemVersionTwoAdmission::reopen_unchecked_for_tests(root)?;
    Ok(DurableWriter::open(admission, root, catalog_policy()?)?)
}

fn unbounded() -> StagingLimits {
    StagingLimits::entries(LayoutEntryLimit::MAXIMUM)
}

fn nearby(content: &[u8]) -> Vec<u8> {
    let mut edited = content.to_vec();
    for byte in edited.iter_mut().skip(200 * 1024).take(64) {
        *byte = byte.wrapping_add(1);
    }
    edited
}

#[test]
fn one_pass_commits_a_blob_readable_by_layout_then_by_anchor() -> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("durable-ingest-round-trip")?;
    let content = long_content();
    let expected = identify(&content)?;
    let mut writer = open_writer(sandbox.path())?;
    let staged = writer.stage(&mut Cursor::new(&content), unbounded())?;
    assert_eq!(staged.target(), expected.target);
    assert_eq!(staged.layout_id(), expected.record.id());
    assert!(!staged.is_already_visible());
    let receipt = staged.commit()?;
    assert_eq!(receipt.generation().get(), 2);
    assert!(receipt.segment().is_some());
    let accounting = receipt.accounting();
    assert_eq!(accounting.logical_bytes(), u64::try_from(content.len())?);
    assert_eq!(accounting.physical_new_bytes(), accounting.logical_bytes());
    assert_eq!(accounting.physical_reused_bytes(), 0);
    assert_eq!(
        accounting.chunks_new(),
        u64::try_from(expected.spans.len())?
    );
    assert_eq!(accounting.chunks_reused(), 0);
    drop(writer);
    let snapshot = DurableStore::open(
        sandbox.path(),
        catalog_policy()?,
        ReaderAttemptLimit::DEFAULT,
    )
    .snapshot()?;
    assert_eq!(snapshot.view().catalog_generation().get(), 2);
    assert!(!snapshot.contains_blob(receipt.target()));
    let mut output = Vec::new();
    let read = snapshot.reconstruct_layout(receipt.layout_id(), &mut output)?;
    assert_eq!(output, content);
    assert_eq!(read.receipt().target(), receipt.target());
    Ok(())
}

#[test]
fn nearby_content_reuses_every_unchanged_chunk_and_an_exact_re_ingest_publishes_nothing()
-> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("durable-ingest-reuse")?;
    let content = long_content();
    let edited = nearby(&content);
    let mut writer = open_writer(sandbox.path())?;
    let first = writer
        .stage(&mut Cursor::new(&content), unbounded())?
        .commit()?;
    let second = writer
        .stage(&mut Cursor::new(&edited), unbounded())?
        .commit()?;
    assert_eq!(second.generation().get(), 3);
    let accounting = second.accounting();
    assert_eq!(accounting.logical_bytes(), u64::try_from(edited.len())?);
    assert!(accounting.physical_reused_bytes() > 0);
    assert!(accounting.physical_new_bytes() < accounting.logical_bytes());
    assert_eq!(
        accounting.physical_new_bytes() + accounting.physical_reused_bytes(),
        accounting.logical_bytes()
    );
    assert!(accounting.chunks_reused() > accounting.chunks_new());
    let staged = writer.stage(&mut Cursor::new(&content), unbounded())?;
    assert!(staged.is_already_visible());
    let again = staged.commit()?;
    assert_eq!(again.segment(), None);
    assert_eq!(again.generation(), second.generation());
    assert_eq!(again.catalog_digest(), second.catalog_digest());
    assert_eq!(again.target(), first.target());
    assert_eq!(again.layout_id(), first.layout_id());
    assert_eq!(again.accounting().physical_new_bytes(), 0);
    assert_eq!(
        again.accounting().physical_reused_bytes(),
        again.accounting().logical_bytes()
    );
    drop(writer);
    let snapshot = DurableStore::open(
        sandbox.path(),
        catalog_policy()?,
        ReaderAttemptLimit::DEFAULT,
    )
    .snapshot()?;
    let mut output = Vec::new();
    let _read = snapshot.reconstruct_layout(second.layout_id(), &mut output)?;
    assert_eq!(output, edited);
    Ok(())
}

#[test]
fn limit_and_identity_refusals_leave_nothing_visible() -> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("durable-ingest-refusals")?;
    let content = long_content();
    let mut writer = open_writer(sandbox.path())?;
    let short = StagingLimits::new(LayoutEntryLimit::MAXIMUM, StagedByteLimit::new(1024));
    let refusal = writer.stage(&mut Cursor::new(&content), short);
    assert!(matches!(
        refusal,
        Err(DurableIngestionError::Ingestion(
            IngestionError::ByteLimitExceeded { .. }
        ))
    ));
    let one_entry = StagingLimits::entries(LayoutEntryLimit::new(1)?);
    let refusal = writer.stage(&mut Cursor::new(&content), one_entry);
    assert!(matches!(
        refusal,
        Err(DurableIngestionError::Ingestion(IngestionError::Layout(_)))
    ));
    drop(writer);
    let _recovered =
        recover_durable_ingestion_unchecked_for_tests(sandbox.path(), catalog_policy()?)?;
    let mut writer = open_writer(sandbox.path())?;
    let wrong = BlobHasher::new().finish();
    let refusal = writer.stage_expected(&mut Cursor::new(b"short"), wrong, unbounded());
    assert!(matches!(
        refusal,
        Err(DurableIngestionError::Ingestion(
            IngestionError::BlobIdentityMismatch { .. }
        ))
    ));
    drop(writer);
    let _recovered =
        recover_durable_ingestion_unchecked_for_tests(sandbox.path(), catalog_policy()?)?;
    let snapshot = DurableStore::open(
        sandbox.path(),
        catalog_policy()?,
        ReaderAttemptLimit::DEFAULT,
    )
    .snapshot()?;
    assert_eq!(snapshot.view().catalog_generation().get(), 1);
    Ok(())
}

struct Interrupted<'a> {
    remaining: &'a [u8],
    after: usize,
}

impl Read for Interrupted<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.after == 0 {
            return Err(io::Error::other("the source went away"));
        }
        let take = buffer.len().min(self.remaining.len()).min(self.after);
        let (head, tail) = self.remaining.split_at(take);
        if let Some(slot) = buffer.get_mut(..take) {
            slot.copy_from_slice(head);
        }
        self.remaining = tail;
        self.after = self.after.saturating_sub(take);
        Ok(take)
    }
}

#[test]
fn an_interrupted_source_leaves_a_stage_that_recovery_discards() -> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("durable-ingest-interrupted")?;
    let content = long_content();
    let mut writer = open_writer(sandbox.path())?;
    let mut source = Interrupted {
        remaining: &content,
        after: 300 * 1024,
    };
    let refusal = writer.stage(&mut source, unbounded());
    assert!(matches!(
        refusal,
        Err(DurableIngestionError::Ingestion(
            IngestionError::Read { .. }
        ))
    ));
    assert!(sandbox.path().join("staging/current.seg").exists());
    let retained = writer.stage(&mut Cursor::new(&content), unbounded());
    assert!(matches!(
        retained,
        Err(DurableIngestionError::StageRetained)
    ));
    drop(writer);
    let recovered =
        recover_durable_ingestion_unchecked_for_tests(sandbox.path(), catalog_policy()?)?;
    assert_eq!(recovered.discarded().len(), 1);
    assert!(!sandbox.path().join("staging/current.seg").exists());
    let mut writer = open_writer(sandbox.path())?;
    let receipt = writer
        .stage(&mut Cursor::new(&content), unbounded())?
        .commit()?;
    assert_eq!(receipt.generation().get(), 2);
    Ok(())
}

#[test]
fn an_anchored_ingest_satisfies_the_port_laws_beside_the_fixture_store()
-> Result<(), Box<dyn Error>> {
    let (sandbox, published) = durable_store("durable-ingest-beside", &[b"anchored", b"spare"])?;
    let content = long_content();
    let mut writer = open_writer(sandbox.path())?;
    let receipt = writer
        .stage(&mut Cursor::new(&content), unbounded())?
        .commit()?;
    assert_eq!(receipt.generation().get(), 3);
    drop(writer);
    let snapshot = DurableStore::open(
        sandbox.path(),
        catalog_policy()?,
        ReaderAttemptLimit::DEFAULT,
    )
    .snapshot()?;
    let anchored = published.first().ok_or("anchored blob")?;
    assert!(ContentReads::contains_blob(&snapshot, anchored.target));
    let mut output = Vec::new();
    let _read = snapshot.reconstruct_layout(receipt.layout_id(), &mut output)?;
    assert_eq!(output, content);
    Ok(())
}
