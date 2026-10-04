//! This boundary module owns canonical verification receipt encoding.

use super::ReceiptSubject as VerificationSubject;
use super::enums::{
    ReceiptEvidenceKind, ReceiptOutcomeKind, ReceiptRefusalClass, ReceiptSubjectKind,
    ReceiptViewKind, depth_code,
};
use super::format::{self, ENCODED_LENGTH};
use super::receipt::{
    ReceiptCorruption, ReceiptMissing, ReceiptRefusal, VERIFICATION_CONTRACT_VERSION,
    VerificationOutcome, VerificationReceipt, VerificationView,
};
use crate::adapters::GcRetentionState;
use crate::{BlobId, LayoutId};

/// The scalar fields of one receipt, in wire order.
struct Header {
    outcome: ReceiptOutcomeKind,
    depth: u16,
    subject_kind: ReceiptSubjectKind,
    view_kind: ReceiptViewKind,
    refusal_class: ReceiptRefusalClass,
    evidence_kind: ReceiptEvidenceKind,
    layout: Option<LayoutId>,
    supported_minimum: u16,
    supported_maximum: u16,
    target: Option<BlobId>,
    evidence_index: u64,
    chunks_verified: u64,
}

pub(super) fn encode(receipt: &VerificationReceipt) -> [u8; ENCODED_LENGTH] {
    let header = header(receipt);
    let mut encoded = [0_u8; ENCODED_LENGTH];
    write(&mut encoded, 0, &format::MAGIC);
    write(&mut encoded, 16, &format::VERSION.to_be_bytes());
    write(&mut encoded, 18, &format::RECORD_LENGTH.to_be_bytes());
    write(
        &mut encoded,
        24,
        &VERIFICATION_CONTRACT_VERSION.to_be_bytes(),
    );
    write(&mut encoded, 28, &header.outcome.code().to_be_bytes());
    write(&mut encoded, 30, &header.depth.to_be_bytes());
    write(&mut encoded, 32, &header.subject_kind.code().to_be_bytes());
    write(&mut encoded, 34, &header.view_kind.code().to_be_bytes());
    write(&mut encoded, 36, &header.refusal_class.code().to_be_bytes());
    write(&mut encoded, 38, &header.evidence_kind.code().to_be_bytes());
    write(
        &mut encoded,
        40,
        &u16::from(header.layout.is_some()).to_be_bytes(),
    );
    write(&mut encoded, 42, &header.supported_minimum.to_be_bytes());
    write(&mut encoded, 44, &header.supported_maximum.to_be_bytes());
    write(
        &mut encoded,
        46,
        &u16::from(header.target.is_some()).to_be_bytes(),
    );
    write_subject(&mut encoded, receipt.subject());
    if let Some(layout) = header.layout {
        write(&mut encoded, format::LAYOUT_OFFSET, &layout.encode_binary());
    }
    if let Some(target) = header.target {
        write(&mut encoded, format::TARGET_OFFSET, &target.encode_binary());
    }
    write_view(&mut encoded, receipt.view());
    write(
        &mut encoded,
        format::EVIDENCE_INDEX_OFFSET,
        &header.evidence_index.to_be_bytes(),
    );
    write(
        &mut encoded,
        format::CHUNKS_VERIFIED_OFFSET,
        &header.chunks_verified.to_be_bytes(),
    );
    let checksum = format::checksum(encoded.get(..format::CHECKSUM_OFFSET).unwrap_or_default());
    write(&mut encoded, format::CHECKSUM_OFFSET, &checksum);
    encoded
}

fn write(encoded: &mut [u8; ENCODED_LENGTH], offset: usize, bytes: &[u8]) {
    if let Some(slot) = offset
        .checked_add(bytes.len())
        .and_then(|end| encoded.get_mut(offset..end))
    {
        slot.copy_from_slice(bytes);
    }
}

fn write_subject(encoded: &mut [u8; ENCODED_LENGTH], subject: VerificationSubject) {
    match subject {
        VerificationSubject::Blob(blob) => {
            write(encoded, format::SUBJECT_OFFSET, &blob.encode_binary());
        }
        VerificationSubject::Layout(layout) => {
            write(encoded, format::SUBJECT_OFFSET, &layout.encode_binary());
        }
    }
}

