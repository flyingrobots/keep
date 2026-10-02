//! This module owns durable public read laws against the independent one-zero corpus.
//!
//! Size: medium (owned filesystem scratch, no network or sleeps).
//! Oracle: the frozen one-zero bundle and root identify the single byte [0].
//! Delete when durable reads are removed or stronger public laws subsume these outcomes.

use std::error::Error;

use super::filesystem_retention_test_fixture::{
    HEAD_HEX, ROOT_HEX, fixture, initial_preparation, open_authority,
};
use crate::{
    AdmittedRetentionRoot, ByteLength, ByteOffset, ByteRange, CatalogRestartByteLimit,
    CatalogRestartPolicy, DurableReadError, DurableStore, LayoutEntryLimit, ReaderAttemptLimit,
    SegmentReadPolicy, SegmentRecordLimit, execute_retention_publication,
};

fn store(root: &std::path::Path) -> Result<DurableStore, Box<dyn Error>> {
    Ok(DurableStore::open(
        root,
        CatalogRestartPolicy::new(
            SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM),
            CatalogRestartByteLimit::new(1_048_576)?,
        ),
        ReaderAttemptLimit::DEFAULT,
    ))
}

#[test]
fn durable_reconstruction_returns_golden_bytes_with_exact_view_coordinates()
-> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("durable-golden-reconstruction")?;
    let bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&bytes)?;
    let _published = execute_retention_publication(&mut authority, &preparation)?;
    drop(authority);
    let root = AdmittedRetentionRoot::decode(&bytes)?;
    let anchor = root
        .root()
        .anchors()
        .first()
        .ok_or("golden anchor absent")?;
    let snapshot = store(sandbox.path())?.snapshot()?;
    let mut output = Vec::new();
    let receipt = snapshot.reconstruct(anchor.blob_id(), &mut output)?;
    assert_eq!(
        output,
        [0],
        "reconstruction must emit the exact golden byte"
    );
    assert_eq!(receipt.receipt().target(), anchor.blob_id());
    assert_eq!(receipt.receipt().layout_id(), anchor.layout_id());
    assert_eq!(receipt.receipt().bytes_written().get(), 1);
    assert_eq!(receipt.view().catalog_generation().get(), 1);
    assert_eq!(
        receipt.view().catalog_digest().as_bytes().as_slice(),
        fixture("0b7cad1b6de663d34beacbc214db7497f2e36ab6b08dfbd5febbc8d06a418811\n")?,
        "catalog coordinate must match the independently frozen bundle"
    );
    let golden_head = fixture(HEAD_HEX)?;
    assert_eq!(
        receipt
            .view()
            .retention()
            .ok_or("retention coordinates absent")?
            .manifest_digest()
            .as_bytes()
            .as_slice(),
        golden_head
            .get(40..72)
            .ok_or("golden manifest digest absent")?,
        "manifest coordinate must match the normative head field in the golden record"
    );
    assert_eq!(
        receipt
            .view()
            .retention()
            .ok_or("retention coordinates absent")?
            .generation(),
        preparation.liveness_generation()
    );
    assert_eq!(receipt.view(), snapshot.view());
    Ok(())
}

#[test]
fn durable_ranges_emit_only_the_requested_golden_interval() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("durable-golden-ranges")?;
    let bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&bytes)?;
    let _published = execute_retention_publication(&mut authority, &preparation)?;
    drop(authority);
    let root = AdmittedRetentionRoot::decode(&bytes)?;
    let anchor = root
        .root()
        .anchors()
        .first()
        .ok_or("golden anchor absent")?;
    let snapshot = store(sandbox.path())?.snapshot()?;
    for (offset, length, expected) in [(0, 0, &[][..]), (0, 1, &[0][..]), (1, 0, &[][..])] {
        let requested = ByteRange::new(ByteOffset::new(offset), ByteLength::new(length))?;
        let mut output = Vec::new();
        let receipt = snapshot.read_range(anchor.blob_id(), requested, &mut output)?;
        assert_eq!(
            output, expected,
            "range {requested:?} must emit its exact interval"
        );
        assert_eq!(receipt.receipt().requested(), requested);
        assert_eq!(receipt.receipt().bytes_written().get(), length);
        assert_eq!(receipt.view(), snapshot.view());
    }
    Ok(())
}

#[test]
fn unretained_blob_refuses_without_output_but_exact_catalog_layout_remains_readable()
-> Result<(), Box<dyn Error>> {
    let (sandbox, authority) = open_authority("durable-unretained-blob")?;
    drop(authority);
    let bytes = fixture(ROOT_HEX)?;
    let root = AdmittedRetentionRoot::decode(&bytes)?;
    let anchor = root
        .root()
        .anchors()
        .first()
        .ok_or("golden anchor absent")?;
    let snapshot = store(sandbox.path())?.snapshot()?;
    assert!(!snapshot.contains_blob(anchor.blob_id())?);
    let mut output = Vec::new();
    let failure = snapshot
        .reconstruct(anchor.blob_id(), &mut output)
        .err()
        .ok_or("unretained blob was read")?;
    assert!(
        matches!(failure, DurableReadError::BlobMissing { requested } if requested == anchor.blob_id())
    );
    assert!(output.is_empty(), "refusal must not emit content");
    let receipt = snapshot.reconstruct_layout(anchor.layout_id(), &mut output)?;
    assert_eq!(output, [0]);
    assert_eq!(receipt.view().retention(), None);
    Ok(())
}
