//! This module owns evidence reporting over an already admitted segment.

use super::AdmittedSegment;
use crate::{VerificationDepth, VerificationRefusal, VerificationReport, VerificationSubject};

const SUPPORTED: &[VerificationDepth] = &[VerificationDepth::Framing, VerificationDepth::Checksum];

impl AdmittedSegment<'_> {
    /// Reports framing or checksum evidence for this exact physical segment.
    ///
    /// Reporting is constant time, allocates nothing, performs no I/O, and
    /// does not mutate or synchronize storage. Construction of this admitted
    /// segment already checked its header, every record and seal, including
    /// checksums, logical identities and duplicates under the admission policy.
    /// Even a framing request therefore requires successful full admission;
    /// this operation is not a shallow scan that ignores deeper corruption.
    ///
    /// The report names this segment's digest only. It grants no publication,
    /// catalog membership, complete-blob identity or retention authority.
    /// Obtain logical chunk/layout evidence from its admitted records.
    ///
    /// # Errors
    ///
    /// Returns [`VerificationRefusal::Unsupported`] for any other depth,
    /// preserving the subject, requested depth and exact supported set.
    pub fn verify(
        &self,
        requested: VerificationDepth,
    ) -> Result<VerificationReport, VerificationRefusal> {
        let subject = VerificationSubject::Segment {
            digest: self.digest(),
        };
        if !SUPPORTED.contains(&requested) {
            return Err(VerificationRefusal::Unsupported {
                subject,
                requested,
                supported: SUPPORTED,
            });
        }
        Ok(VerificationReport::established(subject, requested))
    }
}
