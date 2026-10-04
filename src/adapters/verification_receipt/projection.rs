//! This module owns checked projection from live evidence to the frozen v1 vocabulary.

use super::{
    ReceiptCorruption, ReceiptMissing, ReceiptRefusal, ReceiptSubject, ReceiptVerificationDepth,
    VerificationOutcome, VerificationReceipt, VerificationReceiptProjectionError as Error,
    VerificationView,
};
use crate::{
    CorruptionEvidence, MissingEvidence, ReferenceVerificationEvidence as Evidence,
    ReferenceVerificationSource, VerificationDepth, VerificationError, VerificationRefusal,
    VerificationReport, VerificationSource,
};

impl VerificationReceipt {
    /// Projects a reference report without allowing its provenance to be relabeled.
    ///
    /// Receipt v1 has only Blob/Layout identity slots and no `SnapshotBinding` code.
    /// Other reports remain valid verification evidence even when v1 cannot carry them.
    /// This operation does not perform I/O or establish fresh verification.
    ///
    /// # Errors
    ///
    /// Refuses unregistered subjects/depths, absent layout-work evidence, or a view
    /// that does not match the report's actual reference origin.
    pub fn from_report(report: &VerificationReport, view: VerificationView) -> Result<Self, Error> {
        let claim = report
            .subjects()
            .first()
            .ok_or(Error::LayoutEvidenceRequired)?;
        let subject = ReceiptSubject::try_from(claim.subject())?;
        let depth = ReceiptVerificationDepth::try_from(claim.depth())?;
        let details = report
            .reference_details()
            .ok_or(Error::LayoutEvidenceRequired)?;
        if view != VerificationView::Reference
            || report.catalog().is_some()
            || report.retention_head().is_some()
        {
            return Err(Error::ViewMismatch);
        }
        Ok(Self::from_parts(
            view,
            VerificationOutcome::Established {
                subject,
                depth,
                layout: details.layout(),
                target: details.target(),
                chunks_verified: details.chunks_verified(),
            },
        ))
    }

    /// Projects an actual reference refusal with its admitted request/stage context.
    ///
    /// A shared semantic refusal alone does not identify the operation's view,
    /// selected layout or stage. Operational failures produce no refusal receipt.
    /// Decoding historical durable receipt bytes remains a separate codec operation;
    /// it never manufactures a live verification report.
    ///
    /// # Errors
    ///
    /// Refuses missing or inconsistent context, view substitution, unregistered v1
    /// coordinates and supported sets that cannot be represented by v1's exact interval.
    pub fn from_error(error: &VerificationError, view: VerificationView) -> Result<Self, Error> {
        let VerificationError::Refused { refusal, source } = error else {
            return Err(Error::OperationalFailure);
        };
        let Some(VerificationSource::Reference(ReferenceVerificationSource::Refusal(context))) =
            source.as_deref()
        else {
            return Err(Error::RefusalContextRequired);
        };
        if context.semantic_refusal() != *refusal {
            return Err(Error::RefusalContextRequired);
        }
        if view != VerificationView::Reference {
            return Err(Error::ViewMismatch);
        }
        let subject = ReceiptSubject::try_from(context.subject())?;
        let stage = ReceiptVerificationDepth::try_from(context.stage())?;
        let projected = match context.evidence() {
            Evidence::Missing(evidence) => ReceiptRefusal::Missing {
                subject,
                stage,
                evidence: missing(evidence)?,
            },
            Evidence::Corrupt(evidence) => ReceiptRefusal::Corrupt {
                subject,
                stage,
                evidence: corrupt(evidence)?,
            },
            Evidence::Unsupported => unsupported(refusal, subject, stage)?,
        };
        Ok(Self::from_parts(
            view,
            VerificationOutcome::Refused(projected),
        ))
    }
}

fn missing(evidence: MissingEvidence) -> Result<ReceiptMissing, Error> {
    Ok(match evidence {
        MissingEvidence::Blob(_) => ReceiptMissing::Blob,
        MissingEvidence::Layout(_) => ReceiptMissing::Layout,
        MissingEvidence::Chunk { layout, index, .. } => ReceiptMissing::Chunk {
            layout,
            index: index_u64(index)?,
        },
    })
}

fn corrupt(evidence: CorruptionEvidence) -> Result<ReceiptCorruption, Error> {
    Ok(match evidence {
        CorruptionEvidence::ChunkIdentity { layout, index, .. } => {
            ReceiptCorruption::ChunkIdentity {
                layout,
                index: index_u64(index)?,
            }
        }
        CorruptionEvidence::LayoutIdentity { expected, .. } => {
            ReceiptCorruption::LayoutIdentity { expected }
        }
        CorruptionEvidence::BlobIdentity { layout, .. } => {
            ReceiptCorruption::BlobIdentity { layout }
        }
        CorruptionEvidence::ProfileBoundary { layout, index, .. } => {
            ReceiptCorruption::ProfileBoundary {
                layout,
                index: index_u64(index)?,
            }
        }
    })
}

fn unsupported(
    refusal: &VerificationRefusal,
    subject: ReceiptSubject,
    requested: ReceiptVerificationDepth,
) -> Result<ReceiptRefusal, Error> {
    let VerificationRefusal::Unsupported { supported, .. } = refusal else {
        return Err(Error::RefusalContextRequired);
    };
    let (supported_minimum, supported_maximum) = supported_interval(supported)?;
    if (supported_minimum.code()..=supported_maximum.code()).contains(&requested.code()) {
        return Err(Error::SupportedSetNotRepresentable);
    }
    Ok(ReceiptRefusal::Unsupported {
        subject,
        requested,
        supported_minimum,
        supported_maximum,
    })
}

fn supported_interval(
    supported: &[VerificationDepth],
) -> Result<(ReceiptVerificationDepth, ReceiptVerificationDepth), Error> {
    if supported.is_empty() || supported.len() > ReceiptVerificationDepth::ALL.len() {
        return Err(Error::SupportedSetNotRepresentable);
    }
    let mut codes = supported
        .iter()
        .copied()
        .map(ReceiptVerificationDepth::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    codes.sort_by_key(|depth| depth.code());
    for pair in codes.windows(2) {
        let [left, right] = pair else {
            return Err(Error::SupportedSetNotRepresentable);
        };
        if left.code().checked_add(1) != Some(right.code()) {
            return Err(Error::SupportedSetNotRepresentable);
        }
    }
    Ok((
        *codes.first().ok_or(Error::SupportedSetNotRepresentable)?,
        *codes.last().ok_or(Error::SupportedSetNotRepresentable)?,
    ))
}

fn index_u64(index: usize) -> Result<u64, Error> {
    u64::try_from(index).map_err(|_source| Error::IndexOverflow)
}
