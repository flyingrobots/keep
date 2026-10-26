//! This module owns the content-store port's generic read laws against a
//! durable snapshot of the independent one-zero corpus.
//!
//! Size: medium (owned filesystem scratch, no network or sleeps).
//! Oracle: the frozen one-zero bundle and root identify the single byte [0].
//! Delete when durable snapshots stop implementing the port or stronger
//! backend-neutral laws subsume these outcomes.

use std::error::Error;

use super::filesystem_retention_test_fixture::{
    ROOT_HEX, fixture, initial_preparation, open_authority,
};
use crate::store::port_laws::{
    RefusalOracle, absence_refuses, ranges_exactly, reconstructs_exactly,
};
use crate::{
    AdmittedRetentionRoot, BlobHasher, CatalogRestartByteLimit, CatalogRestartPolicy,
    DurableReadError, DurableSnapshot, LayoutEntryLimit, RangePlanError, RangeReadError,
    ReaderAttemptLimit, SegmentReadPolicy, SegmentRecordLimit, execute_retention_publication,
};

const DURABLE_ORACLE: RefusalOracle<DurableSnapshot> = RefusalOracle {
    past_end: |refusal| {
        matches!(refusal, DurableReadError::RangeRead(source)
            if matches!(source.as_ref(), RangeReadError::RangePlan(RangePlanError::OutOfBounds { .. })))
    },
    absent: |refusal| matches!(refusal, DurableReadError::BlobMissing { .. }),
};

#[test]
fn a_durable_snapshot_satisfies_the_generic_read_laws_through_the_port()
-> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("durable-port-read-laws")?;
    let bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&bytes)?;
    let _published = execute_retention_publication(&mut authority, &preparation)?;
    drop(authority);
    let root = AdmittedRetentionRoot::decode(&bytes)?;
    let anchor = *root
        .root()
        .anchors()
        .first()
        .ok_or("golden anchor absent")?;
    let snapshot = DurableSnapshot::open(
        sandbox.path(),
        CatalogRestartPolicy::new(
            SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM),
            CatalogRestartByteLimit::new(1_048_576)?,
        ),
        ReaderAttemptLimit::DEFAULT,
    )?;
    reconstructs_exactly(&snapshot, anchor.blob_id(), anchor.layout_id(), &[0])?;
    ranges_exactly(
        &snapshot,
        &DURABLE_ORACLE,
        anchor.blob_id(),
        &[0],
        &[(0, 1), (0, 0), (1, 0)],
    )?;
    absence_refuses(&snapshot, &DURABLE_ORACLE, BlobHasher::new().finish())?;
    Ok(())
}
