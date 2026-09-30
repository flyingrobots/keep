//! This boundary module owns one GC retirement candidate.

use super::VerificationEvidenceDigest;
use crate::SegmentDigest;

/// One immutable segment proposed for retirement.
///
/// The value names the segment by its physical digest and exact length and
/// carries the digest of the evidence that verified it. It makes no claim
/// that the segment is unreachable; the intent that lists it does.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GcCandidate {
    segment_digest: SegmentDigest,
    segment_length: u64,
    evidence_digest: VerificationEvidenceDigest,
}

impl GcCandidate {
    /// Binds one candidate segment to its verification evidence.
    pub const fn new(
        segment_digest: SegmentDigest,
        segment_length: u64,
        evidence_digest: VerificationEvidenceDigest,
    ) -> Self {
        Self {
            segment_digest,
            segment_length,
            evidence_digest,
        }
    }

    /// Returns the physical segment digest.
    pub const fn segment_digest(&self) -> SegmentDigest {
        self.segment_digest
    }

    /// Returns the exact segment byte length.
    #[must_use]
    pub const fn segment_length(&self) -> u64 {
        self.segment_length
    }

    /// Returns the verification-evidence digest.
    pub const fn evidence_digest(&self) -> VerificationEvidenceDigest {
        self.evidence_digest
    }
}
