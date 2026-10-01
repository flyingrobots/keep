//! Durable read laws: every anchored blob reconstructs exactly with a
//! receipt naming the view; ranges across chunk boundaries emit exactly the
//! requested bytes; absence and unanchored layouts refuse as evidence; a
//! pinned snapshot keeps its view while a successor publishes and blocks
//! collection; identical views yield identical receipts.

use std::error::Error;

use super::test_fixture::{durable_store, long_content};
use super::{DurableReadError, DurableSnapshot, DurableStore};
use crate::adapters::compaction::{
    FilesystemCompactionAuthority, observe_compaction, plan_compaction,
};
use crate::adapters::gc::{
    FilesystemGcAuthority, FilesystemGcError, GcLimits, observe_gc_liveness, plan_gc,
};
use crate::adapters::retention::filesystem_retention_test_fixture::catalog_policy;
use crate::adapters::{
    FilesystemRetentionSnapshot, FilesystemVersionTwoAdmission, ReaderAttemptLimit,
};
use crate::{ByteLength, ByteOffset, ByteRange, ReconstructionError};

fn store(root: &std::path::Path) -> Result<DurableStore, Box<dyn Error>> {
    Ok(DurableStore::open(
        root,
        catalog_policy()?,
        ReaderAttemptLimit::DEFAULT,
    ))
}

fn range(offset: u64, length: u64) -> Result<ByteRange, Box<dyn Error>> {
    Ok(ByteRange::new(
        ByteOffset::new(offset),
        ByteLength::new(length),
    )?)
}

#[test]
fn every_anchored_blob_reconstructs_exactly_with_a_receipt_naming_the_view()
-> Result<(), Box<dyn Error>> {
    let long = long_content();
    let contents: [&[u8]; 3] = [b"first durable blob", &long, b"unanchored"];
    let (sandbox, published) = durable_store("durable-reconstruct", &contents)?;
    let snapshot = store(sandbox.path())?.snapshot()?;
    assert_eq!(snapshot.view().catalog_generation().get(), 2);
    assert!(matches!(
        snapshot.view().retention(),
        crate::adapters::GcRetentionState::Published { generation, .. } if generation.get() == 1
    ));
    for (bytes, entry) in contents.iter().zip(&published).take(2) {
        assert!(snapshot.contains_blob(entry.target));
        let mut output = Vec::new();
        let receipt = snapshot.reconstruct(entry.target, &mut output)?;
        assert_eq!(output.as_slice(), *bytes);
        assert_eq!(receipt.receipt().target(), entry.target);
        assert_eq!(receipt.receipt().layout_id(), entry.layout);
        assert_eq!(
            receipt.receipt().bytes_written().get(),
            u64::try_from(bytes.len())?
        );
        assert_eq!(receipt.view(), snapshot.view());
        let mut again = Vec::new();
        let exact = snapshot.reconstruct_layout(entry.layout, &mut again)?;
        assert_eq!(again.as_slice(), *bytes);
        assert_eq!(exact, receipt);
    }
    sandbox.remove()?;
    Ok(())
}

#[test]
fn ranges_across_chunk_boundaries_emit_exactly_the_requested_bytes() -> Result<(), Box<dyn Error>> {
    let long = long_content();
    let contents: [&[u8]; 2] = [&long, b"tail"];
    let (sandbox, published) = durable_store("durable-range", &contents)?;
    let snapshot = store(sandbox.path())?.snapshot()?;
    let entry = published.first().ok_or("published")?;
    for (offset, length) in [(0, 1), (65_000, 70_000), (200_000, 300_000), (524_287, 1)] {
        let mut output = Vec::new();
        let receipt = snapshot.read_range(entry.target, range(offset, length)?, &mut output)?;
        let start = usize::try_from(offset)?;
        let end = usize::try_from(offset.saturating_add(length))?;
        assert_eq!(Some(output.as_slice()), long.get(start..end));
        assert_eq!(receipt.receipt().bytes_written().get(), length);
        assert_eq!(receipt.receipt().requested(), range(offset, length)?);
        assert_eq!(receipt.view(), snapshot.view());
    }
    let mut output = Vec::new();
    let error = snapshot
        .read_range(entry.target, range(524_288, 1)?, &mut output)
        .err()
        .ok_or("a range past the end was read")?;
    assert!(matches!(error, DurableReadError::RangeRead(_)), "{error}");
    sandbox.remove()?;
    Ok(())
}

