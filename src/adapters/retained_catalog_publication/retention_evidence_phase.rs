//! This module owns the retention evidence reads of catalog publication.

use std::fmt;

/// Retention evidence read attempted by retained-closure catalog publication.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionEvidencePhase {
    /// Read the current retention head before any catalog storage call.
    AdmitHead,
    /// Read the manifest the admitted retention head selects.
    ReadManifest,
    /// Reread the retention head after `head.next` verification.
    RevalidateHead,
}

impl fmt::Display for RetentionEvidencePhase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::AdmitHead => "retention-head admission",
            Self::ReadManifest => "retention-manifest read",
            Self::RevalidateHead => "retention-head revalidation",
        })
    }
}
