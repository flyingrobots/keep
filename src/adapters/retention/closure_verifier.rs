//! This module owns deterministic verification of one retained-root closure.

use std::collections::BTreeMap;

use crate::profile::StorageProfileVerifier;
use crate::{
    AdmittedLayout, AdmittedSegmentRecord, BlobHasher, CatalogSnapshot, LayoutDecodePolicy,
    LayoutEntryLimit, RetentionAnchor, RetentionClosureVerificationError, RetentionRoot,
    SegmentRecordIdentity,
};

use super::{
    VerifiedRetentionClosure, closure_accounting::ClosureAccounting, closure_digest,
    closure_profile_error,
};

const LAYOUT_DEPTH: u16 = 1;
const CHUNK_DEPTH: u16 = 2;

/// Verifies every anchor against one pinned admitted catalog.
///
/// Verification performs no I/O. It allocates one bounded ordered record index
/// and one bounded decoded entry set per anchor. Every selected chunk is
/// scanned to replay its storage profile and authenticate the complete blob.
///
/// # Errors
///
/// Returns the first deterministic resource, catalog-member, layout, profile,
/// or reconstructed-identity refusal. No failure returns partial evidence.
pub fn verify_retention_closure(
    root: &RetentionRoot,
    catalog: &CatalogSnapshot<'_, '_, '_>,
) -> Result<VerifiedRetentionClosure, RetentionClosureVerificationError> {
    let mut verifier = ClosureVerifier::new(root, catalog);
    for anchor in root.anchors().iter().copied() {
        verifier.verify_anchor(anchor)?;
    }
    Ok(verifier.finish())
}

struct ClosureVerifier<'snapshot, 'head, 'catalog, 'records> {
    root: &'snapshot RetentionRoot,
    catalog: &'snapshot CatalogSnapshot<'head, 'catalog, 'records>,
    accounting: ClosureAccounting,
    records: BTreeMap<SegmentRecordIdentity, AdmittedSegmentRecord<'records>>,
}

impl<'snapshot, 'head, 'catalog, 'records> ClosureVerifier<'snapshot, 'head, 'catalog, 'records> {
    const fn new(
        root: &'snapshot RetentionRoot,
        catalog: &'snapshot CatalogSnapshot<'head, 'catalog, 'records>,
    ) -> Self {
        Self {
            root,
            catalog,
            accounting: ClosureAccounting::new(root.limits()),
            records: BTreeMap::new(),
        }
    }

    fn verify_anchor(
        &mut self,
        anchor: RetentionAnchor,
    ) -> Result<(), RetentionClosureVerificationError> {
        let layout_id = anchor.layout_id();
        let identity = SegmentRecordIdentity::Layout(layout_id);
        let (record, first_scheduled) = self.resolve(identity, LAYOUT_DEPTH)?;
        self.accounting
            .add_physical(record.header().record_length().get())?;
        if first_scheduled {
            self.accounting
                .add_encoded(record.header().payload_length().get())?;
        }
        let policy = LayoutDecodePolicy::new(LayoutEntryLimit::MAXIMUM).with_expected_id(layout_id);
        let layout = AdmittedLayout::decode_record(record.payload(), policy).map_err(|source| {
            RetentionClosureVerificationError::LayoutDecode {
                layout: layout_id,
                source,
            }
        })?;
        require_anchor_target(anchor, &layout)?;
        self.verify_reconstruction(anchor, &layout)
    }

    fn verify_reconstruction(
        &mut self,
        anchor: RetentionAnchor,
        layout: &AdmittedLayout,
    ) -> Result<(), RetentionClosureVerificationError> {
        let layout_id = anchor.layout_id();
        let mut profile = StorageProfileVerifier::new(layout)
            .map_err(|error| closure_profile_error::map(layout_id, error))?;
        let mut blob = BlobHasher::new();
        for entry in layout.entries().iter().copied() {
            let identity = SegmentRecordIdentity::Chunk(entry.chunk_id());
            let (record, _first_scheduled) = self.resolve(identity, CHUNK_DEPTH)?;
            self.accounting
                .add_physical(record.header().record_length().get())?;
            let bytes = record.payload();
            profile
                .feed(bytes)
                .map_err(|error| closure_profile_error::map(layout_id, error))?;
            blob.update(bytes)
                .map_err(|source| RetentionClosureVerificationError::BlobHash {
                    layout: layout_id,
                    source,
                })?;
        }
        profile
            .finish()
            .map_err(|error| closure_profile_error::map(layout_id, error))?;
        let observed = blob.finish();
        let expected = anchor.blob_id();
        if observed != expected {
            return Err(RetentionClosureVerificationError::BlobIdentityMismatch {
                layout: layout_id,
                expected,
                observed,
            });
        }
        Ok(())
    }

    fn resolve(
        &mut self,
        identity: SegmentRecordIdentity,
        depth: u16,
    ) -> Result<(AdmittedSegmentRecord<'records>, bool), RetentionClosureVerificationError> {
        self.accounting.admit_depth(depth)?;
        if let Some(record) = self.records.get(&identity).copied() {
            return Ok((record, false));
        }
        self.accounting.add_node()?;
        let record = self
            .catalog
            .record(identity)
            .ok_or(RetentionClosureVerificationError::MissingMember { identity })?;
        self.records.insert(identity, record);
        Ok((record, true))
    }

    fn finish(self) -> VerifiedRetentionClosure {
        let profile = self.root.profile();
        let generation = self.catalog.generation();
        let catalog_digest = self.catalog.catalog_digest();
        let usage = self.accounting.usage();
        let digest =
            closure_digest::calculate(profile, generation, catalog_digest, usage, &self.records);
        VerifiedRetentionClosure::new(profile, generation, catalog_digest, usage, digest)
    }
}

fn require_anchor_target(
    anchor: RetentionAnchor,
    layout: &AdmittedLayout,
) -> Result<(), RetentionClosureVerificationError> {
    let expected = anchor.blob_id();
    let observed = layout.target();
    if observed == expected {
        return Ok(());
    }
    Err(RetentionClosureVerificationError::AnchorTargetMismatch {
        layout: anchor.layout_id(),
        expected,
        observed,
    })
}
