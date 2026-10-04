//! Explicit-depth verification over the non-durable reference view.
#![expect(
    clippy::result_large_err,
    reason = "preserve main's bounded inline diagnostic coordinates without a new refusal allocation"
)]

use crate::profile::StorageProfileVerifier;
use crate::{
    AdmittedLayout, BlobHasher, BlobId, CorruptionEvidence, LayoutId, MissingEvidence,
    ReferenceStore, VerificationDepth, VerificationError, VerificationReport, VerificationSubject,
};

use super::ReferenceVerificationSource as Source;
use super::verification_outcome::{
    canonical_identity, chunk_outcome, missing, operational, profile_failure, profile_outcome,
    refused, require_layout_identity, require_supported,
};
use super::{ReferenceVerificationContext as Context, ReferenceVerificationEvidence as Evidence};
use crate::authenticated_read::verified_chunk;

impl ReferenceStore {
    /// Verifies one committed subject to exactly `depth` and reports it.
    ///
    /// The reference view supports `ChunkIdentity` through
    /// `CompleteBlobIdentity`; any other depth is refused as unsupported,
    /// never degraded. Every chunk the layout names is read and hashed once.
    /// `LayoutIdentity` and deeper additionally materialize one canonical
    /// layout record bounded by the layout's entry limit; `CompleteBlobIdentity`
    /// additionally replays the registered storage profile. A lower-stage
    /// refusal is always reported before a deeper one. Nothing is repaired.
    ///
    /// # Errors
    ///
    /// Returns [`VerificationError::Refused`] with exact missing, corrupt, or
    /// unsupported evidence, or [`VerificationError::Operational`] when the
    /// operation could not run to a conclusion.
    pub fn verify(
        &self,
        subject: VerificationSubject,
        depth: VerificationDepth,
    ) -> Result<VerificationReport, VerificationError> {
        require_supported(subject, depth)?;
        let layout_id = match subject {
            VerificationSubject::Blob { identity: target } => self
                .first_layout_id(target)
                .ok_or_else(|| missing(subject, MissingEvidence::Blob(target)))?,
            VerificationSubject::Layout {
                identity: layout_id,
            } => layout_id,
            _ => return Err(refused(Context::new(subject, depth, Evidence::Unsupported))),
        };
        let layout = self
            .layout(layout_id)
            .ok_or_else(|| missing(subject, MissingEvidence::Layout(layout_id)))?;
        let binding = LayoutBinding {
            id: layout_id,
            layout,
            committed: true,
        };
        verify_layout(self, subject, depth, &binding)
    }

    /// Verifies a caller-supplied admitted layout against this view.
    ///
    /// The layout need not be committed here; its canonical identity is
    /// calculated first and becomes the report's subject. Every other law is
    /// identical to [`ReferenceStore::verify`].
    ///
    /// # Errors
    ///
    /// As [`ReferenceStore::verify`], plus an operational failure when the
    /// layout cannot produce its canonical record.
    pub fn verify_admitted_layout(
        &self,
        layout: &AdmittedLayout,
        depth: VerificationDepth,
    ) -> Result<VerificationReport, VerificationError> {
        let layout_id = canonical_identity(layout)?;
        let subject = VerificationSubject::Layout {
            identity: layout_id,
        };
        require_supported(subject, depth)?;
        let binding = LayoutBinding {
            id: layout_id,
            layout,
            committed: false,
        };
        verify_layout(self, subject, depth, &binding)
    }
}

/// One layout under verification and whether the view committed it under
/// its identity (a supplied layout has nothing to compare its identity to).
struct LayoutBinding<'a> {
    id: LayoutId,
    layout: &'a AdmittedLayout,
    committed: bool,
}

fn verify_layout(
    store: &ReferenceStore,
    subject: VerificationSubject,
    depth: VerificationDepth,
    binding: &LayoutBinding<'_>,
) -> Result<VerificationReport, VerificationError> {
    let layout_id = binding.id;
    let layout = binding.layout;
    let committed = binding.committed;
    let mut pass = DeepPass::begin(depth, subject, layout_id, layout)?;
    let mut chunks_verified = 0_u64;
    for (index, entry) in layout.entries().iter().copied().enumerate() {
        let bytes = verified_chunk(store, layout_id, index, entry)
            .map_err(|error| chunk_outcome(subject, error))?;
        pass.feed(bytes)?;
        chunks_verified = chunks_verified
            .checked_add(1)
            .ok_or_else(|| operational(Source::ChunkCountOverflow))?;
    }
    if committed
        && matches!(
            depth,
            VerificationDepth::LayoutIdentity | VerificationDepth::CompleteBlobIdentity
        )
    {
        require_layout_identity(subject, layout_id, layout)?;
    }
    pass.conclude(layout.target())?;
    Ok(
        VerificationReport::established(subject, depth).in_reference(
            layout_id,
            layout.target(),
            chunks_verified,
        ),
    )
}

/// The complete-blob work folded into the single chunk pass.
///
/// A profile-boundary contradiction found mid-pass is held until every chunk
/// has been authenticated, so a chunk-identity refusal (a lower stage) is
/// always the one reported.
struct DeepPass<'a> {
    subject: VerificationSubject,
    layout_id: LayoutId,
    hasher: Option<BlobHasher>,
    profile: Option<StorageProfileVerifier<'a>>,
    pending: Option<Context>,
}

impl<'a> DeepPass<'a> {
    fn begin(
        depth: VerificationDepth,
        subject: VerificationSubject,
        layout_id: LayoutId,
        layout: &'a AdmittedLayout,
    ) -> Result<Self, VerificationError> {
        if depth != VerificationDepth::CompleteBlobIdentity {
            return Ok(Self {
                subject,
                layout_id,
                hasher: None,
                profile: None,
                pending: None,
            });
        }
        let profile = StorageProfileVerifier::new(layout)
            .map_err(|error| profile_failure(subject, layout_id, error))?;
        Ok(Self {
            subject,
            layout_id,
            hasher: Some(BlobHasher::new()),
            profile: Some(profile),
            pending: None,
        })
    }

    fn feed(&mut self, bytes: &[u8]) -> Result<(), VerificationError> {
        if let Some(hasher) = self.hasher.as_mut() {
            hasher
                .update(bytes)
                .map_err(|source| operational(Source::BlobHash(source)))?;
        }
        if let Some(profile) = self.profile.as_mut()
            && let Err(error) = profile.feed(bytes)
        {
            self.profile = None;
            self.pending = Some(profile_outcome(self.subject, self.layout_id, error)?);
        }
        Ok(())
    }

    fn conclude(self, target: BlobId) -> Result<(), VerificationError> {
        let Self {
            subject,
            layout_id,
            hasher,
            profile,
            pending,
        } = self;
        if let Some(pending) = pending {
            return Err(refused(pending));
        }
        if let Some(profile) = profile
            && let Err(error) = profile.finish()
        {
            return Err(refused(profile_outcome(subject, layout_id, error)?));
        }
        let Some(hasher) = hasher else {
            return Ok(());
        };
        let observed = hasher.finish();
        if observed == target {
            return Ok(());
        }
        Err(refused(Context::new(
            subject,
            VerificationDepth::CompleteBlobIdentity,
            Evidence::Corrupt(CorruptionEvidence::BlobIdentity {
                layout: layout_id,
                expected: target,
                observed,
            }),
        )))
    }
}

#[cfg(test)]
#[path = "verification_tests.rs"]
mod tests;
