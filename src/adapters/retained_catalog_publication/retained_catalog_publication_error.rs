//! This module owns refusals of retained-closure catalog publication.

use std::error::Error;
use std::io;

use super::super::CatalogPublicationError;
use super::RetentionEvidencePhase;
use crate::{
    LivenessGeneration, RetentionClosureVerificationError, RetentionHead, RetentionHeadDecodeError,
    RetentionManifestDecodeError, RetentionManifestDigest, RetentionManifestEntry,
    RetentionManifestLength, RetentionRootDecodeError,
};

/// Failure before a candidate catalog that must preserve retention becomes current.
///
/// Retention evidence and closure refusals occur before the first catalog
/// storage call. [`Self::RetentionChanged`] and failures in
/// [`RetentionEvidencePhase::RevalidateHead`] occur after `head.next`
/// verification and before `HEAD` replacement. [`Self::Catalog`] preserves
/// the catalog protocol's own preflight refusal or storage phase.
#[derive(Debug)]
pub enum RetainedCatalogPublicationError {
    /// Candidate preflight or one catalog protocol transition failed.
    Catalog {
        /// Preserved catalog publication refusal.
        source: CatalogPublicationError,
    },
    /// A retention head or manifest read failed under writer authority.
    RetentionStorage {
        /// Evidence read attempted.
        phase: RetentionEvidencePhase,
        /// Preserved storage source.
        source: io::Error,
    },
    /// The root one manifest entry selects could not be read.
    RootStorage {
        /// Manifest entry whose root was requested.
        entry: RetentionManifestEntry,
        /// Preserved storage source.
        source: io::Error,
    },
    /// Retention head bytes failed canonical admission.
    HeadDecode {
        /// Evidence read whose bytes were refused.
        phase: RetentionEvidencePhase,
        /// Preserved head decoder refusal.
        source: RetentionHeadDecodeError,
    },
    /// The selected manifest's length differs from the length the head names.
    ManifestLength {
        /// Length the admitted head names.
        expected: RetentionManifestLength,
        /// Byte length storage returned.
        observed: usize,
    },
    /// The selected manifest failed canonical admission.
    ManifestDecode {
        /// Preserved manifest decoder refusal.
        source: Box<RetentionManifestDecodeError>,
    },
    /// The selected manifest's digest differs from the digest the head names.
    ManifestDigest {
        /// Digest the admitted head names.
        expected: RetentionManifestDigest,
        /// Digest of the admitted manifest bytes.
        observed: RetentionManifestDigest,
    },
    /// The selected manifest's generation differs from the head's generation.
    ManifestGeneration {
        /// Generation the admitted head names.
        expected: LivenessGeneration,
        /// Generation the admitted manifest records.
        observed: LivenessGeneration,
    },
    /// The selected manifest's predecessor differs from the head's predecessor.
    ManifestPredecessor {
        /// Predecessor the admitted head names.
        expected: Option<RetentionManifestDigest>,
        /// Predecessor the admitted manifest records.
        observed: Option<RetentionManifestDigest>,
    },
    /// One selected root failed canonical admission.
    RootDecode {
        /// Manifest entry whose root was refused.
        entry: RetentionManifestEntry,
        /// Preserved root decoder refusal.
        source: Box<RetentionRootDecodeError>,
    },
    /// An admitted root is not the root its manifest entry selects.
    ///
    /// The observed coordinate is boxed so every refusal stays small.
    RootBinding {
        /// Manifest entry that selected the root.
        expected: RetentionManifestEntry,
        /// Namespace, generation, and digest of the admitted root.
        observed: Box<RetentionManifestEntry>,
    },
    /// The candidate catalog cannot reconstruct one retained closure.
    Closure {
        /// Manifest entry whose root closure was refused.
        entry: RetentionManifestEntry,
        /// Preserved closure refusal against the candidate catalog.
        source: Box<RetentionClosureVerificationError>,
    },
    /// The retention head changed between admission and `HEAD` replacement.
    ///
    /// The observed head is boxed so every refusal stays small. `None` on
    /// either side means no retention head was published.
    RetentionChanged {
        /// Head admitted before the first catalog storage call.
        expected: Option<RetentionHead>,
        /// Head observed after `head.next` verification.
        observed: Box<Option<RetentionHead>>,
    },
}

impl From<CatalogPublicationError> for RetainedCatalogPublicationError {
    fn from(source: CatalogPublicationError) -> Self {
        Self::Catalog { source }
    }
}

impl Error for RetainedCatalogPublicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Catalog { source } => Some(source),
            Self::RetentionStorage { source, .. } | Self::RootStorage { source, .. } => {
                Some(source)
            }
            Self::HeadDecode { source, .. } => Some(source),
            Self::ManifestDecode { source } => Some(source.as_ref()),
            Self::RootDecode { source, .. } => Some(source.as_ref()),
            Self::Closure { source, .. } => Some(source.as_ref()),
            Self::ManifestLength { .. }
            | Self::ManifestDigest { .. }
            | Self::ManifestGeneration { .. }
            | Self::ManifestPredecessor { .. }
            | Self::RootBinding { .. }
            | Self::RetentionChanged { .. } => None,
        }
    }
}
