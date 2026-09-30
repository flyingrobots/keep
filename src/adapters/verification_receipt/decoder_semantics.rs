//! This boundary module owns the semantic laws that turn admitted receipt
//! fields into one `VerificationReceipt`, refusing every contradiction.

use super::decode_error::{
    VerificationReceiptDecodeError as Error, VerificationReceiptField as Field,
};
use super::decoder::{Fields, blob_slot, layout_slot, zero_slot};
use super::enums::{
    ReceiptEvidenceKind, ReceiptOutcomeKind, ReceiptRefusalClass, ReceiptSubjectKind,
    ReceiptViewKind, depth_from_code,
};
use super::receipt::{
    ReceiptCorruption, ReceiptMissing, ReceiptRefusal, VerificationOutcome, VerificationReceipt,
    VerificationView,
};
use crate::adapters::GcRetentionState;
use crate::{
    CatalogDigest, CatalogGeneration, LayoutId, LivenessGeneration, RetentionManifestDigest,
    VerificationDepth, VerificationSubject,
};

const fn law(law: &'static str) -> Error {
    Error::Semantic { law }
}

pub(super) fn admit(fields: &Fields) -> Result<VerificationReceipt, Error> {
    let subject = match fields.subject_kind {
        ReceiptSubjectKind::Blob => VerificationSubject::Blob(blob_slot(&fields.subject_slot)?),
        ReceiptSubjectKind::Layout => {
            VerificationSubject::Layout(layout_slot(&fields.subject_slot)?)
        }
    };
    let depth = depth_from_code(fields.depth).ok_or(Error::UnregisteredCode {
        field: Field::Depth,
        observed: fields.depth,
    })?;
    let view = view(fields)?;
    let layout = if fields.layout_present {
        Some(layout_slot(&fields.layout_slot)?)
    } else {
        zero_slot(&fields.layout_slot, "absent layout slot must be zero")?;
        None
    };
    let outcome = match fields.outcome {
        ReceiptOutcomeKind::Report => report(fields, subject, depth, layout)?,
        ReceiptOutcomeKind::Refusal => {
            if fields.target_present {
                return Err(law("a refusal names no target"));
            }
            zero_slot(&fields.target_slot, "absent target slot must be zero")?;
            if fields.chunks_verified != 0 {
                return Err(law("a refusal verified no chunks"));
            }
            VerificationOutcome::Refused(refusal(fields, subject, depth, layout)?)
        }
    };
    Ok(VerificationReceipt::from_parts(view, outcome))
}

fn view(fields: &Fields) -> Result<VerificationView, Error> {
    match fields.view_kind {
        ReceiptViewKind::Reference => {
            if fields.catalog_generation != 0 || fields.liveness_generation != 0 {
                return Err(law("a reference view binds no generation"));
            }
            if fields.catalog_digest != [0_u8; 32] || fields.manifest_digest != [0_u8; 32] {
                return Err(law("a reference view binds no digest"));
            }
            Ok(VerificationView::Reference)
        }
        ReceiptViewKind::Durable => {
            let catalog_generation =
                CatalogGeneration::new(fields.catalog_generation).map_err(|_source| {
                    Error::NonZero {
                        field: Field::CatalogGeneration,
                    }
                })?;
            let retention = if fields.liveness_generation == 0 {
                if fields.manifest_digest != [0_u8; 32] {
                    return Err(law("empty retention binds no manifest digest"));
                }
                GcRetentionState::Empty
            } else {
                GcRetentionState::Published {
                    generation: LivenessGeneration::new(fields.liveness_generation).map_err(
                        |_source| Error::NonZero {
                            field: Field::LivenessGeneration,
                        },
                    )?,
                    manifest_digest: RetentionManifestDigest::from_hash(fields.manifest_digest),
                }
            };
            Ok(VerificationView::Durable {
                catalog_generation,
                catalog_digest: CatalogDigest::from_validated(fields.catalog_digest),
                retention,
            })
        }
    }
}

fn report(
    fields: &Fields,
    subject: VerificationSubject,
    depth: VerificationDepth,
    layout: Option<LayoutId>,
) -> Result<VerificationOutcome, Error> {
    if fields.refusal_class != ReceiptRefusalClass::None
        || fields.evidence_kind != ReceiptEvidenceKind::None
        || fields.evidence_index != 0
        || fields.supported_minimum != 0
        || fields.supported_maximum != 0
    {
        return Err(law("a report carries no refusal coordinates"));
    }
    let layout = layout.ok_or(law("a report names the layout it established"))?;
    if !fields.target_present {
        return Err(law("a report names the target it established"));
    }
    let target = blob_slot(&fields.target_slot)?;
    if let VerificationSubject::Blob(blob) = subject
        && blob != target
    {
        return Err(law("a blob report's target is its subject"));
    }
    if let VerificationSubject::Layout(subject_layout) = subject
        && subject_layout != layout
    {
        return Err(law("a layout report's layout is its subject"));
    }
    Ok(VerificationOutcome::Established {
        subject,
        depth,
        layout,
        target,
        chunks_verified: fields.chunks_verified,
    })
}

