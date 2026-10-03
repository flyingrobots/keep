//! This module owns evidence reporting over an already admitted catalog view.
#![expect(
    clippy::result_large_err,
    reason = "bounded full diagnostic coordinates remain inline instead of adding refusal allocations"
)]

use super::CatalogSnapshot;
use crate::{VerificationDepth, VerificationRefusal, VerificationReport, VerificationSubject};

const SUPPORTED: &[VerificationDepth] = &[
    VerificationDepth::Framing,
    VerificationDepth::Checksum,
    VerificationDepth::CatalogReachability,
];

impl CatalogSnapshot<'_, '_, '_> {
    /// Reports the requested evidence for this exact admitted catalog.
    ///
    /// Supports framing, checksum, and catalog reachability. Admission already
    /// verified those properties and bound each catalog entry to an admitted
    /// segment record. Reporting is constant time, performs no I/O, allocates
    /// nothing, and does not block or mutate storage.
    ///
    /// This is not a shallow scan of untrusted bytes: constructing the snapshot
    /// first checks catalog/segment integrity and record identities, even for a
    /// framing request. It does not traverse each layout's referenced chunks,
    /// reconstruct all blobs, or verify retained-root closure. The report binds
    /// the owned or borrowed snapshot bytes, not a later filesystem observation.
    ///
    /// # Errors
    ///
    /// Returns [`VerificationRefusal::Unsupported`] for any other depth, with
    /// the exact subject, request, and supported set. No shallower report is
    /// returned after an unsupported request.
    pub fn verify(
        &self,
        requested: VerificationDepth,
    ) -> Result<VerificationReport, VerificationRefusal> {
        let subject = VerificationSubject::Catalog {
            generation: self.generation(),
            digest: self.catalog_digest(),
        };
        if !SUPPORTED.contains(&requested) {
            return Err(VerificationRefusal::Unsupported {
                subject,
                requested,
                supported: SUPPORTED,
            });
        }
        Ok(VerificationReport::established(subject, requested)
            .in_catalog(self.generation(), self.catalog_digest()))
    }
}
