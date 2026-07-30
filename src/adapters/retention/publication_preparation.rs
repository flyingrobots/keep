//! This boundary module owns storage-independent retention publication preparation.

use super::{
    AdmittedRetentionManifest, CanonicalRetentionHead, CanonicalRetentionManifest,
    PreparedRetentionPublication, RetentionPublicationPreparation,
    RetentionPublicationPreparationError, RetentionTransitionDisposition,
    RetentionTransitionPreflight, successor_manifest,
};
use crate::{RetentionHead, RetentionManifestLength};

/// Binds preflight evidence to one current manifest and canonical successors.
///
/// A publish result owns the complete manifest and head bytes required by the
/// storage protocol. An exact retry returns no new global artifacts. This
/// function performs no I/O.
///
/// # Errors
///
/// Returns [`RetentionPublicationPreparationError`] when the current manifest
/// disagrees with the preflight candidate, checked generation arithmetic or
/// bounded allocation fails, or canonical successor construction refuses.
pub fn prepare_retention_publication<'encoded>(
    preflight: RetentionTransitionPreflight<'encoded>,
    current_manifest: Option<&AdmittedRetentionManifest<'_>>,
) -> Result<RetentionPublicationPreparation<'encoded>, RetentionPublicationPreparationError> {
    let (disposition, expected, observed, candidate, closure) = preflight.into_parts();
    match disposition {
        RetentionTransitionDisposition::AlreadyCommitted => {
            let current =
                successor_manifest::require_current_selection(&candidate, current_manifest)?;
            Ok(RetentionPublicationPreparation::already_committed(
                expected, observed, candidate, closure, current,
            ))
        }
        RetentionTransitionDisposition::Publish => {
            let semantic_manifest = successor_manifest::build(&candidate, current_manifest)?;
            let liveness_generation = semantic_manifest.generation();
            let predecessor = semantic_manifest.predecessor();
            let manifest = CanonicalRetentionManifest::from_manifest(&semantic_manifest).map_err(
                |source| RetentionPublicationPreparationError::ManifestEncoding { source },
            )?;
            let manifest_length = manifest_length(&manifest)?;
            let semantic_head = RetentionHead::new(
                liveness_generation,
                manifest_length,
                manifest.digest(),
                predecessor,
            )
            .map_err(|source| RetentionPublicationPreparationError::Head { source })?;
            let head = CanonicalRetentionHead::from_head(&semantic_head);
            let publication =
                PreparedRetentionPublication::new(manifest, head, liveness_generation);
            Ok(RetentionPublicationPreparation::publish(
                expected,
                observed,
                candidate,
                closure,
                publication,
            ))
        }
    }
}

fn manifest_length(
    manifest: &CanonicalRetentionManifest,
) -> Result<RetentionManifestLength, RetentionPublicationPreparationError> {
    let observed = manifest.encoded().len();
    let value = u64::try_from(observed)
        .map_err(|_| RetentionPublicationPreparationError::ManifestLengthOverflow { observed })?;
    RetentionManifestLength::new(value)
        .map_err(|source| RetentionPublicationPreparationError::ManifestLength { source })
}
