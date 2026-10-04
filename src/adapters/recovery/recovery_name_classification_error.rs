//! This module owns recovery name-classification failures.

use std::error::Error;
use std::fmt;

use super::{
    RecoveryEntryName, RecoveryEntryRole, RecoveryNamespace, RecoveryPoolNameError,
    RecoveryRequiredEntry,
};

/// Why a recovery inventory cannot name one unambiguous protocol state.
#[derive(Debug)]
pub enum RecoveryNameClassificationError {
    /// A fixed-name namespace contains an unknown entry.
    Unexpected {
        /// Owning namespace.
        namespace: RecoveryNamespace,
        /// Exact unexpected name.
        name: RecoveryEntryName,
    },
    /// An immutable-pool name is noncanonical.
    PoolName {
        /// Owning immutable-pool namespace.
        namespace: RecoveryNamespace,
        /// Exact invalid name.
        name: RecoveryEntryName,
        /// Exact grammar refusal.
        source: RecoveryPoolNameError,
    },
    /// An initialized root entry is absent.
    Missing {
        /// Required absent entry.
        required: RecoveryRequiredEntry,
    },
    /// More than one fixed recovery stage exists.
    ConflictingStages {
        /// First stage in deterministic inventory order.
        first: RecoveryEntryRole,
        /// Second stage in deterministic inventory order.
        second: RecoveryEntryRole,
    },
}

impl fmt::Display for RecoveryNameClassificationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unexpected { namespace, name } => write!(
                formatter,
                "unexpected recovery name {:?} in {namespace}",
                name.as_bytes()
            ),
            Self::PoolName {
                namespace,
                name,
                source,
            } => write!(
                formatter,
                "invalid recovery pool name {:?} in {namespace}: {source}",
                name.as_bytes()
            ),
            Self::Missing { required } => {
                write!(formatter, "initialized recovery root is missing {required}")
            }
            Self::ConflictingStages { first, second } => write!(
                formatter,
                "recovery inventory contains conflicting stages {first:?} and {second:?}"
            ),
        }
    }
}

impl Error for RecoveryNameClassificationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::PoolName { source, .. } => Some(source),
            Self::Unexpected { .. } | Self::Missing { .. } | Self::ConflictingStages { .. } => None,
        }
    }
}
