//! This module owns contradictions between a selected root and its namespace.

use std::error::Error;
use std::fmt;

use crate::RetentionNamespaceDigest;

/// A canonically decoded root contradicts the manifest selection that named it.
///
/// The public reader preserves this cause inside its root-boundary I/O error.
/// This refusal proves a contradiction; it is not an operational read failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RetentionSelectedRootRefusal {
    /// The root belongs to a different namespace than the manifest entry.
    Namespace {
        /// Namespace selected by the manifest and requested by the reader.
        expected: RetentionNamespaceDigest,
        /// Namespace authenticated by the decoded canonical root.
        observed: RetentionNamespaceDigest,
    },
}

impl fmt::Display for RetentionSelectedRootRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Namespace { .. } => {
                formatter.write_str("selected root belongs to a different retention namespace")
            }
        }
    }
}

impl Error for RetentionSelectedRootRefusal {}
