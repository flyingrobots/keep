//! This module owns lossless translation of semantic reconstruction failures.

use super::ReconstructionError;
use super::profile_error_mapping::profile_error;
use super::reconstruction_error_mapping::{
    reconstruction_chunk_error, reconstruction_output_error,
};
use crate::authenticated_read::ReconstructionFailure;

impl From<ReconstructionFailure> for ReconstructionError {
    fn from(failure: ReconstructionFailure) -> Self {
        match failure {
            ReconstructionFailure::Chunk(source) => reconstruction_chunk_error(source),
            ReconstructionFailure::BlobHash(source) => Self::BlobHash(source),
            ReconstructionFailure::BlobIdentityMismatch {
                layout,
                expected,
                observed,
            } => Self::BlobIdentityMismatch {
                layout,
                expected,
                observed,
            },
            ReconstructionFailure::Profile { layout, source } => profile_error(layout, source),
            ReconstructionFailure::Output { layout, source } => {
                reconstruction_output_error(layout, source)
            }
            ReconstructionFailure::WrittenLengthMismatch {
                layout,
                expected,
                observed,
            } => Self::WrittenLengthMismatch {
                layout,
                expected,
                observed,
            },
        }
    }
}
