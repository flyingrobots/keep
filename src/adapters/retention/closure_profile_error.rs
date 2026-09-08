//! This module owns retention mapping for storage-profile replay failures.

use crate::profile::StorageProfileVerificationError;
use crate::{LayoutId, RetentionClosureVerificationError};

pub(super) const fn map(
    layout: LayoutId,
    error: StorageProfileVerificationError,
) -> RetentionClosureVerificationError {
    match error {
        StorageProfileVerificationError::Unsupported { profile } => {
            RetentionClosureVerificationError::ProfileVerifierUnavailable { layout, profile }
        }
        StorageProfileVerificationError::Chunking { source } => {
            RetentionClosureVerificationError::ProfileChunking { layout, source }
        }
        StorageProfileVerificationError::BoundaryMismatch {
            index,
            expected,
            observed,
        } => RetentionClosureVerificationError::ProfileBoundaryMismatch {
            layout,
            index,
            expected,
            observed,
        },
    }
}
