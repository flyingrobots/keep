//! Canonical binary `LayoutId` codec.

use super::layout_id_binary_error::LayoutIdBinaryParseError;
use crate::{LayoutId, LayoutRecordLength};

const BINARY_MAGIC: [u8; 16] = *b"KEEP:LAYOUT:ID\0\0";
const IDENTITY_VERSION: u16 = 1;
const LAYOUT_CODEC: u16 = 1;
const BINARY_BYTES: usize = 60;

impl LayoutId {
    /// Number of bytes in the canonical version-1 binary representation.
    pub const BINARY_LENGTH: usize = BINARY_BYTES;

    /// Encodes this identity into its fixed canonical binary representation.
    ///
    /// The operation is allocation-free.
    #[must_use]
    pub const fn encode_binary(self) -> [u8; BINARY_BYTES] {
        let mut encoded = [0_u8; BINARY_BYTES];
        let (magic_slot, rest) = encoded.split_at_mut(BINARY_MAGIC.len());
        magic_slot.copy_from_slice(&BINARY_MAGIC);
        let (version_slot, rest) = rest.split_at_mut(2);
        version_slot.copy_from_slice(&IDENTITY_VERSION.to_be_bytes());
        let (codec_slot, rest) = rest.split_at_mut(2);
        codec_slot.copy_from_slice(&LAYOUT_CODEC.to_be_bytes());
        let (length_slot, digest_slot) = rest.split_at_mut(8);
        length_slot.copy_from_slice(&self.plan_length().get().to_be_bytes());
        digest_slot.copy_from_slice(self.digest());
        encoded
    }

    /// Parses the exact canonical binary representation.
    ///
    /// Parsing validates coordinate framing and support only.
    ///
    /// # Errors
    ///
    /// Returns [`LayoutIdBinaryParseError`] for wrong width, magic, version,
    /// codec, or plan-length law.
    pub fn parse_binary(encoded: &[u8]) -> Result<Self, LayoutIdBinaryParseError> {
        validate_binary_length(encoded.len())?;
        let Some((magic, remainder)) = encoded.split_first_chunk::<16>() else {
            return Err(wrong_length(encoded.len()));
        };
        if magic != &BINARY_MAGIC {
            return Err(LayoutIdBinaryParseError::InvalidMagic { observed: *magic });
        }
        let (version, remainder) = read_u16(remainder, encoded.len())?;
        validate_version(version)?;
        let (codec, remainder) = read_u16(remainder, encoded.len())?;
        validate_codec(codec)?;
        let (length_bytes, remainder) = remainder
            .split_first_chunk::<8>()
            .ok_or_else(|| wrong_length(encoded.len()))?;
        let plan_length = validate_plan_length(u64::from_be_bytes(*length_bytes))?;
        let (digest, trailing) = remainder
            .split_first_chunk::<32>()
            .ok_or_else(|| wrong_length(encoded.len()))?;
        if !trailing.is_empty() {
            return Err(wrong_length(encoded.len()));
        }
        Ok(Self::from_validated_parts(plan_length, *digest))
    }
}

const fn read_u16(
    remainder: &[u8],
    observed: usize,
) -> Result<(u16, &[u8]), LayoutIdBinaryParseError> {
    let Some((bytes, trailing)) = remainder.split_first_chunk::<2>() else {
        return Err(wrong_length(observed));
    };
    Ok((u16::from_be_bytes(*bytes), trailing))
}

const fn validate_binary_length(observed: usize) -> Result<(), LayoutIdBinaryParseError> {
    if observed == BINARY_BYTES {
        return Ok(());
    }
    Err(wrong_length(observed))
}

const fn wrong_length(observed: usize) -> LayoutIdBinaryParseError {
    LayoutIdBinaryParseError::WrongLength {
        expected: BINARY_BYTES,
        observed,
    }
}

const fn validate_version(observed: u16) -> Result<(), LayoutIdBinaryParseError> {
    if observed == IDENTITY_VERSION {
        return Ok(());
    }
    Err(LayoutIdBinaryParseError::UnsupportedVersion {
        expected: IDENTITY_VERSION,
        observed,
    })
}

const fn validate_codec(observed: u16) -> Result<(), LayoutIdBinaryParseError> {
    if observed == LAYOUT_CODEC {
        return Ok(());
    }
    Err(LayoutIdBinaryParseError::UnsupportedCodec {
        expected: LAYOUT_CODEC,
        observed,
    })
}

fn validate_plan_length(observed: u64) -> Result<LayoutRecordLength, LayoutIdBinaryParseError> {
    if !(LayoutRecordLength::MINIMUM..=LayoutRecordLength::MAXIMUM).contains(&observed) {
        return Err(LayoutIdBinaryParseError::PlanLengthOutOfBounds {
            minimum: LayoutRecordLength::MINIMUM,
            maximum: LayoutRecordLength::MAXIMUM,
            observed,
        });
    }
    LayoutRecordLength::from_wire(observed)
        .ok_or(LayoutIdBinaryParseError::PlanLengthNotCongruent { observed })
}
