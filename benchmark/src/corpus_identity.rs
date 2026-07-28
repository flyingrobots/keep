//! Stable identity and aggregate accounting for corpus members.

use keep::BlobId;

use crate::{BenchmarkCorpus, CorpusError};

pub(super) const fn member_names() -> [&'static str; BenchmarkCorpus::MEMBER_COUNT] {
    [
        "large-text",
        "large-binary",
        "edit-base",
        "early-insertion",
        "early-deletion",
        "near-neighbor",
        "zero-dedup",
    ]
}

pub(super) fn identify_members(
    members: [&[u8]; BenchmarkCorpus::MEMBER_COUNT],
    names: [&'static str; BenchmarkCorpus::MEMBER_COUNT],
) -> Result<[BlobId; BenchmarkCorpus::MEMBER_COUNT], CorpusError> {
    let mut identities = [BlobId::hash_bytes(&[]).map_err(|source| CorpusError::Identity {
        member: "empty-initializer",
        source,
    })?; BenchmarkCorpus::MEMBER_COUNT];
    for ((identity, member), name) in identities.iter_mut().zip(members).zip(names) {
        *identity = BlobId::hash_bytes(member).map_err(|source| CorpusError::Identity {
            member: name,
            source,
        })?;
    }
    Ok(identities)
}

pub(super) fn total_bytes(
    members: [&[u8]; BenchmarkCorpus::MEMBER_COUNT],
    tiny_blobs: &[Box<[u8]>],
) -> Result<usize, CorpusError> {
    members
        .into_iter()
        .map(<[u8]>::len)
        .chain(tiny_blobs.iter().map(|blob| blob.len()))
        .try_fold(0_usize, |total, length| {
            total
                .checked_add(length)
                .ok_or(CorpusError::TotalLengthOverflow)
        })
}
