//! This module owns bounded, deterministic recovery inventory orchestration.

use super::{
    RecoveryEntryName, RecoveryInventoryError, RecoveryInventoryLimit, RecoveryInventoryOperation,
    RecoveryInventoryStorage, RecoveryNamespace,
};

const NAMESPACES: [RecoveryNamespace; 4] = [
    RecoveryNamespace::Root,
    RecoveryNamespace::Staging,
    RecoveryNamespace::Segments,
    RecoveryNamespace::Catalogs,
];

/// One namespace-qualified entry in a recovery inventory.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RecoveryInventoryEntry {
    namespace: RecoveryNamespace,
    name: RecoveryEntryName,
}

impl RecoveryInventoryEntry {
    /// Returns the owning protocol namespace.
    #[must_use]
    pub const fn namespace(&self) -> RecoveryNamespace {
        self.namespace
    }

    /// Returns the exact raw name.
    #[must_use]
    pub const fn name(&self) -> &RecoveryEntryName {
        &self.name
    }
}

/// One duplicate-free, deterministically ordered recovery inventory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryInventory {
    entries: Vec<RecoveryInventoryEntry>,
}

impl RecoveryInventory {
    /// Returns entries ordered by namespace and raw name bytes.
    #[must_use]
    pub fn entries(&self) -> &[RecoveryInventoryEntry] {
        &self.entries
    }
}

/// Counts every namespace before retaining and sorting any entry name.
///
/// The operation is synchronous and may block on storage I/O. Its allocation
/// is bounded by `limit` and the admitted entry-name component lengths. It
/// never writes, repairs, deletes, or substitutes storage state.
///
/// # Errors
///
/// Returns [`RecoveryInventoryError`] on storage refusal, entry-limit excess,
/// count drift, duplicate names, or address-space incompatibility.
pub fn read_recovery_inventory(
    storage: &mut impl RecoveryInventoryStorage,
    limit: RecoveryInventoryLimit,
) -> Result<RecoveryInventory, RecoveryInventoryError> {
    let counts = count_namespaces(storage, limit)?;
    read_namespaces(storage, counts)
}

fn count_namespaces(
    storage: &mut impl RecoveryInventoryStorage,
    limit: RecoveryInventoryLimit,
) -> Result<[u64; 4], RecoveryInventoryError> {
    let mut counts = [0_u64; 4];
    let mut total = 0_u64;
    for (count_slot, namespace) in counts.iter_mut().zip(NAMESPACES) {
        let count = storage.count_entries(namespace).map_err(|source| {
            RecoveryInventoryError::io(namespace, RecoveryInventoryOperation::Count, source)
        })?;
        let remaining = limit
            .get()
            .checked_sub(total)
            .ok_or(RecoveryInventoryError::AddressSpace { observed: total })?;
        if count > remaining {
            let observed_at_least =
                limit
                    .get()
                    .checked_add(1)
                    .ok_or_else(|| RecoveryInventoryError::AddressSpace {
                        observed: limit.get(),
                    })?;
            return Err(RecoveryInventoryError::EntryLimit {
                maximum: limit.get(),
                observed_at_least,
            });
        }
        total = total
            .checked_add(count)
            .ok_or(RecoveryInventoryError::AddressSpace { observed: count })?;
        *count_slot = count;
    }
    Ok(counts)
}

fn read_namespaces(
    storage: &mut impl RecoveryInventoryStorage,
    counts: [u64; 4],
) -> Result<RecoveryInventory, RecoveryInventoryError> {
    let total = counts.into_iter().try_fold(0_u64, |total, count| {
        total
            .checked_add(count)
            .ok_or(RecoveryInventoryError::AddressSpace { observed: count })
    })?;
    let capacity = usize::try_from(total)
        .map_err(|_| RecoveryInventoryError::AddressSpace { observed: total })?;
    let mut entries = Vec::with_capacity(capacity);
    for (namespace, counted) in NAMESPACES.into_iter().zip(counts) {
        let names = storage
            .read_entry_names(namespace, counted)
            .map_err(|source| {
                RecoveryInventoryError::io(namespace, RecoveryInventoryOperation::ReadNames, source)
            })?;
        admit_names(&mut entries, namespace, counted, names)?;
    }
    entries.sort_unstable();
    refuse_duplicate(&entries)?;
    Ok(RecoveryInventory { entries })
}

fn admit_names(
    entries: &mut Vec<RecoveryInventoryEntry>,
    namespace: RecoveryNamespace,
    counted: u64,
    names: Vec<RecoveryEntryName>,
) -> Result<(), RecoveryInventoryError> {
    let observed = u64::try_from(names.len())
        .map_err(|_| RecoveryInventoryError::AddressSpace { observed: counted })?;
    if observed != counted {
        return Err(RecoveryInventoryError::Changed {
            namespace,
            counted,
            observed,
        });
    }
    entries.extend(
        names
            .into_iter()
            .map(|name| RecoveryInventoryEntry { namespace, name }),
    );
    Ok(())
}

fn refuse_duplicate(entries: &[RecoveryInventoryEntry]) -> Result<(), RecoveryInventoryError> {
    for pair in entries.windows(2) {
        let [left, right] = pair else {
            continue;
        };
        if left == right {
            return Err(RecoveryInventoryError::Duplicate {
                namespace: left.namespace,
                name: left.name.clone(),
            });
        }
    }
    Ok(())
}
