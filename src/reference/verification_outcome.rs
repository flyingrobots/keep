//! This module maps reference evidence to shared verification outcomes without loss.
#![expect(
    clippy::result_large_err,
    reason = "preserve main's bounded inline diagnostic coordinates without a new refusal allocation"
)]

use super::ReferenceVerificationSource as Source;
use super::{ReferenceVerificationContext as Context, ReferenceVerificationEvidence as Evidence};
use crate::authenticated_read::ChunkVerificationError;
use crate::profile::StorageProfileVerificationError;
use crate::{
    AdmittedLayout, CorruptionEvidence, LayoutId, MissingEvidence, VerificationDepth as Depth,
    VerificationError, VerificationObservation as Observation, VerificationRefusal as Refusal,
    VerificationSource, VerificationSubject as Subject,
};

pub(super) const SUPPORTED: &[Depth] = &[
    Depth::ChunkIdentity,
    Depth::LayoutIdentity,
    Depth::CompleteBlobIdentity,
];

pub(super) fn operational(source: Source) -> VerificationError {
    VerificationError::Operational {
        source: Box::new(VerificationSource::Reference(source)),
    }
}

pub(super) const fn semantic_refusal(context: Context) -> Refusal {
    match context.evidence() {
        Evidence::Missing(evidence) => Refusal::Missing {
            subject: match evidence {
                MissingEvidence::Blob(identity) => Subject::Blob { identity },
                MissingEvidence::Layout(identity) => Subject::Layout { identity },
                MissingEvidence::Chunk { chunk, .. } => Subject::Chunk { identity: chunk },
            },
        },
        Evidence::Corrupt(evidence) => {
            let (expected, observed) = match evidence {
                CorruptionEvidence::ChunkIdentity {
                    expected, observed, ..
                } => (Observation::Chunk(expected), Observation::Chunk(observed)),
                CorruptionEvidence::LayoutIdentity { expected, observed } => {
                    (Observation::Layout(expected), Observation::Layout(observed))
                }
                CorruptionEvidence::BlobIdentity {
                    expected, observed, ..
                } => (Observation::Blob(expected), Observation::Blob(observed)),
                CorruptionEvidence::ProfileBoundary {
                    expected, observed, ..
                } => (
                    Observation::ProfileBoundary(expected),
                    Observation::ProfileBoundary(observed),
                ),
            };
            Refusal::Corrupt {
                subject: context.subject(),
                expected,
                observed,
            }
        }
        Evidence::Unsupported => Refusal::Unsupported {
            subject: context.subject(),
            requested: context.stage(),
            supported: supported(context.subject()),
        },
    }
}

pub(super) fn refused(context: Context) -> VerificationError {
    VerificationError::Refused {
        refusal: semantic_refusal(context),
        source: Some(Box::new(VerificationSource::Reference(Source::Refusal(
            context,
        )))),
    }
}

const fn supported(subject: Subject) -> &'static [Depth] {
    match subject {
        Subject::Blob { .. } | Subject::Layout { .. } => SUPPORTED,
        _ => &[],
    }
}

pub(super) fn require_supported(subject: Subject, depth: Depth) -> Result<(), VerificationError> {
    if supported(subject).contains(&depth) {
        return Ok(());
    }
    Err(refused(Context::new(subject, depth, Evidence::Unsupported)))
}

pub(super) fn missing(subject: Subject, evidence: MissingEvidence) -> VerificationError {
    refused(Context::new(
        subject,
        Depth::ChunkIdentity,
        Evidence::Missing(evidence),
    ))
}

pub(super) fn canonical_identity(layout: &AdmittedLayout) -> Result<LayoutId, VerificationError> {
    layout
        .encode_record()
        .map(|record| record.id())
        .map_err(|source| operational(Source::LayoutEncoding(source)))
}

pub(super) fn require_layout_identity(
    subject: Subject,
    expected: LayoutId,
    layout: &AdmittedLayout,
) -> Result<(), VerificationError> {
    let observed = canonical_identity(layout)?;
    if observed == expected {
        return Ok(());
    }
    Err(refused(Context::new(
        subject,
        Depth::LayoutIdentity,
        Evidence::Corrupt(CorruptionEvidence::LayoutIdentity { expected, observed }),
    )))
}

pub(super) fn chunk_outcome(subject: Subject, error: ChunkVerificationError) -> VerificationError {
    match error {
        ChunkVerificationError::Missing {
            layout,
            index,
            requested,
        } => missing(
            subject,
            MissingEvidence::Chunk {
                layout,
                index,
                chunk: requested,
            },
        ),
        ChunkVerificationError::IdentityMismatch {
            layout,
            index,
            expected,
            observed,
        } => refused(Context::new(
            subject,
            Depth::ChunkIdentity,
            Evidence::Corrupt(CorruptionEvidence::ChunkIdentity {
                layout,
                index,
                expected,
                observed,
            }),
        )),
        ChunkVerificationError::Hash {
            layout,
            index,
            source,
            ..
        } => operational(Source::ChunkHash {
            layout,
            index,
            source,
        }),
    }
}

pub(super) fn profile_outcome(
    subject: Subject,
    layout: LayoutId,
    error: StorageProfileVerificationError,
) -> Result<Context, VerificationError> {
    match error {
        StorageProfileVerificationError::BoundaryMismatch {
            index,
            expected,
            observed,
        } => Ok(Context::new(
            subject,
            Depth::CompleteBlobIdentity,
            Evidence::Corrupt(CorruptionEvidence::ProfileBoundary {
                layout,
                index,
                expected,
                observed,
            }),
        )),
        StorageProfileVerificationError::Unsupported { profile } => {
            Err(operational(Source::ProfileVerifierUnavailable {
                layout,
                profile,
            }))
        }
        StorageProfileVerificationError::Chunking { source } => {
            Err(operational(Source::ProfileChunking { layout, source }))
        }
    }
}

pub(super) fn profile_failure(
    subject: Subject,
    layout: LayoutId,
    error: StorageProfileVerificationError,
) -> VerificationError {
    match profile_outcome(subject, layout, error) {
        Ok(context) => refused(context),
        Err(error) => error,
    }
}
