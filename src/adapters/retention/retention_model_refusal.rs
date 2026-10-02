//! This module owns the model's exact rejected-operation diagnostic oracles.

use std::error::Error;

use super::super::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, RetentionCurrentStateRefusal,
    RetentionPublicationError, RetentionPublicationPreparationError,
};
use crate::{
    LivenessGeneration, RetentionManifestDigest, RetentionNamespaceDigest, RetentionRootDigest,
    RootGeneration,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Refusal {
    RepeatedInitial {
        namespace: RetentionNamespaceDigest,
        generation: RootGeneration,
        digest: RetentionRootDigest,
    },
    Superseded {
        generation: LivenessGeneration,
        digest: RetentionManifestDigest,
    },
}

impl Refusal {
    pub(super) fn repeated_initial(
        candidate: &[u8],
        manifest: Option<&[u8]>,
    ) -> Result<Self, Box<dyn Error>> {
        let namespace = AdmittedRetentionRoot::decode(candidate)?
            .root()
            .namespace()
            .digest();
        let manifest = AdmittedRetentionManifest::decode(manifest.ok_or("model manifest absent")?)?;
        let entry = manifest
            .manifest()
            .entries()
            .iter()
            .find(|entry| entry.namespace() == namespace)
            .ok_or("model namespace absent from pre-operation manifest")?;
        Ok(Self::RepeatedInitial {
            namespace,
            generation: entry.root_generation(),
            digest: entry.root_digest(),
        })
    }

    pub(super) fn superseded(manifest: Option<&[u8]>) -> Result<Self, Box<dyn Error>> {
        let manifest = AdmittedRetentionManifest::decode(manifest.ok_or("model manifest absent")?)?;
        Ok(Self::Superseded {
            generation: manifest.manifest().generation(),
            digest: manifest.digest(),
        })
    }

    pub(super) fn verify(self, error: &(dyn Error + 'static)) -> Result<(), Box<dyn Error>> {
        match self {
            Self::RepeatedInitial {
                namespace,
                generation,
                digest,
            } => {
                assert!(
                    matches!(
                        error.downcast_ref::<RetentionPublicationPreparationError>(),
                        Some(RetentionPublicationPreparationError::ManifestSuccessorMismatch {
                            namespace: observed_namespace,
                            current_generation,
                            current_digest,
                            candidate_generation: RootGeneration::INITIAL,
                            candidate_predecessor: None,
                        }) if (*observed_namespace, *current_generation, *current_digest)
                            == (namespace, generation, digest)
                    ),
                    "repeated initial must report its exact manifest successor mismatch: {error:?}"
                );
            }
            Self::Superseded { generation, digest } => {
                let Some(RetentionPublicationError::CurrentVerification { source }) =
                    error.downcast_ref::<RetentionPublicationError>()
                else {
                    return Err(format!(
                        "superseded publication must fail current verification: {error:?}"
                    )
                    .into());
                };
                assert!(
                    matches!(
                        source.get_ref().and_then(|source| source.downcast_ref::<RetentionCurrentStateRefusal>()),
                        Some(RetentionCurrentStateRefusal::Superseded {
                            current_generation, current_digest,
                        }) if (*current_generation, *current_digest) == (generation, digest)
                    ),
                    "stale initial must report the exact superseding manifest: {error:?}"
                );
            }
        }
        Ok(())
    }
}
