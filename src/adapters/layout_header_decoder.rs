//! Fixed-width outer flat-layout header decoding.

use super::LayoutDecodeError;
use super::layout_record_format::{
    CHECKSUM_ALGORITHM, CHUNK_HASH_ALGORITHM, CHUNK_IDENTITY_VERSION, ENTRY_LENGTH, FLAGS,
    FORMAT_VERSION, HEADER_LENGTH, LAYOUT_CODEC, MAGIC,
};

pub(super) struct DecodedHeader {
    pub(super) record_length: u64,
    pub(super) entry_count: u32,
    pub(super) target_blob_id: [u8; 59],
    pub(super) profile_identity_version: u16,
    pub(super) profile_hash_algorithm: u8,
    pub(super) profile_digest: [u8; 32],
}

pub(super) fn decode_header(encoded: &[u8]) -> Result<DecodedHeader, LayoutDecodeError> {
    let header_width = usize::from(HEADER_LENGTH);
    let Some(header) = encoded.get(..header_width) else {
        return Err(LayoutDecodeError::TruncatedHeader {
            expected: header_width,
            observed: encoded.len(),
        });
    };
    let mut cursor = HeaderCursor::new(header, encoded.len());
    validate_magic(cursor.take::<16>()?)?;
    validate_format_version(cursor.take_u16()?)?;
    validate_codec(cursor.take_u16()?)?;
    validate_flags(cursor.take_u32()?)?;
    validate_header_length(cursor.take_u16()?)?;
    validate_entry_length(cursor.take_u16()?)?;
    let record_length = cursor.take_u64()?;
    let entry_count = cursor.take_u32()?;
    validate_checksum_algorithm(cursor.take_u8()?)?;
    validate_chunk_algorithm(cursor.take_u8()?)?;
    validate_chunk_version(cursor.take_u16()?)?;
    let target_blob_id = cursor.take::<59>()?;
    let profile_identity_version = cursor.take_u16()?;
    let profile_hash_algorithm = cursor.take_u8()?;
    let profile_digest = cursor.take::<32>()?;
    validate_reserved(cursor.take::<6>()?)?;
    Ok(DecodedHeader {
        record_length,
        entry_count,
        target_blob_id,
        profile_identity_version,
        profile_hash_algorithm,
        profile_digest,
    })
}

fn validate_magic(observed: [u8; 16]) -> Result<(), LayoutDecodeError> {
    if observed == MAGIC {
        return Ok(());
    }
    Err(LayoutDecodeError::InvalidMagic { observed })
}

const fn validate_format_version(observed: u16) -> Result<(), LayoutDecodeError> {
    if observed == FORMAT_VERSION {
        return Ok(());
    }
    Err(LayoutDecodeError::UnsupportedFormatVersion {
        expected: FORMAT_VERSION,
        observed,
    })
}

const fn validate_codec(observed: u16) -> Result<(), LayoutDecodeError> {
    if observed == LAYOUT_CODEC {
        return Ok(());
    }
    Err(LayoutDecodeError::UnsupportedCodec {
        expected: LAYOUT_CODEC,
        observed,
    })
}

const fn validate_flags(observed: u32) -> Result<(), LayoutDecodeError> {
    if observed == FLAGS {
        return Ok(());
    }
    Err(LayoutDecodeError::UnknownFlags {
        expected: FLAGS,
        observed,
    })
}

const fn validate_header_length(observed: u16) -> Result<(), LayoutDecodeError> {
    if observed == HEADER_LENGTH {
        return Ok(());
    }
    Err(LayoutDecodeError::WrongHeaderLength {
        expected: HEADER_LENGTH,
        observed,
    })
}

const fn validate_entry_length(observed: u16) -> Result<(), LayoutDecodeError> {
    if observed == ENTRY_LENGTH {
        return Ok(());
    }
    Err(LayoutDecodeError::WrongEntryLength {
        expected: ENTRY_LENGTH,
        observed,
    })
}

const fn validate_checksum_algorithm(observed: u8) -> Result<(), LayoutDecodeError> {
    if observed == CHECKSUM_ALGORITHM {
        return Ok(());
    }
    Err(LayoutDecodeError::UnsupportedChecksumAlgorithm {
        expected: CHECKSUM_ALGORITHM,
        observed,
    })
}

const fn validate_chunk_algorithm(observed: u8) -> Result<(), LayoutDecodeError> {
    if observed == CHUNK_HASH_ALGORITHM {
        return Ok(());
    }
    Err(LayoutDecodeError::UnsupportedChunkHashAlgorithm {
        expected: CHUNK_HASH_ALGORITHM,
        observed,
    })
}

const fn validate_chunk_version(observed: u16) -> Result<(), LayoutDecodeError> {
    if observed == CHUNK_IDENTITY_VERSION {
        return Ok(());
    }
    Err(LayoutDecodeError::UnsupportedChunkIdentityVersion {
        expected: CHUNK_IDENTITY_VERSION,
        observed,
    })
}

fn validate_reserved(reserved: [u8; 6]) -> Result<(), LayoutDecodeError> {
    for (offset, observed) in (138_usize..144).zip(reserved) {
        if observed != 0 {
            return Err(LayoutDecodeError::NonzeroReserved {
                offset,
                expected: 0,
                observed,
            });
        }
    }
    Ok(())
}

struct HeaderCursor<'a> {
    remaining: &'a [u8],
    input_length: usize,
}

impl<'a> HeaderCursor<'a> {
    const fn new(header: &'a [u8], input_length: usize) -> Self {
        Self {
            remaining: header,
            input_length,
        }
    }

    fn take<const WIDTH: usize>(&mut self) -> Result<[u8; WIDTH], LayoutDecodeError> {
        let Some((value, remaining)) = self.remaining.split_first_chunk::<WIDTH>() else {
            return Err(LayoutDecodeError::TruncatedHeader {
                expected: usize::from(HEADER_LENGTH),
                observed: self.input_length,
            });
        };
        self.remaining = remaining;
        Ok(*value)
    }

    fn take_u8(&mut self) -> Result<u8, LayoutDecodeError> {
        let [value] = self.take::<1>()?;
        Ok(value)
    }

    fn take_u16(&mut self) -> Result<u16, LayoutDecodeError> {
        Ok(u16::from_be_bytes(self.take::<2>()?))
    }

    fn take_u32(&mut self) -> Result<u32, LayoutDecodeError> {
        Ok(u32::from_be_bytes(self.take::<4>()?))
    }

    fn take_u64(&mut self) -> Result<u64, LayoutDecodeError> {
        Ok(u64::from_be_bytes(self.take::<8>()?))
    }
}