#[test]
fn absence_is_evidence_against_the_pinned_view() -> Result<(), Box<dyn Error>> {
    let contents: [&[u8]; 2] = [b"anchored", b"committed but unanchored"];
    let (sandbox, published) = durable_store("durable-absent", &contents)?;
    let snapshot = store(sandbox.path())?.snapshot()?;
    let unanchored = published.get(1).ok_or("published")?;
    assert!(!snapshot.contains_blob(unanchored.target));
    let mut output = Vec::new();
    let error = snapshot
        .reconstruct(unanchored.target, &mut output)
        .err()
        .ok_or("an unanchored blob reconstructed")?;
    assert!(
        matches!(error, DurableReadError::BlobMissing { requested } if requested == unanchored.target)
    );
    assert!(output.is_empty());
    // The exact committed layout is still readable by identity: the
    // catalog names it, and the fence protects it.
    let receipt = snapshot.reconstruct_layout(unanchored.layout, &mut output)?;
    assert_eq!(output.as_slice(), b"committed but unanchored");
    assert_eq!(receipt.receipt().target(), unanchored.target);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn a_pinned_snapshot_keeps_its_view_beside_a_successor_and_blocks_collection()
-> Result<(), Box<dyn Error>> {
    let contents: [&[u8]; 2] = [b"kept across compaction", b"unreachable"];
    let (sandbox, published) = durable_store("durable-pinned", &contents)?;
    let policy = catalog_policy()?;
    let anchored = published.first().ok_or("published")?;
    let pinned = store(sandbox.path())?.snapshot()?;
    assert_eq!(pinned.view().catalog_generation().get(), 2);

    // A compaction successor publishes beside the pinned reader.
    let view = FilesystemRetentionSnapshot::load_under_writer_authority(
        sandbox.path(),
        policy,
        ReaderAttemptLimit::DEFAULT,
    )?;
    let plan = plan_compaction(&observe_compaction(sandbox.path(), &view, policy)?)?;
    drop(view);
    let admission = FilesystemVersionTwoAdmission::reopen_unchecked_for_tests(sandbox.path())?;
    let receipt =
        FilesystemCompactionAuthority::open(admission, sandbox.path(), policy)?.execute(&plan)?;
    assert_eq!(receipt.generation().get(), 3);

    // The pinned view still reads its generation; a fresh view reads the successor.
    let mut output = Vec::new();
    let old = pinned.reconstruct(anchored.target, &mut output)?;
    assert_eq!(output.as_slice(), b"kept across compaction");
    assert_eq!(old.view().catalog_generation().get(), 2);
    let fresh = store(sandbox.path())?.snapshot()?;
    let mut again = Vec::new();
    let new = fresh.reconstruct(anchored.target, &mut again)?;
    assert_eq!(again, output);
    assert_eq!(new.view().catalog_generation().get(), 3);
    assert_eq!(
        new.receipt(),
        old.receipt(),
        "the same identities at a new location"
    );
    drop(fresh);

    // Collection refuses while the pinned view lives, then retires.
    let fenced = FilesystemRetentionSnapshot::load_under_writer_authority(
        sandbox.path(),
        policy,
        ReaderAttemptLimit::DEFAULT,
    )?;
    let gc = plan_gc(
        &observe_gc_liveness(sandbox.path(), &fenced, policy)?,
        GcLimits::MAXIMUM,
    )?;
    drop(fenced);
    // The bundle segment was never anchored, so compaction omitted it too:
    // every segment the receipt superseded is now a retirement candidate.
    assert_eq!(
        usize::try_from(gc.candidate_count())?,
        receipt.superseded().len()
    );
    let admission = FilesystemVersionTwoAdmission::reopen_unchecked_for_tests(sandbox.path())?;
    let mut collector = FilesystemGcAuthority::open(admission, sandbox.path(), policy)?;
    let error = collector
        .execute(&gc)
        .err()
        .ok_or("GC ran beside a pinned reader")?;
    assert!(matches!(error, FilesystemGcError::ReadersActive), "{error}");
    drop(pinned);
    let _retired = collector.execute(&gc)?;
    drop(collector);
    let mut after = Vec::new();
    let _ = store(sandbox.path())?.reconstruct(anchored.target, &mut after)?;
    assert_eq!(after, output);
    sandbox.remove()?;
    Ok(())
}

#[test]
fn identical_views_yield_identical_receipts_across_reopen() -> Result<(), Box<dyn Error>> {
    let contents: [&[u8]; 2] = [b"same view, same receipt", b"other"];
    let (sandbox, published) = durable_store("durable-identical", &contents)?;
    let anchored = published.first().ok_or("published")?;
    let first = DurableSnapshot::open(
        sandbox.path(),
        catalog_policy()?,
        ReaderAttemptLimit::DEFAULT,
    )?;
    let mut one = Vec::new();
    let receipt_one = first.reconstruct(anchored.target, &mut one)?;
    drop(first);
    let second = store(sandbox.path())?.snapshot()?;
    let mut two = Vec::new();
    let receipt_two = second.reconstruct(anchored.target, &mut two)?;
    assert_eq!(receipt_one, receipt_two);
    assert_eq!(one, two);
    assert_eq!(
        receipt_one.view().verification_view(),
        receipt_two.view().verification_view()
    );
    sandbox.remove()?;
    Ok(())
}

#[test]
fn a_failing_writer_returns_no_receipt_and_the_exact_accepted_prefix() -> Result<(), Box<dyn Error>>
{
    struct Refusing {
        accepted: usize,
        after: usize,
    }
    impl std::io::Write for Refusing {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.accepted >= self.after {
                return Err(std::io::Error::other("output refused"));
            }
            let take = bytes.len().min(self.after.saturating_sub(self.accepted));
            self.accepted = self.accepted.saturating_add(take);
            Ok(take)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let contents: [&[u8]; 2] = [b"a receipt never precedes its bytes", b"x"];
    let (sandbox, published) = durable_store("durable-output", &contents)?;
    let anchored = published.first().ok_or("published")?;
    let snapshot = store(sandbox.path())?.snapshot()?;
    let mut output = Refusing {
        accepted: 0,
        after: 5,
    };
    let error = snapshot
        .reconstruct(anchored.target, &mut output)
        .err()
        .ok_or("a refusing writer received a receipt")?;
    assert!(
        matches!(
            &error,
            DurableReadError::Reconstruction(source)
                if matches!(**source, ReconstructionError::Write { .. })
        ),
        "{error:?}"
    );
    assert_eq!(output.accepted, 5);
    sandbox.remove()?;
    Ok(())
}
