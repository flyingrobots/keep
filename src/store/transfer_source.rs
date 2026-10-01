//! This port module owns the pull side: a source that streams one
//! layout's chunks, authenticated as they are served, into a consumer.

use std::error::Error;
use std::fmt;
use std::io::Read;

use crate::{BlobId, ChunkId, LayoutId};

pub(super) mod sealed {
    pub trait Sealed {}
}

/// The consumer of one streamed layout: the blob it identifies and a
/// reader serving its bytes in order.
pub type StreamConsumer<'call> = &'call mut dyn FnMut(BlobId, &mut dyn Read);

/// A view that can stream one committed layout's chunks without
/// materializing the blob.
///
/// Implemented by `ReferenceStore` and `DurableSnapshot`. The reader
/// authenticates each chunk against its identity as it is first served;
/// the complete identity is the consumer's to verify, which
/// `ContentStaging::stage_expected` does.
pub trait TransferSource: sealed::Sealed {
    /// Streams `layout_id` into `consume`, called exactly once.
    ///
    /// # Errors
    ///
    /// Returns [`TransferSourceError`] when the layout is absent, the view
    /// refuses, or a chunk refused while being served; in the last case
    /// the consumer already observed the read failure.
    fn stream_layout(
        &self,
        layout_id: LayoutId,
        consume: StreamConsumer<'_>,
    ) -> Result<(), TransferSourceError>;
}

/// Why a source did not stream, or stopped streaming, a layout.
#[derive(Debug)]
pub enum TransferSourceError {
    /// The view names no such layout.
    LayoutMissing {
        /// The requested layout.
        requested: LayoutId,
    },
    /// The view could not be read.
    View(Box<dyn Error + 'static>),
    /// The view holds no chunk under the identity.
    ChunkMissing {
        /// The layout being served.
        layout: LayoutId,
        /// The entry index.
        index: usize,
        /// The absent chunk.
        requested: ChunkId,
    },
    /// The chunk's bytes do not hash to its identity.
    ChunkIdentityMismatch {
        /// The layout being served.
        layout: LayoutId,
        /// The entry index.
        index: usize,
        /// The identity the layout names.
        expected: ChunkId,
        /// The identity the bytes have.
        observed: ChunkId,
    },
    /// The chunk could not be hashed.
    ChunkHash {
        /// The layout being served.
        layout: LayoutId,
        /// The entry index.
        index: usize,
        /// The exact failure.
        source: crate::ChunkHashError,
    },
}

impl fmt::Display for TransferSourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LayoutMissing { .. } => formatter.write_str("the view names no such layout"),
            Self::View(source) => write!(formatter, "the view refused: {source}"),
            Self::ChunkMissing { index, .. } => {
                write!(formatter, "chunk {index} is absent from the view")
            }
            Self::ChunkIdentityMismatch { index, .. } => {
                write!(formatter, "chunk {index} does not hash to its identity")
            }
            Self::ChunkHash { index, .. } => write!(formatter, "chunk {index} could not be hashed"),
        }
    }
}

impl Error for TransferSourceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::View(source) => Some(source.as_ref()),
            Self::ChunkHash { source, .. } => Some(source),
            Self::LayoutMissing { .. }
            | Self::ChunkMissing { .. }
            | Self::ChunkIdentityMismatch { .. } => None,
        }
    }
}
