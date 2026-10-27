//! Interrupted version-two publication and its stage recovery.
//!
//! Each law fails the real version-two writer at one storage-port boundary,
//! then recovers with the public version-two recovery tools and requires the
//! store to read both blobs exactly with every retained root preserved.

#[path = "../../tests/golden_file_worldline/durable_fixture.rs"]
#[allow(
    dead_code,
    reason = "interruption laws use complete stores; golden laws cover partial records"
)]
mod durable_fixture;
#[path = "../../tests/version_two_catalog_writer/recovery_fixture.rs"]
#[allow(
    dead_code,
    reason = "interruption laws finalize and complete; the public suite also inspects roles"
)]
mod recovery_fixture;
#[path = "../../tests/support/successor_catalog.rs"]
#[allow(
    dead_code,
    reason = "interruption laws publish and republish; the public suite also stages by hand"
)]
mod successor_catalog;
#[path = "../../tests/version_two_catalog_writer/writer_fixture.rs"]
#[allow(
    dead_code,
    reason = "interruption laws ingest and retain; the public suite also stages by hand"
)]
mod writer_fixture;

use std::io;
use std::path::Path;

use super::filesystem_test_sandbox as durable_sandbox;
use super::populated_store;

use crate::{CatalogPublicationError, CatalogPublicationPhase, RetainedCatalogPublicationError};

use super::filesystem_version_two_catalog_stages::CatalogPublicationBoundary;
use recovery_fixture::{complete, discard, finalize, stage_roles};
use writer_fixture::{
    Predecessor, TestResult, assert_reads, first_blob, open_publisher, pool_segments, publish,
    republish, retain, second_blob,
};

const INJECTED: &str = "injected version-two publication interruption";

/// Publishes the second blob, failing after `phase` completes.
fn interrupted_after(root: &Path, phase: CatalogPublicationPhase) -> TestResult<()> {
    let mut publisher = open_publisher(root)?;
    publisher
        .stages
        .observe(Box::new(move |observed, boundary| {
            if observed == phase && boundary == CatalogPublicationBoundary::After {
                Err(io::Error::other(INJECTED))
            } else {
                Ok(())
            }
        }));
    let outcome = publish(&mut publisher, root, &second_blob(), Predecessor::Kept)?;
    match outcome {
        Err(RetainedCatalogPublicationError::Catalog {
            source: CatalogPublicationError::Storage { phase: failed, .. },
        }) if failed == phase => Ok(()),
        other => Err(format!("expected interruption at {phase}: {other:?}").into()),
    }
}

/// Requires both blobs to read back once the second blob is retained.
fn assert_both_read(root: &Path) -> TestResult<()> {
    let first = first_blob();
    assert_reads(root, &first)?;
    retain(root, b"second", &second_blob())?;
    assert_reads(root, &first)?;
    assert_reads(root, &second_blob())
}

#[test]
fn a_head_stage_interrupted_after_synchronization_is_finalized_through_the_gate() -> TestResult<()>
{
    let sandbox = durable_fixture::build("v2-interrupt-head", &[&first_blob()])?;
    let root = sandbox.path();
    interrupted_after(root, CatalogPublicationPhase::SynchronizeHead)?;
    assert_eq!(
        stage_roles(root)?,
        [crate::RecoveryEntryRole::NextHeadStage]
    );
    assert_reads(root, &first_blob())?;

    let candidate = pool_segments(root)?;
    let _receipt = finalize(root, &candidate)??;
    assert!(stage_roles(root)?.is_empty());
    assert_both_read(root)
}

#[test]
fn a_linked_segment_stage_is_completed_and_the_catalog_republished() -> TestResult<()> {
    let sandbox = durable_fixture::build("v2-interrupt-segment", &[&first_blob()])?;
    let root = sandbox.path();
    interrupted_after(root, CatalogPublicationPhase::LinkSegment)?;
    assert_eq!(stage_roles(root)?, [crate::RecoveryEntryRole::SegmentStage]);

    let _receipt = complete(root, crate::RecoveryStage::Segment)?;
    assert!(stage_roles(root)?.is_empty());
    let mut publisher = open_publisher(root)?;
    let _published = republish(&mut publisher, root)??;
    drop(publisher);
    assert_both_read(root)
}

#[test]
fn an_empty_catalog_stage_is_discarded_and_the_catalog_republished() -> TestResult<()> {
    let sandbox = durable_fixture::build("v2-interrupt-catalog", &[&first_blob()])?;
    let root = sandbox.path();
    interrupted_after(root, CatalogPublicationPhase::CreateCatalogStage)?;
    assert_eq!(stage_roles(root)?, [crate::RecoveryEntryRole::CatalogStage]);

    let _receipt = discard(root, crate::RecoveryStage::Catalog)?;
    assert!(stage_roles(root)?.is_empty());
    let mut publisher = open_publisher(root)?;
    let _published = republish(&mut publisher, root)??;
    drop(publisher);
    assert_both_read(root)
}
