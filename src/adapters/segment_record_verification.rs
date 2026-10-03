//! This module owns subject-specific evidence from an admitted segment record.

use super::{AdmittedSegmentRecord, SegmentRecordIdentity};
use crate::{VerificationDepth, VerificationRefusal, VerificationReport, VerificationSubject};

const CHUNK_DEPTHS: &[VerificationDepth] = &[
    VerificationDepth::Framing,
    VerificationDepth::Checksum,
    VerificationDepth::ChunkIdentity,
];
const LAYOUT_DEPTHS: &[VerificationDepth] = &[
    VerificationDepth::Framing,
    VerificationDepth::Checksum,
    VerificationDepth::LayoutIdentity,
];

impl AdmittedSegmentRecord<'_> {
    /// Reports evidence for this record's exact logical chunk or layout.
    ///
    /// Every record supports framing and checksum evidence. A chunk supports
    /// chunk identity; a layout supports layout identity. No depth infers that
    /// a layout's chunks are present, obey its storage profile, or reconstruct
    /// its named blob. Physical catalog and retention claims are unsupported.
    ///
    /// This constant-time operation allocates nothing, performs no I/O and
    /// changes no storage. Its prerequisite admission already checked framing,
    /// checksum and logical identity, even when the request is shallower; the
    /// borrowed immutable payload is not rehashed by reporting. Records built
    /// for publication carry the same logical proof without implying that
    /// their canonical bytes have been written or made durable.
    ///
    /// # Errors
    ///
    /// Returns [`VerificationRefusal::Unsupported`] for a depth outside the
    /// exact supported set for this record kind; no downgraded report is returned.
    pub fn verify(
        &self,
        requested: VerificationDepth,
    ) -> Result<VerificationReport, VerificationRefusal> {
        let (subject, supported) = match self.identity() {
            SegmentRecordIdentity::Chunk(identity) => {
                (VerificationSubject::Chunk { identity }, CHUNK_DEPTHS)
            }
            SegmentRecordIdentity::Layout(identity) => {
                (VerificationSubject::Layout { identity }, LAYOUT_DEPTHS)
            }
        };
        if !supported.contains(&requested) {
            return Err(VerificationRefusal::Unsupported {
                subject,
                requested,
                supported,
            });
        }
        Ok(VerificationReport::established(subject, requested))
    }
}
