//! This module owns CDC profile bytes, the Gear recipe, and typed identities.

use std::fmt::Write;

use md5::{Digest, Md5};

use super::{
    GearTable, LONG_MASK, MAXIMUM, MINIMUM, NORMALIZATION, SEED, SHORT_MASK, STATE_WIDTH, TARGET,
};
use crate::protocol_conformance::canonical::exact_hex;
use crate::protocol_conformance::corpus::{Corpus, TablePolicy};
use crate::protocol_conformance::{ConformanceError, external_digest};

const BOUNDARY_ALGORITHM: u16 = 1;
const FORMAT_VERSION: u16 = 1;
const GEAR_MAGIC: [u8; 16] = *b"KEEP:GEAR:TABLE\0";
const HASH_ALGORITHM: [u8; 1] = [1];
const PROFILE_COLUMNS: [&str; 5] = [
    "profile",
    "gear_table",
    "gear_checksum_hex",
    "profile_record",
    "storage_profile_id",
];
const PROFILE_LENGTH: usize = 96;
const PROFILE_MAGIC: [u8; 16] = *b"KEEP:CDC:PROFILE";
const PROFILE_NAME: &str = "fastcdc-64k-v1";
const PROFILE_POLICY: TablePolicy = TablePolicy::new(
    "keep.cdc-profile-fixture/v1",
    &PROFILE_COLUMNS,
    1_048_576,
    1,
);
const TABLE_BYTES: usize = 2_048;

pub(super) fn check(corpus: &Corpus) -> Result<GearTable, ConformanceError> {
    let rows = corpus.rows("profile.tsv", PROFILE_POLICY)?;
    let row = rows
        .first()
        .ok_or_else(|| ConformanceError::violation("profile.tsv has no canonical profile"))?;
    if rows.len() != 1 || row.field("profile")? != PROFILE_NAME {
        return Err(ConformanceError::violation(
            "profile.tsv must contain exactly the canonical profile",
        ));
    }
    let table = corpus
        .source_file(row.field("gear_table")?)?
        .bounded_bytes(TABLE_BYTES, "gear table")?;
    let generated = generated_gear_table();
    if table.len() != TABLE_BYTES || table != generated {
        return Err(ConformanceError::violation(
            "authoritative Gear table differs from its reproducible recipe",
        ));
    }
    let checksum = typed_gear_checksum(&table)?;
    let expected_checksum = exact_hex(row.field("gear_checksum_hex")?, "Gear checksum", 32)?;
    if checksum.as_slice() != expected_checksum {
        return Err(ConformanceError::violation(
            "typed Gear checksum differs from profile.tsv",
        ));
    }
    check_profile_record(
        corpus,
        row.field("profile_record")?,
        &checksum,
        row.field("storage_profile_id")?,
    )?;
    decode_gear_table(&table)
}

/// Reproduces the versioned, deterministic Gear-table recipe.
///
/// `MD5` is intentionally part of this non-cryptographic public recipe. The
/// typed table checksum is verified separately through [`external_digest`].
fn generated_gear_table() -> Vec<u8> {
    let mut table = Vec::with_capacity(TABLE_BYTES);
    for value in 0_u8..=u8::MAX {
        let digest: [u8; 16] = Md5::digest([value; 64]).into();
        let [
            first,
            second,
            third,
            fourth,
            fifth,
            sixth,
            seventh,
            eighth,
            ..,
        ] = digest;
        table.extend_from_slice(&[first, second, third, fourth, fifth, sixth, seventh, eighth]);
    }
    table
}

fn typed_gear_checksum(table: &[u8]) -> Result<[u8; 32], ConformanceError> {
    let version = FORMAT_VERSION.to_be_bytes();
    let length = u64::try_from(table.len())
        .map_err(|source| ConformanceError::violation(format!("Gear length overflow: {source}")))?
        .to_be_bytes();
    external_digest::digest(&[
        GEAR_MAGIC.as_slice(),
        version.as_slice(),
        HASH_ALGORITHM.as_slice(),
        table,
        length.as_slice(),
    ])
}

