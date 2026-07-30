//! This module owns reconstruction of append state from a reusable prefix.

use std::collections::HashSet;

use super::segment_digest_builder::SegmentDigestBuilder;
use super::segment_record_cursor::SegmentRecordCursor;
use super::{
    RecoverySegmentResumeRequest, SegmentHeader, SegmentReadError, SegmentRecordIdentity,
    SegmentRecordLimit,
};

pub(super) struct RecoverySegmentResumeState {
    pub(super) digest: SegmentDigestBuilder,
    pub(super) identities: HashSet<SegmentRecordIdentity>,
    pub(super) record_limit: SegmentRecordLimit,
    pub(super) record_count: u32,
    pub(super) bytes_written: u64,
}

impl RecoverySegmentResumeState {
    pub(super) fn rebuild(
        encoded: &[u8],
        request: RecoverySegmentResumeRequest,
    ) -> Result<Self, SegmentReadError> {
        let records =
            encoded
                .get(SegmentHeader::ENCODED_LENGTH..)
                .ok_or(SegmentReadError::WrongLength {
                    minimum: SegmentHeader::ENCODED_LENGTH,
                    observed: encoded.len(),
                })?;
        let mut identities = reserve_identities(request.record_count())?;
        let mut cursor =
            SegmentRecordCursor::new(records, request.record_count(), request.policy());
        while let Some(located) = cursor.next_record()? {
            let _inserted = identities.insert(located.record.identity());
        }
        cursor.finish()?;
        let mut digest = SegmentDigestBuilder::new();
        digest.update(encoded);
        Ok(Self {
            digest,
            identities,
            record_limit: request.record_limit(),
            record_count: request.record_count(),
            bytes_written: request.length().get(),
        })
    }
}

fn reserve_identities(
    record_count: u32,
) -> Result<HashSet<SegmentRecordIdentity>, SegmentReadError> {
    let capacity = usize::try_from(record_count).map_err(|_source| {
        SegmentReadError::RecordCountHostWidth {
            observed: record_count,
        }
    })?;
    let mut identities = HashSet::new();
    identities.try_reserve(capacity).map_err(|source| {
        SegmentReadError::IdentityIndexAllocation {
            record_count,
            source,
        }
    })?;
    Ok(identities)
}
