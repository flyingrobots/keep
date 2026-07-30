//! This boundary module owns post-integrity retention header admission.

use super::RetentionRootDecodeError;
use super::root_header_decoder::DecodedRootHeader;
use crate::{
    RegisteredRetentionProfile, RetentionClosureLimits, RetentionRoot, RetentionRootDigest,
    RootGeneration,
};

pub(super) struct AdmittedRootHeader {
    pub(super) generation: RootGeneration,
    pub(super) profile: RegisteredRetentionProfile,
    pub(super) limits: RetentionClosureLimits,
    pub(super) predecessor: Option<RetentionRootDigest>,
}

pub(super) fn admit(
    header: &DecodedRootHeader,
) -> Result<AdmittedRootHeader, RetentionRootDecodeError> {
    if header.anchor_count > RetentionRoot::MAXIMUM_ANCHOR_COUNT {
        return Err(RetentionRootDecodeError::AnchorCountExceeded {
            maximum: RetentionRoot::MAXIMUM_ANCHOR_COUNT,
            observed: header.anchor_count,
        });
    }
    let generation = RootGeneration::new(header.generation)
        .map_err(|source| RetentionRootDecodeError::Generation { source })?;
    let profile = RegisteredRetentionProfile::admit(
        header.profile_identity,
        header.profile_version,
        header.profile_digest,
    )
    .map_err(|source| RetentionRootDecodeError::Profile { source })?;
    let limits = RetentionClosureLimits::new(
        header.closure_nodes,
        header.closure_depth,
        header.closure_encoded_bytes,
        header.closure_physical_bytes,
    )
    .map_err(|source| RetentionRootDecodeError::ClosureLimit { source })?;
    Ok(AdmittedRootHeader {
        generation,
        profile,
        limits,
        predecessor: predecessor(header.predecessor),
    })
}

fn predecessor(bytes: [u8; 32]) -> Option<RetentionRootDigest> {
    if bytes == [0_u8; 32] {
        None
    } else {
        Some(RetentionRootDigest::from_hash(bytes))
    }
}
