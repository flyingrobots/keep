//! Field-by-field layout oracle independent of the production layout codec.

use std::error::Error;

use keep::{BlobId, LayoutId};

use crate::support::{decode_hex, invalid_corpus};

const ENTRIES: &str = include_str!("../../conformance/layout/v1/entries.tsv");
const RECORD_MAGIC: [u8; 16] = *b"KEEP:LAYOUT:PLAN";
const LAYOUT_ID_MAGIC: [u8; 16] = *b"KEEP:LAYOUT:ID\0\0";

/// Reconstructs and verifies every canonical field, checksum, and identity.
///
/// # Errors
///
/// Returns a corpus or public identity error when a fixture cannot be
/// independently reconstructed.
///
/// # Panics
///
/// Panics when a decoded fixture field differs from its independently
/// reconstructed expected value.
pub fn verify_record(case: &str, row: &str, record: &[u8]) -> Result<(), Box<dyn Error>> {
    let record_length = field(row, 8)?.parse::<u64>()?;
    let entry_count = field(row, 7)?.parse::<u32>()?;
    assert_eq!(u64::try_from(record.len())?, record_length, "{case}");
    assert_eq!(array_at::<16>(record, 0)?, RECORD_MAGIC, "{case}");
    assert_eq!(u16_at(record, 16)?, 1, "{case}");
    assert_eq!(u16_at(record, 18)?, 1, "{case}");
    assert_eq!(u32_at(record, 20)?, 0, "{case}");
    assert_eq!(u16_at(record, 24)?, 144, "{case}");
    assert_eq!(u16_at(record, 26)?, 44, "{case}");
    assert_eq!(u64_at(record, 28)?, record_length, "{case}");
    assert_eq!(u32_at(record, 36)?, entry_count, "{case}");
    assert_eq!(u8_at(record, 40)?, 1, "{case}");
    assert_eq!(u8_at(record, 41)?, 1, "{case}");
    assert_eq!(u16_at(record, 42)?, 1, "{case}");
    verify_target(case, row, record)?;
    verify_profile(case, row, record)?;
    assert_eq!(array_at::<6>(record, 138)?, [0_u8; 6], "{case}");
    verify_entries(case, entry_count, record)?;
    verify_checksum(case, row, record)?;
    verify_layout_id(case, row, record, record_length)?;
    Ok(())
}

fn verify_target(case: &str, row: &str, record: &[u8]) -> Result<(), Box<dyn Error>> {
    let expected = field(row, 5)?.parse::<BlobId>()?.encode_binary();
    assert_eq!(array_at::<59>(record, 44)?, expected, "{case}");
    Ok(())
}

fn verify_profile(case: &str, row: &str, record: &[u8]) -> Result<(), Box<dyn Error>> {
    assert_eq!(u16_at(record, 103)?, 1, "{case}");
    assert_eq!(u8_at(record, 105)?, 1, "{case}");
    let digest_text = field(row, 6)?
        .rsplit(':')
        .next()
        .ok_or_else(|| invalid_corpus("storage-profile digest is missing"))?;
    let expected = decode_hex(digest_text)?;
    assert_eq!(slice_at(record, 106, 32)?, expected, "{case}");
    Ok(())
}

fn verify_entries(case: &str, expected_count: u32, record: &[u8]) -> Result<(), Box<dyn Error>> {
    let rows = ENTRIES
        .lines()
        .skip(2)
        .filter(|row| row.split('\t').next() == Some(case))
        .collect::<Vec<_>>();
    assert_eq!(u32::try_from(rows.len())?, expected_count, "{case}");
    for (position, row) in rows.into_iter().enumerate() {
        let index = field(row, 1)?.parse::<usize>()?;
        assert_eq!(index, position, "{case}");
        let relative = position
            .checked_mul(44)
            .ok_or_else(|| invalid_corpus("entry offset multiplication overflow"))?;
        let offset = 144_usize
            .checked_add(relative)
            .ok_or_else(|| invalid_corpus("entry offset addition overflow"))?;
        assert_eq!(u64_at(record, offset)?, field(row, 2)?.parse()?, "{case}");
        let length_offset = offset
            .checked_add(8)
            .ok_or_else(|| invalid_corpus("entry length offset overflow"))?;
        assert_eq!(
            u32_at(record, length_offset)?,
            field(row, 3)?.parse()?,
            "{case}"
        );
        let digest_offset = offset
            .checked_add(12)
            .ok_or_else(|| invalid_corpus("entry digest offset overflow"))?;
        assert_eq!(
            slice_at(record, digest_offset, 32)?,
            decode_hex(field(row, 4)?)?,
            "{case}"
        );
    }
    Ok(())
}

