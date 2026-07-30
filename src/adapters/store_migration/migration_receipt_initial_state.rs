//! This boundary module owns registered empty-state receipt admission.

use super::migration_record_bytes::read_array;
use super::{
    EmptyDispositionSetDigest, InitialGcStateDigest, InitialRetentionStateDigest,
    StoreMigrationReceiptDecodeError,
};

const INITIAL_RETENTION_DOMAIN: &[u8] = b"keep.initial-retention-state/v2\0";
const INITIAL_GC_DOMAIN: &[u8] = b"keep.initial-gc-state/v2\0";
const EMPTY_DISPOSITION_DOMAIN: &[u8] = b"keep.empty-disposition-set/v2\0";

pub(super) fn read_initial_retention_digest(
    encoded: &[u8],
) -> Result<InitialRetentionStateDigest, StoreMigrationReceiptDecodeError> {
    let expected = digest(INITIAL_RETENTION_DOMAIN);
    let observed = read_array(encoded, 120)?;
    if observed == expected {
        Ok(InitialRetentionStateDigest::from_hash(expected))
    } else {
        Err(
            StoreMigrationReceiptDecodeError::InitialRetentionStateDigestMismatch {
                expected,
                observed,
            },
        )
    }
}

pub(super) fn read_initial_gc_digest(
    encoded: &[u8],
) -> Result<InitialGcStateDigest, StoreMigrationReceiptDecodeError> {
    let expected = digest(INITIAL_GC_DOMAIN);
    let observed = read_array(encoded, 152)?;
    if observed == expected {
        Ok(InitialGcStateDigest::from_hash(expected))
    } else {
        Err(StoreMigrationReceiptDecodeError::InitialGcStateDigestMismatch { expected, observed })
    }
}

pub(super) fn read_empty_disposition_digest(
    encoded: &[u8],
) -> Result<EmptyDispositionSetDigest, StoreMigrationReceiptDecodeError> {
    let expected = digest(EMPTY_DISPOSITION_DOMAIN);
    let observed = read_array(encoded, 184)?;
    if observed == expected {
        Ok(EmptyDispositionSetDigest::from_hash(expected))
    } else {
        Err(
            StoreMigrationReceiptDecodeError::EmptyDispositionSetDigestMismatch {
                expected,
                observed,
            },
        )
    }
}

fn digest(domain: &[u8]) -> [u8; 32] {
    *blake3::hash(domain).as_bytes()
}
