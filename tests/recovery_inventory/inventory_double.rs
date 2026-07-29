//! Fault-recording recovery inventory port.

use std::io;

use keep::{RecoveryEntryName, RecoveryInventoryStorage, RecoveryNamespace};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// One observed inventory-port call.
pub enum InventoryCall {
    /// Count the selected namespace.
    Count(RecoveryNamespace),
    /// Read the selected namespace with the admitted expected count.
    Read(RecoveryNamespace, u64),
}

/// Deterministic recovery-inventory storage double.
pub struct InventoryDouble {
    counts: [u64; 4],
    names: [Vec<RecoveryEntryName>; 4],
    calls: Vec<InventoryCall>,
}

impl InventoryDouble {
    pub(crate) const fn new(counts: [u64; 4], names: [Vec<RecoveryEntryName>; 4]) -> Self {
        Self {
            counts,
            names,
            calls: Vec::new(),
        }
    }

    pub(crate) fn calls(&self) -> &[InventoryCall] {
        &self.calls
    }
}

impl RecoveryInventoryStorage for InventoryDouble {
    fn count_entries(&mut self, namespace: RecoveryNamespace) -> io::Result<u64> {
        self.calls.push(InventoryCall::Count(namespace));
        let [root, staging, segments, catalogs] = self.counts;
        Ok(match namespace {
            RecoveryNamespace::Root => root,
            RecoveryNamespace::Staging => staging,
            RecoveryNamespace::Segments => segments,
            RecoveryNamespace::Catalogs => catalogs,
        })
    }

    fn read_entry_names(
        &mut self,
        namespace: RecoveryNamespace,
        expected_count: u64,
    ) -> io::Result<Vec<RecoveryEntryName>> {
        self.calls
            .push(InventoryCall::Read(namespace, expected_count));
        let [root, staging, segments, catalogs] = &self.names;
        Ok(match namespace {
            RecoveryNamespace::Root => root,
            RecoveryNamespace::Staging => staging,
            RecoveryNamespace::Segments => segments,
            RecoveryNamespace::Catalogs => catalogs,
        }
        .clone())
    }
}
