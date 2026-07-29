//! This module owns bounded recovery-inventory failures.

use std::error::Error;
use std::fmt;
use std::io;

use super::{RecoveryEntryName, RecoveryInventoryOperation, RecoveryNamespace};

/// Why read-only recovery inventory could not produce one exact snapshot.
#[derive(Debug)]
pub enum RecoveryInventoryError {
    /// A storage operation failed.
    Io {
        /// Namespace being inspected.
        namespace: RecoveryNamespace,
        /// Exact failed operation.
        operation: RecoveryInventoryOperation,
        /// Underlying storage refusal.
        source: io::Error,
    },
    /// Counting exceeded the configured entry ceiling.
    EntryLimit {
        /// Admitted maximum.
        maximum: u64,
        /// Smallest count proved before stopping.
        observed_at_least: u64,
    },
    /// A namespace changed between count and name reads.
    Changed {
        /// Namespace that changed.
        namespace: RecoveryNamespace,
        /// Previously observed count.
        counted: u64,
        /// Count returned by the bounded name read.
        observed: u64,
    },
    /// One namespace returned the same raw name more than once.
    Duplicate {
        /// Namespace containing the duplicate.
        namespace: RecoveryNamespace,
        /// Exact duplicate raw name.
        name: RecoveryEntryName,
    },
    /// The admitted count cannot fit the current address space.
    AddressSpace {
        /// Entry count that could not be represented.
        observed: u64,
    },
}

impl RecoveryInventoryError {
    pub(super) const fn io(
        namespace: RecoveryNamespace,
        operation: RecoveryInventoryOperation,
        source: io::Error,
    ) -> Self {
        Self::Io {
            namespace,
            operation,
            source,
        }
    }
}

impl fmt::Display for RecoveryInventoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io {
                namespace,
                operation,
                source,
            } => write!(
                formatter,
                "recovery inventory {operation} failed in {namespace}: {source}"
            ),
            Self::EntryLimit {
                maximum,
                observed_at_least,
            } => write!(
                formatter,
                "recovery inventory exceeds {maximum} entries; observed at least {observed_at_least}"
            ),
            Self::Changed {
                namespace,
                counted,
                observed,
            } => write!(
                formatter,
                "{namespace} changed during recovery inventory: counted {counted}, observed {observed}"
            ),
            Self::Duplicate { namespace, name } => write!(
                formatter,
                "{namespace} returned duplicate recovery name {:?}",
                name.as_bytes()
            ),
            Self::AddressSpace { observed } => write!(
                formatter,
                "recovery inventory count {observed} does not fit the address space"
            ),
        }
    }
}

impl Error for RecoveryInventoryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::EntryLimit { .. }
            | Self::Changed { .. }
            | Self::Duplicate { .. }
            | Self::AddressSpace { .. } => None,
        }
    }
}
