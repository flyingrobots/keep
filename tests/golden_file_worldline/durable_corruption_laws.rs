//! Durable payload corruption is refused before any output.
//!
//! Size: medium. Oracle: the normative v1 record framing and independently
//! assembled checksum preimage, plus caller output preservation.
//! Delete when durable segment reads disappear or stronger corruption laws subsume this case.

use std::error::Error;
use std::fs;

use super::durable_fixture::{build, identify, policy};
use keep::{
    CatalogRestartError, DurableOutcome, DurableStore, DurableStoreError,
    FilesystemRetentionSnapshotError, ReaderAttemptLimit, SegmentReadError,
    SegmentRecordDecodeError,
};

#[test]
fn a_corrupt_selected_chunk_preserves_exact_record_refusal_before_output()
-> Result<(), Box<dyn Error>> {
    let source = b"one physical chunk";
    let sandbox = build("durable-corrupt-chunk", &[source])?;
    let target = identify(source)?.target;
    let segment_path = fs::read_dir(sandbox.path().join("segments"))?
        .next()
        .ok_or("segment absent")??
        .path();
    let mut bytes = fs::read(&segment_path)?;
    // The v1 grammar fixes the segment header at 64 bytes and record header at 112.
    let payload = 64_usize.checked_add(112).ok_or("payload offset overflow")?;
    let checksum_offset = payload
        .checked_add(source.len())
        .ok_or("checksum offset overflow")?;
    let checksum_end = checksum_offset
        .checked_add(32)
        .ok_or("checksum end overflow")?;
    let observed: [u8; 32] = bytes
        .get(checksum_offset..checksum_end)
        .ok_or("record checksum absent")?
        .try_into()?;
    *bytes.get_mut(payload).ok_or("payload absent")? ^= 1;
    let covered = bytes
        .get(64..checksum_offset)
        .ok_or("record preimage absent")?;
    let expected = record_checksum(covered)?;
    fs::write(segment_path, bytes)?;
    let store = DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT);
    let mut output = vec![0xAB];
    let failure = store
        .reconstruct(target, &mut output)
        .err()
        .ok_or("corrupt chunk reconstructed")?;
    assert!(
        matches!(&failure, DurableOutcome::Store(DurableStoreError::Snapshot(snapshot))
        if matches!(snapshot.as_ref(), FilesystemRetentionSnapshotError::Catalog { source: CatalogRestartError::Segment { source, .. } }
            if matches!(source.as_ref(), SegmentReadError::RecordDecode { record_index: 0, offset: 64,
                source: SegmentRecordDecodeError::ChecksumMismatch { expected: actual_expected, observed: actual_observed } }
                if actual_expected.as_bytes() == &expected && actual_observed.as_bytes() == &observed))),
        "payload corruption must retain exact record and checksum coordinates: {failure:?}"
    );
    assert_eq!(output, [0xAB]);
    Ok(())
}

fn record_checksum(covered: &[u8]) -> Result<[u8; 32], Box<dyn Error>> {
    let mut oracle = blake3::Hasher::new();
    oracle.update(b"KEEP:SEG:RECORD:SUM\0");
    oracle.update(&1_u16.to_be_bytes());
    oracle.update(&[1]);
    oracle.update(covered);
    oracle.update(&u64::try_from(covered.len())?.to_be_bytes());
    Ok(*oracle.finalize().as_bytes())
}
