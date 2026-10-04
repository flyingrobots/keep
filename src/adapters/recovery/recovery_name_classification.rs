//! This module owns deterministic recovery entry-name classification.

use super::{
    RecoveryEntryName, RecoveryEntryRole, RecoveryInventory, RecoveryNameClassificationError,
    RecoveryNameManifest, RecoveryNamedEntry, RecoveryNamespace, RecoveryRequiredEntry,
    recovery_pool_name,
};

/// Classifies one complete inventory without opening artifact bytes.
///
/// The operation preserves inventory order and allocates at most one manifest
/// entry per inventory entry. It rejects unknown names, missing initialized
/// root entries, noncanonical pool coordinates, and simultaneous fixed stages.
///
/// # Errors
///
/// Returns [`RecoveryNameClassificationError`] at the first deterministic
/// namespace, grammar, required-entry, or stage-conflict failure.
pub fn classify_recovery_names(
    inventory: RecoveryInventory,
) -> Result<RecoveryNameManifest, RecoveryNameClassificationError> {
    let mut required = RequiredEntries::default();
    let mut first_stage = None;
    let mut named = Vec::with_capacity(inventory.entries().len());
    for entry in inventory.into_entries() {
        let (namespace, name) = entry.into_parts();
        let role = match classify(namespace, &name) {
            Ok(role) => role,
            Err(NameFailure::Unexpected) => {
                return Err(RecoveryNameClassificationError::Unexpected { namespace, name });
            }
            Err(NameFailure::Pool(source)) => {
                return Err(RecoveryNameClassificationError::PoolName {
                    namespace,
                    name,
                    source,
                });
            }
        };
        required.observe(role);
        if role.is_stage() {
            if let Some(first) = first_stage {
                return Err(RecoveryNameClassificationError::ConflictingStages {
                    first,
                    second: role,
                });
            }
            first_stage = Some(role);
        }
        named.push(RecoveryNamedEntry::new(namespace, name, role));
    }
    if let Some(required) = required.first_missing() {
        return Err(RecoveryNameClassificationError::Missing { required });
    }
    Ok(RecoveryNameManifest::new(named))
}

fn classify(
    namespace: RecoveryNamespace,
    name: &RecoveryEntryName,
) -> Result<RecoveryEntryRole, NameFailure> {
    match namespace {
        RecoveryNamespace::Root => classify_root(name),
        RecoveryNamespace::Staging => classify_staging(name),
        RecoveryNamespace::Segments => recovery_pool_name::segment(name)
            .map(|digest| RecoveryEntryRole::ImmutableSegment { digest })
            .map_err(NameFailure::Pool),
        RecoveryNamespace::Catalogs => recovery_pool_name::catalog(name)
            .map(|(generation, digest)| RecoveryEntryRole::ImmutableCatalog { generation, digest })
            .map_err(NameFailure::Pool),
    }
}

fn classify_root(name: &RecoveryEntryName) -> Result<RecoveryEntryRole, NameFailure> {
    match name.as_bytes() {
        b"writer.lock" => Ok(RecoveryEntryRole::WriterLock),
        b"staging" => Ok(RecoveryEntryRole::StagingDirectory),
        b"segments" => Ok(RecoveryEntryRole::SegmentPoolDirectory),
        b"catalogs" => Ok(RecoveryEntryRole::CatalogPoolDirectory),
        b"HEAD" => Ok(RecoveryEntryRole::CurrentHead),
        b"head.next" => Ok(RecoveryEntryRole::NextHeadStage),
        _ => Err(NameFailure::Unexpected),
    }
}

fn classify_staging(name: &RecoveryEntryName) -> Result<RecoveryEntryRole, NameFailure> {
    match name.as_bytes() {
        b"current.seg" => Ok(RecoveryEntryRole::SegmentStage),
        b"current.cat" => Ok(RecoveryEntryRole::CatalogStage),
        _ => Err(NameFailure::Unexpected),
    }
}

enum NameFailure {
    Unexpected,
    Pool(super::RecoveryPoolNameError),
}

#[derive(Default)]
struct RequiredEntries(u8);

impl RequiredEntries {
    const WRITER_LOCK: u8 = 1 << 0;
    const STAGING: u8 = 1 << 1;
    const SEGMENTS: u8 = 1 << 2;
    const CATALOGS: u8 = 1 << 3;

    const fn observe(&mut self, role: RecoveryEntryRole) {
        match role {
            RecoveryEntryRole::WriterLock => self.0 |= Self::WRITER_LOCK,
            RecoveryEntryRole::StagingDirectory => self.0 |= Self::STAGING,
            RecoveryEntryRole::SegmentPoolDirectory => self.0 |= Self::SEGMENTS,
            RecoveryEntryRole::CatalogPoolDirectory => self.0 |= Self::CATALOGS,
            RecoveryEntryRole::CurrentHead
            | RecoveryEntryRole::NextHeadStage
            | RecoveryEntryRole::SegmentStage
            | RecoveryEntryRole::CatalogStage
            | RecoveryEntryRole::ImmutableSegment { .. }
            | RecoveryEntryRole::ImmutableCatalog { .. } => {}
        }
    }

    const fn first_missing(&self) -> Option<RecoveryRequiredEntry> {
        if self.0 & Self::WRITER_LOCK == 0 {
            Some(RecoveryRequiredEntry::WriterLock)
        } else if self.0 & Self::STAGING == 0 {
            Some(RecoveryRequiredEntry::StagingDirectory)
        } else if self.0 & Self::SEGMENTS == 0 {
            Some(RecoveryRequiredEntry::SegmentPoolDirectory)
        } else if self.0 & Self::CATALOGS == 0 {
            Some(RecoveryRequiredEntry::CatalogPoolDirectory)
        } else {
            None
        }
    }
}
