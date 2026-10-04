//! Medium filesystem law: oversized recovery evidence is refused before materialization.
//! Oracle: the fixed head format bound and measured allocation below the rejected file size.
//! Delete if recovery is removed or a stronger generated memory-bound law subsumes this one.

#![cfg(target_os = "linux")]

#[allow(dead_code, reason = "shared fixture serves independent runtime laws")]
#[expect(
    clippy::redundant_pub_crate,
    reason = "shared fixture has several module depths"
)]
#[path = "golden_file_worldline/durable_fixture.rs"]
mod durable_fixture;
#[allow(dead_code, reason = "shared sandbox also offers explicit teardown")]
#[expect(
    clippy::redundant_pub_crate,
    reason = "shared sandbox has several module depths"
)]
#[path = "segment_filesystem_stage/sandbox.rs"]
mod durable_sandbox;

use keep::{RecoveryStage, RecoveryStageMetadataError, recover_compaction};
use std::{error::Error, fs};

#[test]
fn oversized_head_is_refused_without_allocating_its_contents() -> Result<(), Box<dyn Error>> {
    let store = durable_fixture::build("compaction-head-memory", &[b"retained"])?;
    let path = store.path().join("head.next");
    let file = fs::File::create(&path)?;
    let length = 16_u64 * 1024 * 1024;
    file.set_len(length)?;
    let head = fs::read(store.path().join("HEAD"))?;
    let policy = durable_fixture::policy()?;
    let mut result = None;

    let allocation = allocation_counter::measure(|| {
        result = Some(recover_compaction(store.path(), policy));
    });

    let error = result
        .ok_or("recovery did not run")?
        .err()
        .ok_or("oversized head admitted")?;
    let mut cause: &(dyn Error + 'static) = &error;
    while cause.downcast_ref::<RecoveryStageMetadataError>().is_none() {
        cause = cause.source().ok_or("metadata refusal cause missing")?;
    }
    assert_eq!(
        cause.downcast_ref::<RecoveryStageMetadataError>(),
        Some(&RecoveryStageMetadataError::Oversized {
            stage: RecoveryStage::NextHead,
            maximum: RecoveryStage::NextHead.maximum_length(),
            observed: length,
        })
    );
    assert!(
        allocation.bytes_max < length,
        "oversized evidence must be refused before content allocation: peak={} file={length}",
        allocation.bytes_max
    );
    assert_eq!(
        fs::metadata(&path)?.len(),
        length,
        "refusal preserves oversized evidence"
    );
    assert_eq!(
        fs::read(store.path().join("HEAD"))?,
        head,
        "refusal preserves publication"
    );
    Ok(())
}
