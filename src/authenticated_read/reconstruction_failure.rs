//! This module owns semantic whole-blob authentication and emission failures.

use super::{ChunkVerificationError, OutputWriteError};
use crate::profile::StorageProfileVerificationError;
use crate::{BlobHashError, BlobId, BlobLength, LayoutId};

#[derive(Debug)]
pub(crate) enum ReconstructionFailure {
    Chunk(ChunkVerificationError),
    BlobHash(BlobHashError),
    BlobIdentityMismatch {
        layout: LayoutId,
        expected: BlobId,
        observed: BlobId,
    },
    Profile {
        layout: LayoutId,
        source: StorageProfileVerificationError,
    },
    Output {
        layout: LayoutId,
        source: OutputWriteError,
    },
    WrittenLengthMismatch {
        layout: LayoutId,
        expected: BlobLength,
        observed: BlobLength,
    },
}
