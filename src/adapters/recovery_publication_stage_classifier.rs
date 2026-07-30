//! This module owns whole-byte classification of staged publication metadata.

use super::{
    ChecksummedCatalog, ChecksummedPublicationHead, RecoveryCatalogStage,
    RecoveryCatalogStageError, RecoveryNextHeadStage, RecoveryNextHeadStageError, RecoveryStage,
    RecoveryStageMetadata, catalog_decoder, catalog_header_decoder, publication_head_decoder,
    recovery_publication_fixed_framing,
};

/// Classifies one complete caller-supplied catalog-stage byte sequence.
///
/// The input must contain all currently observed `current.cat` bytes. The call
/// performs no I/O, allocation, or content copy.
///
/// # Errors
///
/// Returns [`RecoveryCatalogStageError`] for oversized input, proven
/// fixed-framing corruption, or complete-looking canonical catalog refusal.
/// Known incomplete boundaries are returned as truncation states only while
/// every available fixed-framing byte remains canonical.
pub fn classify_recovery_catalog_stage(
    encoded: &[u8],
) -> Result<RecoveryCatalogStage<'_>, RecoveryCatalogStageError> {
    let observed = catalog_metadata_length(encoded)?;
    if encoded.len() < catalog_header_decoder::HEADER_LENGTH_BYTES {
        recovery_publication_fixed_framing::catalog_header(encoded)
            .map_err(|source| RecoveryCatalogStageError::Header { source })?;
        return Ok(RecoveryCatalogStage::HeaderTruncated {
            required: catalog_header_decoder::HEADER_LENGTH_BYTES,
            observed: encoded.len(),
        });
    }
    let fields = catalog_header_decoder::decode_header(encoded)
        .map_err(|source| RecoveryCatalogStageError::Header { source })?;
    let metadata = catalog_decoder::validate_header(&fields)
        .map_err(|source| RecoveryCatalogStageError::Header { source })?;
    if observed < metadata.length().get() {
        return Ok(RecoveryCatalogStage::BodyTruncated {
            expected: metadata.length().get(),
            observed: encoded.len(),
        });
    }
    ChecksummedCatalog::decode(encoded)
        .map(RecoveryCatalogStage::Complete)
        .map_err(|source| RecoveryCatalogStageError::Complete { source })
}

/// Classifies one complete caller-supplied candidate-head byte sequence.
///
/// The input must contain all currently observed `head.next` bytes. The call
/// performs no I/O, allocation, or content copy.
///
/// # Errors
///
/// Returns [`RecoveryNextHeadStageError`] for oversized input or a
/// complete-looking canonical publication-head refusal. Short input is
/// returned as a truncation state only while every available fixed-framing
/// byte remains canonical.
pub fn classify_recovery_next_head_stage(
    encoded: &[u8],
) -> Result<RecoveryNextHeadStage<'_>, RecoveryNextHeadStageError> {
    admit_next_head_metadata(encoded)?;
    if encoded.len() < publication_head_decoder::ENCODED_LENGTH {
        recovery_publication_fixed_framing::next_head(encoded)
            .map_err(|source| RecoveryNextHeadStageError::Complete { source })?;
        return Ok(RecoveryNextHeadStage::Truncated {
            required: publication_head_decoder::ENCODED_LENGTH,
            observed: encoded.len(),
        });
    }
    ChecksummedPublicationHead::decode(encoded)
        .map(RecoveryNextHeadStage::Complete)
        .map_err(|source| RecoveryNextHeadStageError::Complete { source })
}

fn catalog_metadata_length(encoded: &[u8]) -> Result<u64, RecoveryCatalogStageError> {
    let observed =
        u64::try_from(encoded.len()).map_err(|_| RecoveryCatalogStageError::AddressSpace {
            observed: encoded.len(),
        })?;
    RecoveryStageMetadata::new(RecoveryStage::Catalog, observed)
        .map(RecoveryStageMetadata::length)
        .map(super::RecoveryStageLength::get)
        .map_err(|source| RecoveryCatalogStageError::Metadata { source })
}

fn admit_next_head_metadata(encoded: &[u8]) -> Result<(), RecoveryNextHeadStageError> {
    let observed =
        u64::try_from(encoded.len()).map_err(|_| RecoveryNextHeadStageError::AddressSpace {
            observed: encoded.len(),
        })?;
    RecoveryStageMetadata::new(RecoveryStage::NextHead, observed)
        .map(|_metadata| ())
        .map_err(|source| RecoveryNextHeadStageError::Metadata { source })
}
