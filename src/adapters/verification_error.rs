//! This module owns verification outcomes with preserved boundary causes.

use std::error::Error;
use std::fmt;

use crate::{
    BlobHashError, LayoutDecodeError, RetentionClosureVerificationError, VerificationRefusal,
};

/// Why an operation produced no verification report.
///
/// Refusal is evidence about the supplied view. An operational failure proves
/// neither corruption nor absence. Error boxing bounds the successful result's
/// stack size; no payload bytes or filesystem paths are copied into a report.
#[derive(Debug)]
#[non_exhaustive]
pub enum VerificationError {
    /// A precise content or policy refusal, optionally with its original cause.
    Refused {
        /// Bounded semantic outcome.
        refusal: VerificationRefusal,
        /// Original boundary cause, without string conversion.
        source: Option<Box<VerificationSource>>,
    },
    /// Verification could not finish; no partial report is returned.
    Operational {
        /// Original resource, capability, or execution failure.
        source: Box<VerificationSource>,
    },
}

/// Original typed causes retained by verification adapters.
#[derive(Debug)]
#[non_exhaustive]
pub enum VerificationSource {
    /// Original reference-view request evidence or execution failure.
    Reference(crate::ReferenceVerificationSource),
    /// Fenced view collection could not choose one consistent observation.
    View(crate::RetentionViewError),
    /// Bounded immutable segment admission failed.
    Segment(crate::SegmentReadError),
    /// Published catalog loading or binding failed.
    Catalog(crate::CatalogRestartError),
    /// Published retention view or selected-root observation failed.
    Retention(crate::FilesystemRetentionSnapshotError),
    /// Canonical retention root admission failed.
    Root(crate::RetentionRootDecodeError),

    /// Canonical layout decoding failed.
    Layout(LayoutDecodeError),
    /// Retention closure admission or its resource accounting failed.
    Closure(RetentionClosureVerificationError),
    /// Complete blob hashing failed.
    BlobHash(BlobHashError),
}

impl From<VerificationRefusal> for VerificationError {
    fn from(refusal: VerificationRefusal) -> Self {
        Self::Refused {
            refusal,
            source: None,
        }
    }
}

impl fmt::Display for VerificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Refused { refusal, .. } => refusal.fmt(formatter),
            Self::Operational { .. } => formatter.write_str("verification could not complete"),
        }
    }
}

impl Error for VerificationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Refused {
                source: Some(source),
                ..
            }
            | Self::Operational { source } => Some(source),
            Self::Refused { source: None, .. } => None,
        }
    }
}

impl fmt::Display for VerificationSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reference(source) => source.fmt(formatter),
            Self::View(source) => source.fmt(formatter),
            Self::Segment(source) => source.fmt(formatter),
            Self::Catalog(source) => source.fmt(formatter),
            Self::Retention(source) => source.fmt(formatter),
            Self::Root(source) => source.fmt(formatter),
            Self::Layout(source) => source.fmt(formatter),

            Self::Closure(source) => source.fmt(formatter),
            Self::BlobHash(source) => source.fmt(formatter),
        }
    }
}

impl Error for VerificationSource {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Reference(source) => Some(source),
            Self::View(source) => Some(source),
            Self::Segment(source) => Some(source),
            Self::Catalog(source) => Some(source),
            Self::Retention(source) => Some(source),
            Self::Root(source) => Some(source),
            Self::Layout(source) => Some(source),

            Self::Closure(source) => Some(source),
            Self::BlobHash(source) => Some(source),
        }
    }
}