fn write_view(encoded: &mut [u8; ENCODED_LENGTH], view: VerificationView) {
    let VerificationView::Durable {
        catalog_generation,
        catalog_digest,
        retention,
    } = view
    else {
        return;
    };
    write(
        encoded,
        format::CATALOG_GENERATION_OFFSET,
        &catalog_generation.get().to_be_bytes(),
    );
    write(
        encoded,
        format::CATALOG_DIGEST_OFFSET,
        catalog_digest.as_bytes(),
    );
    if let GcRetentionState::Published {
        generation,
        manifest_digest,
    } = retention
    {
        write(
            encoded,
            format::LIVENESS_GENERATION_OFFSET,
            &generation.get().to_be_bytes(),
        );
        write(
            encoded,
            format::MANIFEST_DIGEST_OFFSET,
            manifest_digest.as_bytes(),
        );
    }
}

const fn header(receipt: &VerificationReceipt) -> Header {
    let subject_kind = match receipt.subject() {
        VerificationSubject::Blob(_) => ReceiptSubjectKind::Blob,
        VerificationSubject::Layout(_) => ReceiptSubjectKind::Layout,
    };
    let view_kind = match receipt.view() {
        VerificationView::Reference => ReceiptViewKind::Reference,
        VerificationView::Durable { .. } => ReceiptViewKind::Durable,
    };
    let mut header = Header {
        outcome: ReceiptOutcomeKind::Report,
        depth: 0,
        subject_kind,
        view_kind,
        refusal_class: ReceiptRefusalClass::None,
        evidence_kind: ReceiptEvidenceKind::None,
        layout: None,
        supported_minimum: 0,
        supported_maximum: 0,
        target: None,
        evidence_index: 0,
        chunks_verified: 0,
    };
    match receipt.outcome() {
        VerificationOutcome::Established {
            depth,
            layout,
            target,
            chunks_verified,
            ..
        } => {
            header.depth = depth_code(depth);
            header.layout = Some(layout);
            header.target = Some(target);
            header.chunks_verified = chunks_verified;
        }
        VerificationOutcome::Refused(refusal) => {
            header.outcome = ReceiptOutcomeKind::Refusal;
            refusal_header(&mut header, refusal);
        }
    }
    header
}

const fn refusal_header(header: &mut Header, refusal: ReceiptRefusal) {
    match refusal {
        ReceiptRefusal::Missing {
            stage, evidence, ..
        } => {
            header.depth = depth_code(stage);
            header.refusal_class = ReceiptRefusalClass::Missing;
            let (kind, layout, index) = match evidence {
                ReceiptMissing::Blob => (ReceiptEvidenceKind::First, None, 0),
                ReceiptMissing::Layout => (ReceiptEvidenceKind::Second, None, 0),
                ReceiptMissing::Chunk { layout, index } => {
                    (ReceiptEvidenceKind::Third, Some(layout), index)
                }
            };
            header.evidence_kind = kind;
            header.layout = layout;
            header.evidence_index = index;
        }
        ReceiptRefusal::Corrupt {
            stage, evidence, ..
        } => {
            header.depth = depth_code(stage);
            header.refusal_class = ReceiptRefusalClass::Corrupt;
            let (kind, layout, index) = match evidence {
                ReceiptCorruption::ChunkIdentity { layout, index } => {
                    (ReceiptEvidenceKind::First, layout, index)
                }
                ReceiptCorruption::LayoutIdentity { expected } => {
                    (ReceiptEvidenceKind::Second, expected, 0)
                }
                ReceiptCorruption::BlobIdentity { layout } => {
                    (ReceiptEvidenceKind::Third, layout, 0)
                }
                ReceiptCorruption::ProfileBoundary { layout, index } => {
                    (ReceiptEvidenceKind::Fourth, layout, index)
                }
            };
            header.evidence_kind = kind;
            header.layout = Some(layout);
            header.evidence_index = index;
        }
        ReceiptRefusal::Ambiguous { stage, .. } => {
            header.depth = depth_code(stage);
            header.refusal_class = ReceiptRefusalClass::Ambiguous;
        }
        ReceiptRefusal::Unsupported {
            requested,
            supported_minimum,
            supported_maximum,
            ..
        } => {
            header.depth = depth_code(requested);
            header.refusal_class = ReceiptRefusalClass::Unsupported;
            header.supported_minimum = depth_code(supported_minimum);
            header.supported_maximum = depth_code(supported_maximum);
        }
    }
}