fn check_profile_record(
    corpus: &Corpus,
    path: &str,
    checksum: &[u8; 32],
    expected_id: &str,
) -> Result<(), ConformanceError> {
    let record = corpus
        .source_file(path)?
        .bounded_bytes(PROFILE_LENGTH, "profile record")?;
    let expected_record = canonical_profile_record(checksum)?;
    if record != expected_record {
        return Err(ConformanceError::violation(
            "profile record differs from its canonical field encoding",
        ));
    }
    let digest = external_digest::digest(&[record.as_slice()])?;
    let observed_id = profile_id(&digest)?;
    if observed_id != expected_id {
        return Err(ConformanceError::violation(
            "StorageProfileId differs from the profile record digest",
        ));
    }
    Ok(())
}

fn canonical_profile_record(checksum: &[u8; 32]) -> Result<Vec<u8>, ConformanceError> {
    let mut record = Vec::with_capacity(PROFILE_LENGTH);
    record.extend_from_slice(&PROFILE_MAGIC);
    extend_u16(&mut record, FORMAT_VERSION);
    extend_u16(
        &mut record,
        u16::try_from(PROFILE_LENGTH).map_err(|source| {
            ConformanceError::violation(format!("profile width overflow: {source}"))
        })?,
    );
    extend_u16(&mut record, BOUNDARY_ALGORITHM);
    extend_u16(&mut record, 0);
    record.extend_from_slice(checksum);
    record.extend_from_slice(&SEED.to_be_bytes());
    extend_u32(&mut record, MINIMUM)?;
    extend_u32(&mut record, TARGET)?;
    extend_u32(&mut record, MAXIMUM)?;
    record.extend_from_slice(&[NORMALIZATION, STATE_WIDTH]);
    extend_u16(&mut record, 0);
    record.extend_from_slice(&SHORT_MASK.to_be_bytes());
    record.extend_from_slice(&LONG_MASK.to_be_bytes());
    if record.len() != PROFILE_LENGTH {
        return Err(ConformanceError::violation(format!(
            "constructed profile has {} bytes, expected {PROFILE_LENGTH}",
            record.len()
        )));
    }
    Ok(record)
}

fn extend_u16(record: &mut Vec<u8>, value: u16) {
    record.extend_from_slice(&value.to_be_bytes());
}

fn extend_u32(record: &mut Vec<u8>, value: usize) -> Result<(), ConformanceError> {
    let value = u32::try_from(value).map_err(|source| {
        ConformanceError::violation(format!("profile field overflow: {source}"))
    })?;
    record.extend_from_slice(&value.to_be_bytes());
    Ok(())
}

fn decode_gear_table(table: &[u8]) -> Result<GearTable, ConformanceError> {
    let mut gear = Vec::with_capacity(256);
    for entry in table.chunks_exact(8) {
        let bytes = <[u8; 8]>::try_from(entry)
            .map_err(|_| ConformanceError::violation("Gear entry has the wrong width"))?;
        gear.push(u64::from_be_bytes(bytes));
    }
    gear.try_into()
        .map_err(|_| ConformanceError::violation("Gear table has the wrong entry count"))
}

fn profile_id(digest: &[u8; 32]) -> Result<String, ConformanceError> {
    let mut identity = String::from("keep:storage-profile:v1:blake3-256:");
    for byte in digest {
        write!(&mut identity, "{byte:02x}")
            .map_err(|_| ConformanceError::violation("profile identity formatting failed"))?;
    }
    Ok(identity)
}

#[cfg(test)]
mod tests {
    use super::{TABLE_BYTES, generated_gear_table};

    #[test]
    fn public_md5_recipe_freezes_first_and_last_gear_entries() {
        let table = generated_gear_table();
        assert_eq!(
            table.get(..8),
            Some([0x3b, 0x5d, 0x3c, 0x7d, 0x20, 0x7e, 0x37, 0xdc].as_slice())
        );
        assert_eq!(
            table.get(TABLE_BYTES - 8..),
            Some([0xaa, 0xbd, 0x2b, 0x2a, 0x45, 0x15, 0x04, 0xe1].as_slice())
        );
    }
}
