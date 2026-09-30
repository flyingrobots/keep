//! This boundary module owns one bounded, admitting read of the segment pool.

use std::io::Read;

use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::fs::{Dir, OpenOptions};

use super::GcLivenessObservationError as Error;
use crate::adapters::{AdmittedSegment, SegmentDigest, SegmentReadPolicy};

const SUFFIX: &str = ".seg";
const DIGEST_HEX_LENGTH: usize = 64;

/// Reads every `segments/` entry, requires each to be a regular file named
/// by the lowercase digest it hashes to, admits it under `policy`, and
/// returns the sorted `(digest, length)` inventory. Total bytes read stay
/// within `byte_limit`; an unknown entry, a wrong kind, a name that is not a
/// digest, or a segment whose bytes do not admit refuses the whole read.
pub(super) fn read(
    segments: &Dir,
    policy: SegmentReadPolicy,
    byte_limit: u64,
) -> Result<Vec<(SegmentDigest, u64)>, Error> {
    let mut inventory = Vec::new();
    let mut read_bytes = 0_u64;
    for entry in segments
        .entries()
        .map_err(|source| Error::pool("list", source))?
    {
        let entry = entry.map_err(|source| Error::pool("read entry", source))?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_name| Error::PoolEntryName)?;
        let expected = parse_name(&name)?;
        let metadata = segments
            .symlink_metadata(&name)
            .map_err(|source| Error::pool("inspect entry", source))?;
        if !metadata.is_file() {
            return Err(Error::PoolEntryKind { segment: expected });
        }
        read_bytes = read_bytes
            .checked_add(metadata.len())
            .filter(|total| *total <= byte_limit)
            .ok_or(Error::PoolByteLimit { limit: byte_limit })?;
        admit(segments, &name, expected, metadata.len(), policy)?;
        inventory.push((expected, metadata.len()));
    }
    inventory.sort_unstable();
    Ok(inventory)
}

fn parse_name(name: &str) -> Result<SegmentDigest, Error> {
    let hex = name.strip_suffix(SUFFIX).ok_or(Error::PoolEntryName)?;
    if hex.len() != DIGEST_HEX_LENGTH {
        return Err(Error::PoolEntryName);
    }
    let mut bytes = [0_u8; 32];
    for (index, byte) in bytes.iter_mut().enumerate() {
        let start = index.checked_mul(2).ok_or(Error::PoolEntryName)?;
        let end = start.checked_add(2).ok_or(Error::PoolEntryName)?;
        let pair = hex.get(start..end).ok_or(Error::PoolEntryName)?;
        if pair.bytes().any(|c| c.is_ascii_uppercase()) {
            return Err(Error::PoolEntryName);
        }
        *byte = u8::from_str_radix(pair, 16).map_err(|_source| Error::PoolEntryName)?;
    }
    Ok(SegmentDigest::from_validated(bytes))
}

/// Reads and admits one named pool entry, requiring it to hash to
/// `expected`; execution calls this before unlinking a candidate.
pub(super) fn admit_entry(
    segments: &Dir,
    name: &str,
    expected: SegmentDigest,
    length: u64,
    policy: SegmentReadPolicy,
) -> Result<(), Error> {
    admit(segments, name, expected, length, policy)
}

fn admit(
    segments: &Dir,
    name: &str,
    expected: SegmentDigest,
    length: u64,
    policy: SegmentReadPolicy,
) -> Result<(), Error> {
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No).nonblock(true);
    let mut file = segments
        .open_with(name, &options)
        .map_err(|source| Error::pool("open segment", source))?;
    let capacity =
        usize::try_from(length).map_err(|_source| Error::PoolByteLimit { limit: u64::MAX })?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(capacity)
        .map_err(|_source| Error::PoolByteLimit { limit: length })?;
    file.read_to_end(&mut bytes)
        .map_err(|source| Error::pool("read segment", source))?;
    let admitted =
        AdmittedSegment::decode(&bytes, policy).map_err(|source| Error::SegmentAdmission {
            segment: expected,
            source: Box::new(source),
        })?;
    if admitted.digest() == expected {
        Ok(())
    } else {
        Err(Error::SegmentDigestMismatch {
            expected,
            observed: admitted.digest(),
        })
    }
}
