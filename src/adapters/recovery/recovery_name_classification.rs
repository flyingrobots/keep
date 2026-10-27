//! This module owns deterministic recovery entry-name classification.

use super::{
    RecoveryEntryName, RecoveryEntryRole, RecoveryInventory, RecoveryNameClassificationError,
    RecoveryNameManifest, RecoveryNamedEntry, RecoveryNamespace, RecoveryRequiredEntry,
    recovery_pool_name,
};

/// Classifies one complete committed version-two inventory without opening bytes.
///
/// The operation preserves inventory order and allocates at most one manifest
/// entry per inventory entry. It rejects unknown names, missing root entries,
/// noncanonical pool coordinates, and simultaneous fixed stages. The root
/// entries `reader.lock`, `FORMAT`, `retention`, `gc`, and `recovery` are
/// classified as known roles, never as unexpected or ignored. Each is required;
/// catalog `HEAD` is not, because the first publication of an initialized
/// store can be interrupted before it exists. `FORMAT.next` is unknown: a
/// committed root never holds it.
/// The contents of `retention`, `gc`, and `recovery` are not inventoried here;
/// version-two admission and retention recovery own them.
///
/// # Errors
///
/// Returns [`RecoveryNameClassificationError`] at the first deterministic
/// namespace, grammar, required-entry, or stage-conflict failure.
pub fn classify_version_two_recovery_names(
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
        b"reader.lock" => Ok(RecoveryEntryRole::ReaderFence),
        b"FORMAT" => Ok(RecoveryEntryRole::FormatMarker),
        b"retention" => Ok(RecoveryEntryRole::RetentionDirectory),
        b"gc" => Ok(RecoveryEntryRole::GcDirectory),
        b"recovery" => Ok(RecoveryEntryRole::RecoveryDirectory),
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

/// Required root entries in report order.
const REQUIRED: [RecoveryRequiredEntry; 9] = [
    RecoveryRequiredEntry::WriterLock,
    RecoveryRequiredEntry::StagingDirectory,
    RecoveryRequiredEntry::SegmentPoolDirectory,
    RecoveryRequiredEntry::CatalogPoolDirectory,
    RecoveryRequiredEntry::ReaderFence,
    RecoveryRequiredEntry::FormatMarker,
    RecoveryRequiredEntry::RetentionDirectory,
    RecoveryRequiredEntry::GcDirectory,
    RecoveryRequiredEntry::RecoveryDirectory,
];

#[derive(Default)]
struct RequiredEntries {
    observed: [bool; REQUIRED.len()],
}

impl RequiredEntries {
    fn observe(&mut self, role: RecoveryEntryRole) {
        let Some(required) = required_entry(role) else {
            return;
        };
        for (seen, entry) in self.observed.iter_mut().zip(REQUIRED) {
            if entry == required {
                *seen = true;
            }
        }
    }

    fn first_missing(&self) -> Option<RecoveryRequiredEntry> {
        REQUIRED
            .into_iter()
            .zip(self.observed)
            .find_map(|(entry, seen)| (!seen).then_some(entry))
    }
}

const fn required_entry(role: RecoveryEntryRole) -> Option<RecoveryRequiredEntry> {
    match role {
        RecoveryEntryRole::WriterLock => Some(RecoveryRequiredEntry::WriterLock),
        RecoveryEntryRole::StagingDirectory => Some(RecoveryRequiredEntry::StagingDirectory),
        RecoveryEntryRole::SegmentPoolDirectory => {
            Some(RecoveryRequiredEntry::SegmentPoolDirectory)
        }
        RecoveryEntryRole::CatalogPoolDirectory => {
            Some(RecoveryRequiredEntry::CatalogPoolDirectory)
        }
        RecoveryEntryRole::ReaderFence => Some(RecoveryRequiredEntry::ReaderFence),
        RecoveryEntryRole::FormatMarker => Some(RecoveryRequiredEntry::FormatMarker),
        RecoveryEntryRole::RetentionDirectory => Some(RecoveryRequiredEntry::RetentionDirectory),
        RecoveryEntryRole::GcDirectory => Some(RecoveryRequiredEntry::GcDirectory),
        RecoveryEntryRole::RecoveryDirectory => Some(RecoveryRequiredEntry::RecoveryDirectory),
        RecoveryEntryRole::CurrentHead
        | RecoveryEntryRole::NextHeadStage
        | RecoveryEntryRole::SegmentStage
        | RecoveryEntryRole::CatalogStage
        | RecoveryEntryRole::ImmutableSegment { .. }
        | RecoveryEntryRole::ImmutableCatalog { .. } => None,
    }
}
