//! This module owns restart assessment of the three fixed retention stages.

use super::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, ChecksummedRetentionHead,
    RetentionHeadDecodeError, RetentionManifestDecodeError, RetentionRootDecodeError,
};

/// One fixed retention stage as assessed from its exact bytes at restart.
///
/// `Truncated` means the bytes end before the boundary the record's own
/// framing declares, which is the shape a crash during the stage write leaves
/// behind. Every other decode failure is `Corrupt`: a complete-looking record
/// that fails a checksum, digest, or semantic law is unrecoverable ambiguity,
/// never an incomplete write.
#[derive(Debug)]
pub enum RetentionStageAssessment<Record, Error> {
    /// No entry exists under the stage name.
    Absent,
    /// The bytes decode as one canonical record.
    Complete(Record),
    /// The bytes end before the declared record boundary.
    Truncated {
        /// The length the framing declares.
        expected: usize,
        /// The length that was present.
        observed: usize,
    },
    /// The bytes are complete enough to judge and fail a canonical law.
    Corrupt(Error),
}

/// Assessment of `retention/root.next`.
pub type RetentionRootStageAssessment<'bytes> =
    RetentionStageAssessment<AdmittedRetentionRoot<'bytes>, RetentionRootDecodeError>;
/// Assessment of `retention/manifest.next`.
pub type RetentionManifestStageAssessment<'bytes> =
    RetentionStageAssessment<AdmittedRetentionManifest<'bytes>, RetentionManifestDecodeError>;
/// Assessment of `retention/head.next`.
pub type RetentionHeadStageAssessment<'bytes> =
    RetentionStageAssessment<ChecksummedRetentionHead<'bytes>, RetentionHeadDecodeError>;

impl<Record, Error> RetentionStageAssessment<Record, Error> {
    /// Returns whether an entry exists under the stage name.
    #[must_use]
    pub const fn is_present(&self) -> bool {
        !matches!(self, Self::Absent)
    }
}

/// Assesses the bytes found under `retention/root.next`, if any.
#[must_use]
pub fn assess_root_stage(bytes: Option<&[u8]>) -> RetentionRootStageAssessment<'_> {
    match bytes.map(AdmittedRetentionRoot::decode) {
        None => RetentionStageAssessment::Absent,
        Some(Ok(root)) => RetentionStageAssessment::Complete(root),
        Some(Err(RetentionRootDecodeError::Truncated { expected, observed })) => {
            RetentionStageAssessment::Truncated { expected, observed }
        }
        Some(Err(source)) => RetentionStageAssessment::Corrupt(source),
    }
}

/// Assesses the bytes found under `retention/manifest.next`, if any.
#[must_use]
pub fn assess_manifest_stage(bytes: Option<&[u8]>) -> RetentionManifestStageAssessment<'_> {
    match bytes.map(AdmittedRetentionManifest::decode) {
        None => RetentionStageAssessment::Absent,
        Some(Ok(manifest)) => RetentionStageAssessment::Complete(manifest),
        Some(Err(RetentionManifestDecodeError::Truncated { expected, observed })) => {
            RetentionStageAssessment::Truncated { expected, observed }
        }
        Some(Err(source)) => RetentionStageAssessment::Corrupt(source),
    }
}

/// Assesses the bytes found under `retention/head.next`, if any.
///
/// The head is one fixed 144-byte record, so fewer bytes are a truncation and
/// more bytes are corruption.
#[must_use]
pub fn assess_head_stage(bytes: Option<&[u8]>) -> RetentionHeadStageAssessment<'_> {
    match bytes.map(ChecksummedRetentionHead::decode) {
        None => RetentionStageAssessment::Absent,
        Some(Ok(head)) => RetentionStageAssessment::Complete(head),
        Some(Err(RetentionHeadDecodeError::WrongLength { expected, observed }))
            if observed < expected =>
        {
            RetentionStageAssessment::Truncated { expected, observed }
        }
        Some(Err(source)) => RetentionStageAssessment::Corrupt(source),
    }
}
