//! This boundary module owns verification receipt field decoding: framing,
//! contract, checksum, registered codes, and identity slots, in that order.

use super::decode_error::{
    VerificationReceiptDecodeError as Error, VerificationReceiptField as Field,
};
use super::enums::{
    ReceiptEvidenceKind, ReceiptOutcomeKind, ReceiptRefusalClass, ReceiptSubjectKind,
    ReceiptViewKind,
};
use super::format::{self, ENCODED_LENGTH};
use super::receipt::VERIFICATION_CONTRACT_VERSION;
use crate::{BlobId, LayoutId};

/// Every wire field of one receipt, framing-admitted and checksummed but not
/// yet held to the semantic laws.
pub(super) struct Fields {
    pub(super) outcome: ReceiptOutcomeKind,
    pub(super) depth: u16,
    pub(super) subject_kind: ReceiptSubjectKind,
    pub(super) view_kind: ReceiptViewKind,
    pub(super) refusal_class: ReceiptRefusalClass,
    pub(super) evidence_kind: ReceiptEvidenceKind,
    pub(super) layout_present: bool,
    pub(super) supported_minimum: u16,
    pub(super) supported_maximum: u16,
    pub(super) target_present: bool,
    pub(super) subject_slot: [u8; format::IDENTITY_SLOT],
    pub(super) layout_slot: [u8; format::IDENTITY_SLOT],
    pub(super) target_slot: [u8; format::IDENTITY_SLOT],
    pub(super) catalog_generation: u64,
    pub(super) catalog_digest: [u8; 32],
    pub(super) liveness_generation: u64,
    pub(super) manifest_digest: [u8; 32],
    pub(super) evidence_index: u64,
    pub(super) chunks_verified: u64,
}

pub(super) fn decode(encoded: &[u8]) -> Result<Fields, Error> {
    if encoded.len() != ENCODED_LENGTH {
        return Err(Error::WrongLength {
            expected: ENCODED_LENGTH,
            observed: encoded.len(),
        });
    }
    validate_framing(encoded)?;
    verify_checksum(encoded)?;
    let fields = Fields {
        outcome: registered(encoded, 28, Field::Outcome, ReceiptOutcomeKind::from_code)?,
        depth: u16_at(encoded, 30)?,
        subject_kind: registered(
            encoded,
            32,
            Field::SubjectKind,
            ReceiptSubjectKind::from_code,
        )?,
        view_kind: registered(encoded, 34, Field::ViewKind, ReceiptViewKind::from_code)?,
        refusal_class: registered(
            encoded,
            36,
            Field::RefusalClass,
            ReceiptRefusalClass::from_code,
        )?,
        evidence_kind: registered(
            encoded,
            38,
            Field::EvidenceKind,
            ReceiptEvidenceKind::from_code,
        )?,
        layout_present: flag(encoded, 40, Field::LayoutPresent)?,
        supported_minimum: u16_at(encoded, 42)?,
        supported_maximum: u16_at(encoded, 44)?,
        target_present: flag(encoded, 46, Field::TargetPresent)?,
        subject_slot: array(encoded, format::SUBJECT_OFFSET)?,
        layout_slot: array(encoded, format::LAYOUT_OFFSET)?,
        target_slot: array(encoded, format::TARGET_OFFSET)?,
        catalog_generation: u64_at(encoded, format::CATALOG_GENERATION_OFFSET)?,
        catalog_digest: array(encoded, format::CATALOG_DIGEST_OFFSET)?,
        liveness_generation: u64_at(encoded, format::LIVENESS_GENERATION_OFFSET)?,
        manifest_digest: array(encoded, format::MANIFEST_DIGEST_OFFSET)?,
        evidence_index: u64_at(encoded, format::EVIDENCE_INDEX_OFFSET)?,
        chunks_verified: u64_at(encoded, format::CHUNKS_VERIFIED_OFFSET)?,
    };
    let reserved: [u8; format::RESERVED_LENGTH] = array(encoded, format::RESERVED_OFFSET)?;
    if reserved != [0_u8; format::RESERVED_LENGTH] {
        return Err(Error::NonZero {
            field: Field::Reserved,
        });
    }
    Ok(fields)
}

