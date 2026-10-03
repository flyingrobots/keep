//! This module owns blob evidence over an immutable admitted catalog.
#![expect(
    clippy::result_large_err,
    reason = "bounded full diagnostic coordinates remain inline instead of adding refusal allocations"
)]

use super::{VerificationError, retention, verification_admission};
use crate::profile::StorageProfileVerifier;
use crate::{
    AdmittedLayout, BlobHasher, BlobId, CatalogSnapshot, LayoutDecodePolicy, LayoutEntryLimit,
    LayoutId, RetentionClosureVerificationError as Closure, SegmentRecordIdentity,
    VerificationDepth, VerificationRefusal, VerificationReport, VerificationSubject,
};

const SUPPORTED: &[VerificationDepth] = &[
    VerificationDepth::Framing,
    VerificationDepth::Checksum,
    VerificationDepth::ChunkIdentity,
    VerificationDepth::LayoutIdentity,
    VerificationDepth::CompleteBlobIdentity,
];

impl CatalogSnapshot<'_, '_, '_> {
    /// Verifies the first canonical catalogued layout naming `blob`.
    ///
    /// Multiple valid layouts are representations, not conflicting evidence.
    /// Selection follows canonical layout identity order. Discovery scans and
    /// decodes catalogued layouts one at a time; peak additional memory is one
    /// bounded decoded layout. No plaintext buffer, I/O, mutation or writer
    /// authority is required. Catalog admission already checked record framing,
    /// checksums and logical identities, including for shallow requests.
    /// Chunk depth requires every selected member; complete depth additionally
    /// streams all bytes through profile replay and the complete blob hash.
    /// Work is linear in discovered layout bytes, plus selected blob bytes for
    /// complete depth, with logarithmic catalog lookup per selected chunk.
    ///
    /// # Errors
    ///
    /// Returns exact missing, corrupt, unsupported or operational outcomes;
    /// no error returns a shallower report. The report describes this immutable
    /// view's evidence and grants no publication or retention authority.
    pub fn verify_blob(
        &self,
        blob: BlobId,
        requested: VerificationDepth,
    ) -> Result<VerificationReport, VerificationError> {
        let subject = VerificationSubject::Blob { identity: blob };
        if !SUPPORTED.contains(&requested) {
            return Err(VerificationRefusal::Unsupported {
                subject,
                requested,
                supported: SUPPORTED,
            }
            .into());
        }
        let (identity, layout) = self.find_verification_layout(blob)?;
        match requested {
            VerificationDepth::ChunkIdentity => require_chunks(self, &layout),
            VerificationDepth::CompleteBlobIdentity => verify_complete(self, identity, &layout),
            _ => Ok(()),
        }
        .map_err(|source| verification_admission::closure(subject, source))?;
        Ok(VerificationReport::established(subject, requested)
            .in_catalog(self.generation(), self.catalog_digest()))
    }

    fn find_verification_layout(
        &self,
        blob: BlobId,
    ) -> Result<(LayoutId, AdmittedLayout), VerificationError> {
        for record in self.records() {
            let SegmentRecordIdentity::Layout(identity) = record.identity() else {
                continue;
            };
            let policy =
                LayoutDecodePolicy::new(LayoutEntryLimit::MAXIMUM).with_expected_id(identity);
            let layout =
                AdmittedLayout::decode_record(record.payload(), policy).map_err(|source| {
                    verification_admission::layout(VerificationSubject::Layout { identity }, source)
                })?;
            if layout.target() == blob {
                return Ok((identity, layout));
            }
        }
        Err(VerificationRefusal::Missing {
            subject: VerificationSubject::Blob { identity: blob },
        }
        .into())
    }
}

fn require_chunks(
    catalog: &CatalogSnapshot<'_, '_, '_>,
    layout: &AdmittedLayout,
) -> Result<(), Closure> {
    for entry in layout.entries() {
        let identity = SegmentRecordIdentity::Chunk(entry.chunk_id());
        let _record = catalog
            .record(identity)
            .ok_or(Closure::MissingMember { identity })?;
    }
    Ok(())
}

fn verify_complete(
    catalog: &CatalogSnapshot<'_, '_, '_>,
    identity: LayoutId,
    layout: &AdmittedLayout,
) -> Result<(), Closure> {
    let mut profile = StorageProfileVerifier::new(layout)
        .map_err(|source| retention::profile_verification_refusal(identity, source))?;
    let mut hasher = BlobHasher::new();
    for entry in layout.entries() {
        let member = SegmentRecordIdentity::Chunk(entry.chunk_id());
        let record = catalog
            .record(member)
            .ok_or(Closure::MissingMember { identity: member })?;
        profile
            .feed(record.payload())
            .map_err(|source| retention::profile_verification_refusal(identity, source))?;
        hasher
            .update(record.payload())
            .map_err(|source| Closure::BlobHash {
                layout: identity,
                source,
            })?;
    }
    profile
        .finish()
        .map_err(|source| retention::profile_verification_refusal(identity, source))?;
    let expected = layout.target();
    let observed = hasher.finish();
    if expected != observed {
        return Err(Closure::BlobIdentityMismatch {
            layout: identity,
            expected,
            observed,
        });
    }
    Ok(())
}
