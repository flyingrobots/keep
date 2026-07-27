//! Owned canonical flat-layout record.

use std::fmt;

use super::{LayoutEncodeError, layout_record_encoder};
use crate::layout::{AdmittedLayout, LayoutId};

/// Exact canonical bytes and identity for one admitted flat layout.
///
/// Encoding materializes the complete durable record in memory. The byte
/// count is bounded by [`crate::LayoutRecordLength::MAXIMUM`].
#[must_use]
#[derive(Clone, Eq, PartialEq)]
pub struct CanonicalLayoutRecord {
    bytes: Box<[u8]>,
    id: LayoutId,
}

impl CanonicalLayoutRecord {
    pub(super) fn from_parts(bytes: Vec<u8>, id: LayoutId) -> Self {
        Self {
            bytes: bytes.into_boxed_slice(),
            id,
        }
    }

    /// Returns the exact canonical durable record bytes.
    #[must_use]
    pub const fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the identity calculated from the complete canonical record.
    #[must_use]
    pub const fn id(&self) -> LayoutId {
        self.id
    }

    /// Consumes the record and returns its exact canonical bytes.
    #[must_use]
    pub fn into_bytes(self) -> Box<[u8]> {
        self.bytes
    }
}

impl fmt::Debug for CanonicalLayoutRecord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CanonicalLayoutRecord")
            .field("length", &self.bytes.len())
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

impl AdmittedLayout {
    /// Encodes this admitted layout into the exact version-1 durable record.
    ///
    /// This materializes one buffer proportional to the entry count and
    /// bounded by [`crate::LayoutRecordLength::MAXIMUM`]. It performs no I/O.
    ///
    /// # Errors
    ///
    /// Returns [`LayoutEncodeError`] for checked conversion, length,
    /// allocation, or internal emission-invariant failures.
    pub fn encode_record(&self) -> Result<CanonicalLayoutRecord, LayoutEncodeError> {
        layout_record_encoder::encode_layout(self)
    }
}
