//! This module owns canonical retention-closure transcript hashing.

use std::collections::BTreeMap;

use blake3::Hasher;

use crate::{
    AdmittedSegmentRecord, CatalogDigest, CatalogGeneration, RegisteredRetentionProfile,
    RetentionClosureDigest, RetentionClosureUsage, SegmentRecordIdentity,
};

use super::closure_member::ClosureMember;

const DOMAIN: &[u8] = b"keep.retention-closure/v2\0";

pub(super) fn calculate(
    profile: RegisteredRetentionProfile,
    catalog_generation: CatalogGeneration,
    catalog_digest: CatalogDigest,
    usage: RetentionClosureUsage,
    records: &BTreeMap<SegmentRecordIdentity, AdmittedSegmentRecord<'_>>,
) -> RetentionClosureDigest {
    let mut hasher = Hasher::new();
    hasher.update(DOMAIN);
    hasher.update(&profile.identity().to_be_bytes());
    hasher.update(&profile.version().to_be_bytes());
    hasher.update(profile.digest());
    hasher.update(&catalog_generation.get().to_be_bytes());
    hasher.update(catalog_digest.as_bytes());
    hasher.update(&usage.node_count().to_be_bytes());
    hasher.update(&usage.maximum_depth().to_be_bytes());
    hasher.update(&[0_u8; 6]);
    hasher.update(&usage.encoded_bytes().to_be_bytes());
    hasher.update(&usage.physical_bytes().to_be_bytes());
    for (identity, record) in records {
        hasher.update(ClosureMember::new(*identity, *record).as_bytes());
    }
    RetentionClosureDigest::from_verified(*hasher.finalize().as_bytes())
}
