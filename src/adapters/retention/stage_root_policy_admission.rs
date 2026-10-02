//! This boundary module owns admission of complete policy groups in interrupted roots.

use super::RetentionRootDecodeError;
use super::root_field_decoder::{read_array, read_u16, read_u32, read_u64};
use crate::{RegisteredRetentionProfile, RetentionClosureLimits};

pub(super) fn admit(encoded: &[u8]) -> Result<(), RetentionRootDecodeError> {
    if encoded.len() >= 88 {
        let _profile = RegisteredRetentionProfile::admit(
            read_u32(encoded, 48)?,
            read_u32(encoded, 52)?,
            read_array(encoded, 56)?,
        )
        .map_err(|source| RetentionRootDecodeError::Profile { source })?;
    }
    if encoded.len() >= 116 {
        let _limits = RetentionClosureLimits::new(
            read_u64(encoded, 88)?,
            read_u16(encoded, 96)?,
            read_u64(encoded, 100)?,
            read_u64(encoded, 108)?,
        )
        .map_err(|source| RetentionRootDecodeError::ClosureLimit { source })?;
    }
    Ok(())
}
