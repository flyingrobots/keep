//! Explicit-depth verification over the non-durable reference view.

use crate::profile::{StorageProfileVerificationError, StorageProfileVerifier};
use crate::{
    AdmittedLayout, BlobHasher, BlobId, CorruptionEvidence, LayoutId, MissingEvidence,
    ReferenceStore, VerificationDepth, VerificationError, VerificationFailure, VerificationRefusal,
    VerificationReport, VerificationSubject,
};

use super::chunk_verification::{ChunkVerificationError, verified_chunk};

/// Shallowest depth the in-memory view establishes: it holds no durable
/// framing or checksums to verify.
const SUPPORTED_MINIMUM: VerificationDepth = VerificationDepth::ChunkIdentity;
/// Deepest depth the in-memory view establishes: it has no catalog and no
/// retention.
const SUPPORTED_MAXIMUM: VerificationDepth = VerificationDepth::CompleteBlobIdentity;

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
            VerificationSubject::Blob(target) => self
                .first_layout_id(target)
                .ok_or_else(|| missing(subject, MissingEvidence::Blob(target)))?,
            VerificationSubject::Layout(layout_id) => layout_id,
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
        let subject = VerificationSubject::Layout(layout_id);
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
        chunks_verified = chunks_verified.saturating_add(1);
    }
    if committed && depth >= VerificationDepth::LayoutIdentity {
        require_layout_identity(subject, layout_id, layout)?;
    }
    pass.conclude(layout.target())?;
    Ok(VerificationReport::established(
        subject,
        depth,
        layout_id,
        layout.target(),
        chunks_verified,
    ))
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
    pending: Option<VerificationRefusal>,
}

impl<'a> DeepPass<'a> {
    fn begin(
        depth: VerificationDepth,
        subject: VerificationSubject,
        layout_id: LayoutId,
        layout: &'a AdmittedLayout,
    ) -> Result<Self, VerificationError> {
        if depth < VerificationDepth::CompleteBlobIdentity {
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
            hasher.update(bytes).map_err(|source| {
                VerificationError::Operational(VerificationFailure::BlobHash { source })
            })?;
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
            return Err(VerificationError::refused(pending));
        }
        if let Some(profile) = profile
            && let Err(error) = profile.finish()
        {
            return Err(VerificationError::refused(profile_outcome(
                subject, layout_id, error,
            )?));
        }
        let Some(hasher) = hasher else {
            return Ok(());
        };
        let observed = hasher.finish();
        if observed == target {
            return Ok(());
        }
        Err(VerificationError::refused(VerificationRefusal::Corrupt {
            subject,
            stage: VerificationDepth::CompleteBlobIdentity,
            evidence: CorruptionEvidence::BlobIdentity {
                layout: layout_id,
                expected: target,
                observed,
            },
        }))
    }
}

/// Maps a replay error to the refusal it evidences, or to the operational
/// failure it is.
fn profile_outcome(
    subject: VerificationSubject,
    layout_id: LayoutId,
    error: StorageProfileVerificationError,
) -> Result<VerificationRefusal, VerificationError> {
    match error {
        StorageProfileVerificationError::BoundaryMismatch { index, .. } => {
            Ok(VerificationRefusal::Corrupt {
                subject,
                stage: VerificationDepth::CompleteBlobIdentity,
                evidence: CorruptionEvidence::ProfileBoundary {
                    layout: layout_id,
                    index,
                },
            })
        }
        other => Err(profile_failure(subject, layout_id, other)),
    }
}

fn require_supported(
    subject: VerificationSubject,
    depth: VerificationDepth,
) -> Result<(), VerificationError> {
    if (SUPPORTED_MINIMUM..=SUPPORTED_MAXIMUM).contains(&depth) {
        Ok(())
    } else {
        Err(VerificationError::refused(
            VerificationRefusal::Unsupported {
                subject,
                requested: depth,
                supported_minimum: SUPPORTED_MINIMUM,
                supported_maximum: SUPPORTED_MAXIMUM,
            },
        ))
    }
}

fn require_layout_identity(
    subject: VerificationSubject,
    expected: LayoutId,
    layout: &AdmittedLayout,
) -> Result<(), VerificationError> {
    let observed = canonical_identity(layout)?;
    if observed == expected {
        Ok(())
    } else {
        Err(VerificationError::refused(VerificationRefusal::Corrupt {
            subject,
            stage: VerificationDepth::LayoutIdentity,
            evidence: CorruptionEvidence::LayoutIdentity { expected, observed },
        }))
    }
}

fn canonical_identity(layout: &AdmittedLayout) -> Result<LayoutId, VerificationError> {
    layout
        .encode_record()
        .map(|record| record.id())
        .map_err(|source| {
            VerificationError::Operational(VerificationFailure::LayoutEncoding { source })
        })
}

fn missing(subject: VerificationSubject, evidence: MissingEvidence) -> VerificationError {
    VerificationError::refused(VerificationRefusal::Missing {
        subject,
        stage: SUPPORTED_MINIMUM,
        evidence,
    })
}

fn chunk_outcome(subject: VerificationSubject, error: ChunkVerificationError) -> VerificationError {
    match error {
        ChunkVerificationError::Missing {
            layout,
            index,
            requested,
        } => VerificationError::refused(VerificationRefusal::Missing {
            subject,
            stage: VerificationDepth::ChunkIdentity,
            evidence: MissingEvidence::Chunk {
                layout,
                index,
                chunk: requested,
            },
        }),
        ChunkVerificationError::IdentityMismatch {
            layout,
            index,
            expected,
            observed,
        } => VerificationError::refused(VerificationRefusal::Corrupt {
            subject,
            stage: VerificationDepth::ChunkIdentity,
            evidence: CorruptionEvidence::ChunkIdentity {
                layout,
                index,
                expected,
                observed,
            },
        }),
        ChunkVerificationError::Hash {
            layout,
            index,
            source,
            ..
        } => VerificationError::Operational(VerificationFailure::ChunkHash {
            layout,
            index,
            source,
        }),
    }
}

fn profile_failure(
    subject: VerificationSubject,
    layout: LayoutId,
    error: StorageProfileVerificationError,
) -> VerificationError {
    match error {
        StorageProfileVerificationError::Unsupported { profile } => {
            VerificationError::Operational(VerificationFailure::ProfileVerifierUnavailable {
                layout,
                profile,
            })
        }
        StorageProfileVerificationError::Chunking { source } => {
            VerificationError::Operational(VerificationFailure::ProfileChunking { layout, source })
        }
        StorageProfileVerificationError::BoundaryMismatch { index, .. } => {
            VerificationError::refused(VerificationRefusal::Corrupt {
                subject,
                stage: VerificationDepth::CompleteBlobIdentity,
                evidence: CorruptionEvidence::ProfileBoundary { layout, index },
            })
        }
    }
}

#[cfg(test)]
#[path = "verification_tests.rs"]
mod tests;
