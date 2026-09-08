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
    let admitted = initial_retention_digest();
    let expected = *admitted.as_bytes();
    let observed = read_array(encoded, 120)?;
    if observed == expected {
        Ok(admitted)
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
    let admitted = initial_gc_digest();
    let expected = *admitted.as_bytes();
    let observed = read_array(encoded, 152)?;
    if observed == expected {
        Ok(admitted)
    } else {
        Err(StoreMigrationReceiptDecodeError::InitialGcStateDigestMismatch { expected, observed })
    }
}

pub(super) fn read_empty_disposition_digest(
    encoded: &[u8],
) -> Result<EmptyDispositionSetDigest, StoreMigrationReceiptDecodeError> {
    let admitted = empty_disposition_digest();
    let expected = *admitted.as_bytes();
    let observed = read_array(encoded, 184)?;
    if observed == expected {
        Ok(admitted)
    } else {
        Err(
            StoreMigrationReceiptDecodeError::EmptyDispositionSetDigestMismatch {
                expected,
                observed,
            },
        )
    }
}

pub(super) fn initial_retention_digest() -> InitialRetentionStateDigest {
    InitialRetentionStateDigest::from_hash(digest(INITIAL_RETENTION_DOMAIN))
}

pub(super) fn initial_gc_digest() -> InitialGcStateDigest {
    InitialGcStateDigest::from_hash(digest(INITIAL_GC_DOMAIN))
}

pub(super) fn empty_disposition_digest() -> EmptyDispositionSetDigest {
    EmptyDispositionSetDigest::from_hash(digest(EMPTY_DISPOSITION_DOMAIN))
}

fn digest(domain: &[u8]) -> [u8; 32] {
    *blake3::hash(domain).as_bytes()
}
