//! This module owns storage-independent double collection of one reader view.

use std::error::Error;
use std::fmt;
use std::io;

use super::ReaderAttemptLimit;
use crate::{CatalogDigest, CatalogGeneration, LivenessGeneration, RetentionManifestDigest};

/// The coordinates both heads name at one instant.
///
/// A view is accepted only when the coordinates read before loading it equal
/// the coordinates read after, so the view belongs to one catalog generation
/// and one liveness generation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetentionViewCoordinates {
    /// The catalog `HEAD` coordinate, or `None` when no catalog is published.
    pub catalog: Option<(CatalogGeneration, CatalogDigest)>,
    /// The `retention/HEAD` coordinate, or `None` when no retention head is published.
    pub retention: Option<(LivenessGeneration, RetentionManifestDigest)>,
}

/// The reads one reader view needs, in the order the collector calls them.
pub trait RetentionViewSource {
    /// The complete view loaded between two coordinate reads.
    type View;

    /// Reads both head coordinates without loading anything they select.
    ///
    /// # Errors
    ///
    /// Returns the exact read or decode failure.
    fn coordinates(&mut self) -> io::Result<RetentionViewCoordinates>;

    /// Loads the complete view the current heads select.
    ///
    /// # Errors
    ///
    /// Returns the exact load failure.
    fn load(&mut self) -> io::Result<Self::View>;
}

/// Why a reader view could not be collected.
#[derive(Debug)]
#[non_exhaustive]
pub enum RetentionViewError {
    /// No catalog `HEAD` is published, so no view exists to collect.
    CatalogAbsent,
    /// The heads moved between every collection within the attempt limit.
    AttemptsExhausted {
        /// The attempts that were made.
        attempts: u32,
    },
    /// A read or load failed.
    Io {
        /// The exact failure.
        source: io::Error,
    },
}

impl fmt::Display for RetentionViewError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CatalogAbsent => formatter.write_str("no catalog head is published"),
            Self::AttemptsExhausted { attempts } => write!(
                formatter,
                "the store moved between every one of {attempts} view collections"
            ),
            Self::Io { .. } => formatter.write_str("reader view collection failed"),
        }
    }
}

impl Error for RetentionViewError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source } => Some(source),
            Self::CatalogAbsent | Self::AttemptsExhausted { .. } => None,
        }
    }
}

/// Collects one view whose head coordinates agree before and after loading.
///
/// Each attempt reads the coordinates, loads the view, and reads the
/// coordinates again; a view is accepted only when both reads agree. A
/// generation, length, digest, or checksum change discards the view and
/// retries until `limit` is exhausted, which refuses.
///
/// # Errors
///
/// Returns [`RetentionViewError`] for an absent catalog, an exhausted limit,
/// or the source's own failure.
pub fn collect_retention_view<S: RetentionViewSource>(
    source: &mut S,
    limit: ReaderAttemptLimit,
) -> Result<S::View, RetentionViewError> {
    let io = |source| RetentionViewError::Io { source };
    for _attempt in 0..limit.get() {
        let before = source.coordinates().map_err(io)?;
        if before.catalog.is_none() {
            return Err(RetentionViewError::CatalogAbsent);
        }
        let view = source.load().map_err(io)?;
        let after = source.coordinates().map_err(io)?;
        if before == after {
            return Ok(view);
        }
    }
    Err(RetentionViewError::AttemptsExhausted {
        attempts: limit.get(),
    })
}