fn refusal(
    fields: &Fields,
    subject: VerificationSubject,
    stage: VerificationDepth,
    layout: Option<LayoutId>,
) -> Result<ReceiptRefusal, Error> {
    let bounds_zero = fields.supported_minimum == 0 && fields.supported_maximum == 0;
    match fields.refusal_class {
        ReceiptRefusalClass::None => Err(law("a refusal carries its classification")),
        ReceiptRefusalClass::Missing => {
            require(bounds_zero, "only unsupported carries a supported range")?;
            let evidence = match (fields.evidence_kind, layout) {
                (ReceiptEvidenceKind::First, None) => ReceiptMissing::Blob,
                (ReceiptEvidenceKind::Second, None) => ReceiptMissing::Layout,
                (ReceiptEvidenceKind::Third, Some(layout)) => ReceiptMissing::Chunk {
                    layout,
                    index: fields.evidence_index,
                },
                _ => return Err(law("missing evidence kind disagrees with its layout")),
            };
            if !matches!(evidence, ReceiptMissing::Chunk { .. }) && fields.evidence_index != 0 {
                return Err(law("only a missing chunk carries an index"));
            }
            Ok(ReceiptRefusal::Missing {
                subject,
                stage,
                evidence,
            })
        }
        ReceiptRefusalClass::Corrupt => {
            require(bounds_zero, "only unsupported carries a supported range")?;
            let layout = layout.ok_or(law("a corruption names its layout"))?;
            let index = fields.evidence_index;
            let evidence = match fields.evidence_kind {
                ReceiptEvidenceKind::First => ReceiptCorruption::ChunkIdentity { layout, index },
                ReceiptEvidenceKind::Second => {
                    ReceiptCorruption::LayoutIdentity { expected: layout }
                }
                ReceiptEvidenceKind::Third => ReceiptCorruption::BlobIdentity { layout },
                ReceiptEvidenceKind::Fourth => ReceiptCorruption::ProfileBoundary { layout, index },
                ReceiptEvidenceKind::None => return Err(law("a corruption names its kind")),
            };
            let indexed = matches!(
                evidence,
                ReceiptCorruption::ChunkIdentity { .. } | ReceiptCorruption::ProfileBoundary { .. }
            );
            if !indexed && index != 0 {
                return Err(law("only an indexed corruption carries an index"));
            }
            Ok(ReceiptRefusal::Corrupt {
                subject,
                stage,
                evidence,
            })
        }
        ReceiptRefusalClass::Ambiguous => {
            require(bounds_zero, "only unsupported carries a supported range")?;
            require(
                fields.evidence_kind == ReceiptEvidenceKind::None
                    && fields.evidence_index == 0
                    && layout.is_none(),
                "an ambiguity names no evidence",
            )?;
            Ok(ReceiptRefusal::Ambiguous { subject, stage })
        }
        ReceiptRefusalClass::Unsupported => unsupported(fields, subject, stage, layout),
    }
}

fn unsupported(
    fields: &Fields,
    subject: VerificationSubject,
    requested: VerificationDepth,
    layout: Option<LayoutId>,
) -> Result<ReceiptRefusal, Error> {
    require(
        fields.evidence_kind == ReceiptEvidenceKind::None
            && fields.evidence_index == 0
            && layout.is_none(),
        "unsupported names no evidence",
    )?;
    let supported_minimum =
        depth_from_code(fields.supported_minimum).ok_or(Error::UnregisteredCode {
            field: Field::SupportedMinimum,
            observed: fields.supported_minimum,
        })?;
    let supported_maximum =
        depth_from_code(fields.supported_maximum).ok_or(Error::UnregisteredCode {
            field: Field::SupportedMaximum,
            observed: fields.supported_maximum,
        })?;
    require(
        supported_minimum <= supported_maximum,
        "the supported range is ordered",
    )?;
    require(
        requested < supported_minimum || requested > supported_maximum,
        "an unsupported depth lies outside the supported range",
    )?;
    Ok(ReceiptRefusal::Unsupported {
        subject,
        requested,
        supported_minimum,
        supported_maximum,
    })
}

const fn require(condition: bool, violated: &'static str) -> Result<(), Error> {
    if condition {
        Ok(())
    } else {
        Err(law(violated))
    }
}
