//! This module owns lossless outward mapping of storage-profile failures.

use super::ReconstructionError;
use crate::LayoutId;
use crate::profile::StorageProfileVerificationError;

pub(super) const fn profile_error(
    layout: LayoutId,
    error: StorageProfileVerificationError,
) -> ReconstructionError {
    match error {
        StorageProfileVerificationError::Unsupported { profile } => {
            ReconstructionError::ProfileVerifierUnavailable { layout, profile }
        }
        StorageProfileVerificationError::Chunking { source } => {
            ReconstructionError::ProfileChunking { layout, source }
        }
        StorageProfileVerificationError::BoundaryMismatch {
            index,
            expected,
            observed,
        } => ReconstructionError::ProfileBoundaryMismatch {
            layout,
            index,
            expected,
            observed,
        },
    }
}
