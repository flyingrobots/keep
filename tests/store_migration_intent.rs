//! Canonical version-2 store-migration intent laws.

#[path = "store_migration_intent/fixture.rs"]
mod fixture;
mod support;

use std::io;

use fixture::{CATALOG_DIGEST, INTENT_DIGEST, INVENTORY_DIGEST, STORE_IDENTIFIER, fixture_bytes};
use keep::{
    AdmittedStoreMigrationIntent, CatalogGeneration, CatalogGenerationError, CatalogLength,
    CatalogLengthError, StoreFormatDefinitionDigest, StoreMigrationIntentDecodeError,
};

#[test]
fn intent_admits_every_frozen_coordinate() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fixture_bytes()?;
    let intent = AdmittedStoreMigrationIntent::decode(&bytes)?;

    assert_eq!(intent.encoded(), bytes);
    assert_eq!(intent.catalog_generation(), CatalogGeneration::new(1)?);
    assert_eq!(intent.catalog_length(), CatalogLength::new(352)?);
    assert_eq!(intent.catalog_digest().as_bytes(), &CATALOG_DIGEST);
    assert_eq!(intent.predecessor_catalog_digest(), None);
    assert_eq!(intent.inventory_digest().as_bytes(), &INVENTORY_DIGEST);
    assert_eq!(intent.root_device_identity().get(), 1);
    assert_eq!(intent.root_mount_identity().get(), 2);
    assert_eq!(intent.root_file_identity().get(), 3);
    assert_eq!(
        intent.target_definition_digest(),
        StoreFormatDefinitionDigest::VERSION_TWO
    );
    assert_eq!(intent.store_identifier().as_bytes(), &STORE_IDENTIFIER);
    assert_eq!(intent.digest().as_bytes(), &INTENT_DIGEST);
    Ok(())
}

#[test]
fn intent_framing_has_exact_first_refusals() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fixture_bytes()?;
    let mut truncated = bytes.clone();
    assert!(truncated.pop().is_some());
    assert_eq!(
        AdmittedStoreMigrationIntent::decode(&truncated),
        Err(StoreMigrationIntentDecodeError::WrongLength {
            expected: 256,
            observed: 255,
        })
    );
    let mut extended = bytes.clone();
    extended.push(0);
    assert_eq!(
        AdmittedStoreMigrationIntent::decode(&extended),
        Err(StoreMigrationIntentDecodeError::WrongLength {
            expected: 256,
            observed: 257,
        })
    );
    assert_fixed_refusal(
        0,
        StoreMigrationIntentDecodeError::InvalidMagic {
            observed: mutated_array(&bytes, 0, 0)?,
        },
    )?;
    assert_fixed_refusal(
        17,
        StoreMigrationIntentDecodeError::UnsupportedVersion {
            expected: 2,
            observed: 3,
        },
    )?;
    assert_fixed_refusal(
        19,
        StoreMigrationIntentDecodeError::InvalidRecordLength {
            expected: 256,
            observed: 257,
        },
    )?;
    assert_fixed_refusal(
        23,
        StoreMigrationIntentDecodeError::UnsupportedFlags { observed: 1 },
    )?;
    Ok(())
}

#[test]
fn checksum_and_semantic_laws_have_exact_precedence() -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = fixture_bytes()?;
    flip_byte(&mut bytes, 31)?;
    assert!(matches!(
        AdmittedStoreMigrationIntent::decode(&bytes),
        Err(StoreMigrationIntentDecodeError::ChecksumMismatch { .. })
    ));
    refresh_checksum(&mut bytes)?;
    assert_eq!(
        AdmittedStoreMigrationIntent::decode(&bytes),
        Err(StoreMigrationIntentDecodeError::InvalidCatalogGeneration {
            observed: 0,
            source: CatalogGenerationError::Zero,
        })
    );

    assert_semantic_refusal(
        39,
        StoreMigrationIntentDecodeError::InvalidCatalogLength {
            observed: 353,
            source: CatalogLengthError::NotCongruent { observed: 353 },
        },
    )?;
    assert_semantic_refusal(
        103,
        StoreMigrationIntentDecodeError::NonZeroInitialPredecessor {
            observed: mutated_array(&fixture_bytes()?, 72, 31)?,
        },
    )?;

    let mut successor = fixture_bytes()?;
    let generation = successor
        .get_mut(31)
        .ok_or_else(|| io::Error::other("intent lacks generation field"))?;
    *generation = 2;
    refresh_checksum(&mut successor)?;
    assert_eq!(
        AdmittedStoreMigrationIntent::decode(&successor),
        Err(StoreMigrationIntentDecodeError::MissingSuccessorPredecessor { generation: 2 })
    );

    assert_semantic_refusal(
        160,
        StoreMigrationIntentDecodeError::DefinitionDigestMismatch {
            expected: *StoreFormatDefinitionDigest::VERSION_TWO.as_bytes(),
            observed: mutated_array(&fixture_bytes()?, 160, 0)?,
        },
    )?;
    assert_semantic_refusal(
        223,
        StoreMigrationIntentDecodeError::StoreIdentifierMismatch {
            expected: STORE_IDENTIFIER,
            observed: mutated_array(&fixture_bytes()?, 192, 31)?,
        },
    )?;
    Ok(())
}

fn assert_fixed_refusal(
    offset: usize,
    expected: StoreMigrationIntentDecodeError,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = fixture_bytes()?;
    flip_byte(&mut bytes, offset)?;
    assert_eq!(AdmittedStoreMigrationIntent::decode(&bytes), Err(expected));
    Ok(())
}

fn assert_semantic_refusal(
    offset: usize,
    expected: StoreMigrationIntentDecodeError,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = fixture_bytes()?;
    flip_byte(&mut bytes, offset)?;
    refresh_checksum(&mut bytes)?;
    assert_eq!(AdmittedStoreMigrationIntent::decode(&bytes), Err(expected));
    Ok(())
}

fn mutated_array<const WIDTH: usize>(
    bytes: &[u8],
    offset: usize,
    relative: usize,
) -> Result<[u8; WIDTH], io::Error> {
    let end = offset
        .checked_add(WIDTH)
        .ok_or_else(|| io::Error::other("intent field offset overflow"))?;
    let field = bytes
        .get(offset..end)
        .ok_or_else(|| io::Error::other("intent lacks fixed field"))?;
    let mut observed = <[u8; WIDTH]>::try_from(field)
        .map_err(|_| io::Error::other("intent field width mismatch"))?;
    flip_byte(&mut observed, relative)?;
    Ok(observed)
}

fn flip_byte(bytes: &mut [u8], offset: usize) -> Result<(), io::Error> {
    let byte = bytes
        .get_mut(offset)
        .ok_or_else(|| io::Error::other("intent mutation offset is out of bounds"))?;
    *byte ^= 1;
    Ok(())
}

fn refresh_checksum(bytes: &mut [u8]) -> Result<(), io::Error> {
    let (preimage, checksum) = bytes
        .split_at_mut_checked(224)
        .ok_or_else(|| io::Error::other("intent lacks checksum boundary"))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"keep.store-migration-intent-checksum/v2\0");
    hasher.update(preimage);
    checksum.copy_from_slice(hasher.finalize().as_bytes());
    Ok(())
}
