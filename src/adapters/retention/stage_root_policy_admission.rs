//! This boundary module owns admission of policy groups and partial profile fields in interrupted roots.

use super::RetentionRootDecodeError;
use super::root_field_decoder::{read_array, read_u32};
use crate::RegisteredRetentionProfile;

pub(super) fn admit(encoded: &[u8]) -> Result<(), RetentionRootDecodeError> {
    if encoded.len() < 88 {
        admit_profile_prefix(encoded)?;
    }
    if encoded.len() >= 88 {
        let _profile = RegisteredRetentionProfile::admit(
            read_u32(encoded, 48)?,
            read_u32(encoded, 52)?,
            read_array(encoded, 56)?,
        )
        .map_err(|source| RetentionRootDecodeError::Profile { source })?;
    }
    super::stage_closure_limit_admission::admit(encoded)
}

fn admit_profile_prefix(encoded: &[u8]) -> Result<(), RetentionRootDecodeError> {
    // The registry is currently closed to this one exact profile; unavailable
    // bytes remain unknown, while every available byte must admit a completion.
    let profile = RegisteredRetentionProfile::SINGLE_CANONICAL_WITNESS_V1;
    super::stage_fixed_field_admission::admit(
        encoded,
        &[
            (48, &profile.identity().to_be_bytes()),
            (52, &profile.version().to_be_bytes()),
            (56, profile.digest()),
        ],
        |offset, expected, observed| RetentionRootDecodeError::PrefixByteMismatch {
            offset,
            expected,
            observed,
        },
    )
}
