//! This boundary module owns store-migration receipt decoding order.

use super::admitted_migration_receipt::StoreMigrationReceiptFields;
use super::migration_receipt_initial_state::{
    read_empty_disposition_digest, read_initial_gc_digest, read_initial_retention_digest,
};
use super::migration_record_bytes::{
    read_array, read_u16, read_u32, read_u64, require_length, wrong_length,
};
use super::{
    AdmittedStoreFormatMarker, AdmittedStoreMigrationIntent, AdmittedStoreMigrationReceipt,
    MigrationSynchronizationMask, StoreMigrationReceiptDecodeError,
    migration_receipt_format as format,
};

pub(super) fn decode<'encoded>(
    encoded: &'encoded [u8],
    intent: &AdmittedStoreMigrationIntent<'_>,
    marker: &AdmittedStoreFormatMarker<'_>,
) -> Result<AdmittedStoreMigrationReceipt<'encoded>, StoreMigrationReceiptDecodeError> {
    require_length(encoded)?;
    validate_fixed_fields(encoded)?;
    verify_checksum(encoded)?;
    let intent_digest = bind_intent_digest(encoded, intent)?;
    let store_identifier = bind_store_identifier(encoded, intent)?;
    let format_marker_digest = bind_marker_digest(encoded, marker)?;
    let initial_retention_state_digest = read_initial_retention_digest(encoded)?;
    let initial_gc_state_digest = read_initial_gc_digest(encoded)?;
    let empty_disposition_set_digest = read_empty_disposition_digest(encoded)?;
    let synchronization_mask = read_synchronization_mask(encoded)?;
    Ok(AdmittedStoreMigrationReceipt::admitted(
        encoded,
        StoreMigrationReceiptFields {
            intent_digest,
            store_identifier,
            format_marker_digest,
            initial_retention_state_digest,
            initial_gc_state_digest,
            empty_disposition_set_digest,
            synchronization_mask,
        },
    ))
}

fn validate_fixed_fields(encoded: &[u8]) -> Result<(), StoreMigrationReceiptDecodeError> {
    let magic = read_array(encoded, 0)?;
    if magic != format::MAGIC {
        return Err(StoreMigrationReceiptDecodeError::InvalidMagic { observed: magic });
    }
    let version = read_u16(encoded, 16)?;
    if version != format::VERSION {
        return Err(StoreMigrationReceiptDecodeError::UnsupportedVersion {
            expected: format::VERSION,
            observed: version,
        });
    }
    let record_length = read_u16(encoded, 18)?;
    if record_length != format::RECORD_LENGTH {
        return Err(StoreMigrationReceiptDecodeError::InvalidRecordLength {
            expected: format::RECORD_LENGTH,
            observed: record_length,
        });
    }
    let flags = read_u32(encoded, 20)?;
    if flags != 0 {
        return Err(StoreMigrationReceiptDecodeError::UnsupportedFlags { observed: flags });
    }
    Ok(())
}

fn verify_checksum(encoded: &[u8]) -> Result<(), StoreMigrationReceiptDecodeError> {
    let preimage = encoded
        .get(..format::CHECKSUM_OFFSET)
        .ok_or_else(|| wrong_length(encoded))?;
    let observed = read_array(encoded, format::CHECKSUM_OFFSET)?;
    let expected = format::checksum(preimage);
    if observed == expected {
        Ok(())
    } else {
        Err(StoreMigrationReceiptDecodeError::ChecksumMismatch { expected, observed })
    }
}

fn bind_intent_digest(
    encoded: &[u8],
    intent: &AdmittedStoreMigrationIntent<'_>,
) -> Result<super::StoreMigrationIntentDigest, StoreMigrationReceiptDecodeError> {
    let expected = *intent.digest().as_bytes();
    let observed = read_array(encoded, 24)?;
    if observed == expected {
        Ok(intent.digest())
    } else {
        Err(StoreMigrationReceiptDecodeError::IntentDigestMismatch { expected, observed })
    }
}

fn bind_store_identifier(
    encoded: &[u8],
    intent: &AdmittedStoreMigrationIntent<'_>,
) -> Result<super::StoreIdentifier, StoreMigrationReceiptDecodeError> {
    let expected = *intent.store_identifier().as_bytes();
    let observed = read_array(encoded, 56)?;
    if observed == expected {
        Ok(intent.store_identifier())
    } else {
        Err(StoreMigrationReceiptDecodeError::StoreIdentifierMismatch { expected, observed })
    }
}

fn bind_marker_digest(
    encoded: &[u8],
    marker: &AdmittedStoreFormatMarker<'_>,
) -> Result<super::StoreFormatMarkerDigest, StoreMigrationReceiptDecodeError> {
    let expected = *marker.digest().as_bytes();
    let observed = read_array(encoded, 88)?;
    if observed == expected {
        Ok(marker.digest())
    } else {
        Err(StoreMigrationReceiptDecodeError::FormatMarkerDigestMismatch { expected, observed })
    }
}

fn read_synchronization_mask(
    encoded: &[u8],
) -> Result<MigrationSynchronizationMask, StoreMigrationReceiptDecodeError> {
    let observed = read_u64(encoded, 216)?;
    let supported = MigrationSynchronizationMask::COMPLETE_BITS;
    if observed & !supported != 0 {
        return Err(
            StoreMigrationReceiptDecodeError::UnsupportedSynchronizationBits {
                supported,
                observed,
            },
        );
    }
    if observed != supported {
        return Err(
            StoreMigrationReceiptDecodeError::IncompleteSynchronizationMask {
                required: supported,
                observed,
            },
        );
    }
    Ok(MigrationSynchronizationMask::complete())
}
