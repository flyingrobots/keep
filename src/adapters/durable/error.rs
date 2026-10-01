//! This boundary module owns typed durable-read failures, keeping evidenced
//! refusal apart from operational failure as the reconstruction contract
//! requires.

use std::error::Error;
use std::fmt;
use std::io;

use crate::adapters::{CatalogRestartError, FilesystemRetentionSnapshotError};
use crate::{
    BlobId, LayoutDecodeError, LayoutId, RangeReadError, ReconstructionError,
    RetentionNamespaceDigest,
};

/// Why a durable store or snapshot could not be opened.
#[derive(Debug)]
pub enum DurableStoreError {
    /// The root did not admit as a version-two store, the fence could not
    /// be taken, or one consistent view could not be collected.
    Snapshot(Box<FilesystemRetentionSnapshotError>),
    /// A retained root the manifest selects could not be read or decoded.
    RetainedRoot {
        /// The namespace whose root refused.
        namespace: RetentionNamespaceDigest,
        /// The exact refusal.
        source: io::Error,
    },
}

impl fmt::Display for DurableStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Snapshot(_) => formatter.write_str("the durable view could not be pinned"),
            Self::RetainedRoot { .. } => formatter.write_str("a retained root refused"),
        }
    }
}

impl Error for DurableStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Snapshot(source) => Some(source.as_ref()),
            Self::RetainedRoot { source, .. } => Some(source),
        }
    }
}

/// Why one durable read did not return a receipt.
///
/// `View` is the one operational failure: the pinned catalog could not be
/// re-admitted, so nothing about content follows. Every other variant is an
/// evidenced refusal against the complete pinned view, or, inside the
/// reference read errors, the output failure those errors already keep
/// distinct.
#[derive(Debug)]
pub enum DurableReadError {
    /// The pinned catalog could not be re-admitted for this read.
    View(Box<CatalogRestartError>),
    /// No retained root anchors the blob in this view.
    BlobMissing {
        /// The requested blob.
        requested: BlobId,
    },
    /// The catalog names no layout record under the identity.
    LayoutMissing {
        /// The requested layout.
        requested: LayoutId,
    },
    /// The committed layout record does not decode as the identity that
    /// names it.
    LayoutDecode(LayoutDecodeError),
    /// The reconstruction core refused or its output failed.
    Reconstruction(Box<ReconstructionError>),
    /// The range core refused or its output failed.
    RangeRead(Box<RangeReadError>),
}

impl fmt::Display for DurableReadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::View(_) => formatter.write_str("the pinned view could not be re-admitted"),
            Self::BlobMissing { .. } => formatter.write_str("no retained root anchors the blob"),
            Self::LayoutMissing { .. } => formatter.write_str("the catalog names no such layout"),
            Self::LayoutDecode(source) => write!(formatter, "committed layout refused: {source}"),
            Self::Reconstruction(source) => write!(formatter, "reconstruction: {source}"),
            Self::RangeRead(source) => write!(formatter, "range read: {source}"),
        }
    }
}

impl Error for DurableReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::View(source) => Some(source.as_ref()),
            Self::LayoutDecode(source) => Some(source),
            Self::Reconstruction(source) => Some(source.as_ref()),
            Self::RangeRead(source) => Some(source.as_ref()),
            Self::BlobMissing { .. } | Self::LayoutMissing { .. } => None,
        }
    }
}
