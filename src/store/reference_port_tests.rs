//! The generic port laws run against the non-durable reference backend.

use std::error::Error;
use std::io::Cursor;

use super::port_laws::{absence_refuses, ranges_exactly, reconstructs_exactly};
use super::{ContentReads, ContentStaging, StagedByteLimit, StagedContent, StagingLimits};
use crate::{BlobHasher, IngestionError, LayoutEntryLimit, ReferenceStore, ReferenceStoreCapacity};

fn content() -> Vec<u8> {
    let mut state = 0x2545_f491_4f6c_dd1d_u64;
    (0..192 * 1024)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            u8::try_from(state & 0xff).unwrap_or_default()
        })
        .collect()
}

#[test]
fn reference_backend_satisfies_the_read_laws_through_the_port() -> Result<(), Box<dyn Error>> {
    let content = content();
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(4 * 1024 * 1024));
    let staged = ContentStaging::stage(
        &mut store,
        &mut Cursor::new(&content),
        StagingLimits::entries(LayoutEntryLimit::MAXIMUM),
    )?;
    let receipt = StagedContent::commit(staged)?;
    reconstructs_exactly(&store, receipt.target(), receipt.layout_id(), &content)?;
    ranges_exactly(
        &store,
        receipt.target(),
        &content,
        &[(0, 1), (65_000, 5_000), (150_000, 42 * 1024)],
    )?;
    absence_refuses(&store, BlobHasher::new().finish());
    Ok(())
}

#[test]
fn the_byte_limit_refuses_before_any_excess_is_materialized() -> Result<(), Box<dyn Error>> {
    let content = content();
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(4 * 1024 * 1024));
    let limit = StagedByteLimit::new(u64::try_from(content.len())?.saturating_sub(1));
    let refusal = ContentStaging::stage(
        &mut store,
        &mut Cursor::new(&content),
        StagingLimits::new(LayoutEntryLimit::MAXIMUM, limit),
    );
    let Err(IngestionError::ByteLimitExceeded {
        limit: observed,
        accepted,
        incoming,
    }) = refusal
    else {
        return Err("expected a byte-limit refusal".into());
    };
    assert_eq!(observed, limit);
    assert!(accepted <= limit.get());
    assert!(accepted.saturating_add(u64::try_from(incoming)?) > limit.get());
    let exact = StagedByteLimit::new(u64::try_from(content.len())?);
    let staged = ContentStaging::stage(
        &mut store,
        &mut Cursor::new(&content),
        StagingLimits::new(LayoutEntryLimit::MAXIMUM, exact),
    )?;
    let target = StagedContent::target(&staged);
    drop(staged);
    assert!(!ContentReads::contains_blob(&store, target));
    Ok(())
}
