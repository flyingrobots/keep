//! This module owns evidence reports for an admitted retention root.
#![expect(
    clippy::result_large_err,
    reason = "bounded full diagnostic coordinates remain inline instead of adding refusal allocations"
)]

use super::{AdmittedRetentionRoot, verify_retention_closure};
use crate::adapters::verification_admission;
use crate::{
    CatalogSnapshot, VerificationDepth, VerificationError, VerificationRefusal, VerificationReport,
    VerificationSubject,
};

const SUPPORTED: &[VerificationDepth] = &[
    VerificationDepth::Framing,
    VerificationDepth::Checksum,
    VerificationDepth::RetentionClosure,
];

impl AdmittedRetentionRoot<'_> {
    /// Verifies this exact root at the requested depth against `catalog`.
    ///
    /// Framing/checksum reporting is constant-time and allocation-free over
    /// already admitted canonical bytes and claims no catalog provenance.
    /// Closure verification visits every
    /// anchor, requires catalog members, replays storage profiles and hashes
    /// complete blobs. Its checked node, depth, encoded and physical byte
    /// limits are those admitted from the root. It allocates an ordered index
    /// bounded by the node limit and one decoded layout at a time. No I/O,
    /// mutation, repair, publication or retention authority is performed.
    ///
    /// # Errors
    ///
    /// Missing members, contradictory content, unsupported depths, and resource
    /// or execution failures remain distinct; original typed causes survive.
    /// A failed anchor prevents a report for the whole root.
    pub fn verify(
        &self,
        catalog: &CatalogSnapshot<'_, '_, '_>,
        requested: VerificationDepth,
    ) -> Result<VerificationReport, VerificationError> {
        let root = self.root();
        let subject = VerificationSubject::RetentionRoot {
            namespace: root.namespace().digest(),
            generation: root.generation(),
            digest: self.digest(),
        };
        if !SUPPORTED.contains(&requested) {
            return Err(VerificationRefusal::Unsupported {
                subject,
                requested,
                supported: SUPPORTED,
            }
            .into());
        }
        let report = VerificationReport::established(subject, requested);
        if requested != VerificationDepth::RetentionClosure {
            return Ok(report);
        }
        let _evidence = verify_retention_closure(root, catalog)
            .map_err(|source| verification_admission::closure(subject, source))?;
        Ok(report.in_catalog(catalog.generation(), catalog.catalog_digest()))
    }
}
