//! This boundary module owns typed durable-read failures, keeping evidenced
//! refusal apart from operational failure as the reconstruction contract
//! requires.

use std::error::Error;
use std::fmt;

use crate::adapters::{CatalogRestartError, FilesystemRetentionSnapshotError};
use crate::{
    BlobId, LayoutDecodeError, LayoutId, RangeReadError, ReconstructionError,
    RetentionClosureVerificationError, RetentionNamespaceDigest, RetentionRootDecodeError,
};

/// Why a durable store or snapshot could not be opened.
#[derive(Debug)]
pub enum DurableStoreError {
    /// The root did not admit as a version-two store, the fence could not
    /// be taken, or one consistent view could not be collected.
    Snapshot(Box<FilesystemRetentionSnapshotError>),
    /// The pinned catalog could not be re-admitted.
    Catalog(Box<CatalogRestartError>),
    /// The manifest-selected root was unexpectedly absent.
    RootMissing {
        /// The namespace selected by the manifest.
        namespace: RetentionNamespaceDigest,
    },
    /// A selected root failed canonical admission.
    RootDecode {
        /// The namespace whose root refused.
        namespace: RetentionNamespaceDigest,
        /// The exact refusal.
        source: RetentionRootDecodeError,
    },
    /// A retained closure could not be proven against the pinned catalog.
    Closure {
        /// The namespace whose closure refused.
        namespace: RetentionNamespaceDigest,
        /// The exact closure refusal.
        source: Box<RetentionClosureVerificationError>,
    },
}

impl fmt::Display for DurableStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Snapshot(_) => formatter.write_str("the durable view could not be pinned"),
            Self::Catalog(_) => formatter.write_str("the pinned catalog refused"),
            Self::RootMissing { .. } => formatter.write_str("the selected root is absent"),
            Self::RootDecode { .. } => formatter.write_str("a retained root refused"),
            Self::Closure { .. } => formatter.write_str("a retained closure refused"),
        }
    }
}

impl Error for DurableStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Snapshot(source) => Some(source.as_ref()),
            Self::Catalog(source) => Some(source.as_ref()),
            Self::RootMissing { .. } => None,
            Self::RootDecode { source, .. } => Some(source),
            Self::Closure { source, .. } => Some(source.as_ref()),
        }
    }
}

/// Why one durable read did not return a receipt.
///
/// Source variants retain the exact admission, corruption or I/O boundary;
/// they must not all be interpreted as content corruption. Missing identities
/// are evidenced against the admitted view. Reconstruction and range errors
/// preserve output failures separately from content refusal.
#[derive(Debug)]
pub enum DurableReadError {
    /// Retained anchor evidence could not be read or admitted.
    Retention(Box<DurableStoreError>),
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
            Self::Retention(_) => formatter.write_str("the retained anchor evidence refused"),
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
            Self::Retention(source) => Some(source.as_ref()),
            Self::View(source) => Some(source.as_ref()),
            Self::LayoutDecode(source) => Some(source),
            Self::Reconstruction(source) => Some(source.as_ref()),
            Self::RangeRead(source) => Some(source.as_ref()),
            Self::BlobMissing { .. } | Self::LayoutMissing { .. } => None,
        }
    }
}
