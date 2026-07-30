//! This boundary module owns canonical migration inventory entries.

use crate::{AdmittedCatalog, AdmittedSegment};

use super::migration_catalog_admission::AdmittedMigrationCatalog;

const SEGMENT_KIND: u8 = 1;
const CATALOG_KIND: u8 = 2;
const ENCODED_LENGTH: usize = 56;

/// Canonical physical coordinate for one admitted version-1 pool artifact.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct StoreMigrationInventoryEntry([u8; ENCODED_LENGTH]);

impl StoreMigrationInventoryEntry {
    /// Constructs the canonical entry for one completely admitted segment.
    pub const fn from_segment(segment: &AdmittedSegment<'_>) -> Self {
        Self(encode(
            SEGMENT_KIND,
            0,
            segment.segment_length(),
            segment.digest().as_bytes(),
        ))
    }

    /// Constructs the canonical entry for one completely admitted catalog.
    pub const fn from_catalog(catalog: &AdmittedCatalog<'_, '_>) -> Self {
        Self(encode(
            CATALOG_KIND,
            catalog.generation().get(),
            catalog.length().get(),
            catalog.digest().as_bytes(),
        ))
    }

    pub(super) const fn from_migration_catalog(catalog: &AdmittedMigrationCatalog<'_>) -> Self {
        Self(encode(
            CATALOG_KIND,
            catalog.generation().get(),
            catalog.length().get(),
            catalog.digest().as_bytes(),
        ))
    }

    /// Returns the exact 56 canonical bytes.
    pub const fn encoded(&self) -> &[u8; ENCODED_LENGTH] {
        &self.0
    }
}

const fn encode(kind: u8, generation: u64, length: u64, digest: &[u8; 32]) -> [u8; ENCODED_LENGTH] {
    let mut encoded = [0_u8; ENCODED_LENGTH];
    let (kind_and_reserved, remainder) = encoded.split_at_mut(8);
    kind_and_reserved.copy_from_slice(&[kind, 0, 0, 0, 0, 0, 0, 0]);
    let (generation_bytes, remainder) = remainder.split_at_mut(8);
    generation_bytes.copy_from_slice(&generation.to_be_bytes());
    let (length_bytes, digest_bytes) = remainder.split_at_mut(8);
    length_bytes.copy_from_slice(&length.to_be_bytes());
    digest_bytes.copy_from_slice(digest);
    encoded
}