fn validate_framing(encoded: &[u8]) -> Result<(), Error> {
    let magic: [u8; 16] = array(encoded, 0)?;
    if magic != format::MAGIC {
        return Err(Error::InvalidMagic { observed: magic });
    }
    let version = u16_at(encoded, 16)?;
    if version != format::VERSION {
        return Err(Error::UnsupportedVersion {
            expected: format::VERSION,
            observed: version,
        });
    }
    let record_length = u16_at(encoded, 18)?;
    if record_length != format::RECORD_LENGTH {
        return Err(Error::InvalidRecordLength {
            expected: format::RECORD_LENGTH,
            observed: record_length,
        });
    }
    let flags = u32::from_be_bytes(array(encoded, 20)?);
    if flags != 0 {
        return Err(Error::UnsupportedFlags { observed: flags });
    }
    let contract = u32::from_be_bytes(array(encoded, 24)?);
    if contract != VERIFICATION_CONTRACT_VERSION {
        return Err(Error::UnsupportedContract {
            expected: VERIFICATION_CONTRACT_VERSION,
            observed: contract,
        });
    }
    Ok(())
}

fn verify_checksum(encoded: &[u8]) -> Result<(), Error> {
    let preimage = encoded
        .get(..format::CHECKSUM_OFFSET)
        .ok_or(Error::WrongLength {
            expected: ENCODED_LENGTH,
            observed: encoded.len(),
        })?;
    let observed: [u8; 32] = array(encoded, format::CHECKSUM_OFFSET)?;
    let expected = format::checksum(preimage);
    if observed == expected {
        Ok(())
    } else {
        Err(Error::ChecksumMismatch { expected, observed })
    }
}

fn registered<T>(
    encoded: &[u8],
    offset: usize,
    field: Field,
    from_code: fn(u16) -> Option<T>,
) -> Result<T, Error> {
    let observed = u16_at(encoded, offset)?;
    from_code(observed).ok_or(Error::UnregisteredCode { field, observed })
}

fn flag(encoded: &[u8], offset: usize, field: Field) -> Result<bool, Error> {
    match u16_at(encoded, offset)? {
        0 => Ok(false),
        1 => Ok(true),
        observed => Err(Error::UnregisteredCode { field, observed }),
    }
}

fn array<const WIDTH: usize>(encoded: &[u8], offset: usize) -> Result<[u8; WIDTH], Error> {
    offset
        .checked_add(WIDTH)
        .and_then(|end| encoded.get(offset..end))
        .and_then(|slice| <[u8; WIDTH]>::try_from(slice).ok())
        .ok_or(Error::WrongLength {
            expected: ENCODED_LENGTH,
            observed: encoded.len(),
        })
}

fn u16_at(encoded: &[u8], offset: usize) -> Result<u16, Error> {
    array(encoded, offset).map(u16::from_be_bytes)
}

fn u64_at(encoded: &[u8], offset: usize) -> Result<u64, Error> {
    array(encoded, offset).map(u64::from_be_bytes)
}

/// Parses a 59-byte `BlobId` binary from a zero-padded 60-byte slot.
pub(super) fn blob_slot(slot: &[u8; format::IDENTITY_SLOT]) -> Result<BlobId, Error> {
    let (binary, padding) = slot.split_at(BlobId::BINARY_LENGTH);
    if padding != [0_u8] {
        return Err(Error::Semantic {
            law: "blob identity slot padding must be zero",
        });
    }
    BlobId::parse_binary(binary).map_err(|source| Error::BlobId { source })
}

/// Parses a 60-byte `LayoutId` binary slot.
pub(super) fn layout_slot(slot: &[u8; format::IDENTITY_SLOT]) -> Result<LayoutId, Error> {
    LayoutId::parse_binary(slot).map_err(|source| Error::LayoutId { source })
}

/// Requires an absent identity slot to be all zero.
pub(super) fn zero_slot(
    slot: &[u8; format::IDENTITY_SLOT],
    law: &'static str,
) -> Result<(), Error> {
    if *slot == [0_u8; format::IDENTITY_SLOT] {
        Ok(())
    } else {
        Err(Error::Semantic { law })
    }
}