fn verify_checksum(case: &str, row: &str, record: &[u8]) -> Result<(), Box<dyn Error>> {
    let checksum_offset = record
        .len()
        .checked_sub(32)
        .ok_or_else(|| invalid_corpus("record is shorter than its checksum"))?;
    let covered = record
        .get(..checksum_offset)
        .ok_or_else(|| invalid_corpus("checksum coverage is out of bounds"))?;
    let mut state = blake3::Hasher::new();
    state.update(b"KEEP:LAYOUT:SUM\0");
    state.update(&1_u16.to_be_bytes());
    state.update(&[1_u8]);
    state.update(covered);
    state.update(&u64::try_from(covered.len())?.to_be_bytes());
    let expected = *state.finalize().as_bytes();
    assert_eq!(array_at::<32>(record, checksum_offset)?, expected, "{case}");
    assert_eq!(expected.as_slice(), decode_hex(field(row, 9)?)?, "{case}");
    Ok(())
}

fn verify_layout_id(
    case: &str,
    row: &str,
    record: &[u8],
    record_length: u64,
) -> Result<(), Box<dyn Error>> {
    let mut state = blake3::Hasher::new();
    state.update(b"KEEP:LAYOUT:ID\0\0");
    state.update(&1_u16.to_be_bytes());
    state.update(&1_u16.to_be_bytes());
    state.update(record);
    state.update(&record_length.to_be_bytes());
    let mut coordinate = Vec::with_capacity(LayoutId::BINARY_LENGTH);
    coordinate.extend_from_slice(&LAYOUT_ID_MAGIC);
    coordinate.extend_from_slice(&1_u16.to_be_bytes());
    coordinate.extend_from_slice(&1_u16.to_be_bytes());
    coordinate.extend_from_slice(&record_length.to_be_bytes());
    coordinate.extend_from_slice(state.finalize().as_bytes());
    assert_eq!(coordinate, decode_hex(field(row, 11)?)?, "{case}");
    assert_eq!(
        LayoutId::parse_binary(&coordinate)?.to_string(),
        field(row, 10)?,
        "{case}"
    );
    Ok(())
}

fn array_at<const WIDTH: usize>(
    bytes: &[u8],
    offset: usize,
) -> Result<[u8; WIDTH], Box<dyn Error>> {
    let slice = slice_at(bytes, offset, WIDTH)?;
    slice
        .try_into()
        .map_err(|_source| Box::<dyn Error>::from(invalid_corpus("fixed field width moved")))
}

fn slice_at(bytes: &[u8], offset: usize, width: usize) -> Result<&[u8], Box<dyn Error>> {
    let end = offset
        .checked_add(width)
        .ok_or_else(|| invalid_corpus("field offset overflow"))?;
    bytes
        .get(offset..end)
        .ok_or_else(|| Box::<dyn Error>::from(invalid_corpus("field is out of bounds")))
}

fn u8_at(bytes: &[u8], offset: usize) -> Result<u8, Box<dyn Error>> {
    Ok(u8::from_be_bytes(array_at(bytes, offset)?))
}

fn u16_at(bytes: &[u8], offset: usize) -> Result<u16, Box<dyn Error>> {
    Ok(u16::from_be_bytes(array_at(bytes, offset)?))
}

fn u32_at(bytes: &[u8], offset: usize) -> Result<u32, Box<dyn Error>> {
    Ok(u32::from_be_bytes(array_at(bytes, offset)?))
}

fn u64_at(bytes: &[u8], offset: usize) -> Result<u64, Box<dyn Error>> {
    Ok(u64::from_be_bytes(array_at(bytes, offset)?))
}

fn field(row: &str, index: usize) -> Result<&str, Box<dyn Error>> {
    row.split('\t')
        .nth(index)
        .ok_or_else(|| Box::<dyn Error>::from(invalid_corpus("TSV row is missing a field")))
}
