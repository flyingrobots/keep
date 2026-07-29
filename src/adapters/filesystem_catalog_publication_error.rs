//! This module owns filesystem catalog publication invariant failures.

use std::error::Error;
use std::fmt;

use super::CatalogRestartArtifact;
use crate::{CatalogDigest, CatalogGeneration};

/// Filesystem state disagreed with a preflighted publication invariant.
#[derive(Debug)]
pub enum FilesystemCatalogPublicationError {
    /// The selected segment has no authority from this publisher.
    SegmentAuthorityRequired,
    /// The current publication coordinate was stale or unexpectedly present.
    CurrentState {
        /// Generation required by the caller, absent for initialization.
        expected_generation: Option<CatalogGeneration>,
        /// Digest required by the caller, absent for initialization.
        expected_digest: Option<CatalogDigest>,
        /// Generation verified from `HEAD`, absent when no head exists.
        observed_generation: Option<CatalogGeneration>,
        /// Digest verified from `HEAD`, absent when no head exists.
        observed_digest: Option<CatalogDigest>,
    },
    /// A stage handle required by the current phase was absent.
    StageState {
        /// Artifact whose writable stage was not retained.
        artifact: CatalogRestartArtifact,
    },
    /// Valid bytes at a physical coordinate differed from preflighted bytes.
    ByteConflict {
        /// Artifact whose exact bytes disagreed.
        artifact: CatalogRestartArtifact,
    },
    /// A leftover `head.next` requires explicit recovery.
    HeadRecoveryRequired,
    /// A leftover `staging/current.cat` requires explicit recovery.
    CatalogRecoveryRequired,
    /// A leftover `staging/current.seg` requires explicit recovery.
    SegmentRecoveryRequired,
}

impl fmt::Display for FilesystemCatalogPublicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SegmentAuthorityRequired => {
                formatter.write_str("selected segment lacks publisher stage authority")
            }
            Self::CurrentState { .. } => {
                formatter.write_str("current catalog publication state is stale")
            }
            Self::StageState { .. } => {
                formatter.write_str("catalog publication stage state is invalid")
            }
            Self::ByteConflict { .. } => formatter.write_str("publication artifact bytes conflict"),
            Self::HeadRecoveryRequired => {
                formatter.write_str("head.next requires explicit recovery")
            }
            Self::CatalogRecoveryRequired => {
                formatter.write_str("current.cat requires explicit recovery")
            }
            Self::SegmentRecoveryRequired => {
                formatter.write_str("current.seg requires explicit recovery")
            }
        }
    }
}

impl Error for FilesystemCatalogPublicationError {}
