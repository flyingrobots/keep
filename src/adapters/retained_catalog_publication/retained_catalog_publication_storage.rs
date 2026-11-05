//! This module owns the blocking port for retained-closure catalog publication.

use std::io;

use super::super::CatalogPublicationStorage;
use super::RetainedClosureAdmission;
use crate::{RetentionHead, RetentionManifestEntry};

/// Writer-authorized storage for catalog publication over retention state.
///
/// An implementation holds exclusive store writer authority and one pinned
/// store root for the complete call, the same authority retention publication
/// and recovery require. Retention reads return exact record bytes without
/// interpreting them; the publication gate decodes, cross-checks, and admits
/// every byte itself, so storage cannot vouch for evidence it supplies.
///
/// The catalog protocol transitions are reachable only through
/// [`catalog_stages`](Self::catalog_stages), which requires a
/// [`RetainedClosureAdmission`]. Only the gate can create that proof. An
/// implementation whose stage type is not otherwise constructible therefore
/// cannot be driven through the ungated
/// [`publish_catalog_generation`](crate::publish_catalog_generation).
pub trait RetainedCatalogPublicationStorage {
    /// Catalog protocol transitions guarded by retained-closure admission.
    type Stages: CatalogPublicationStorage;

    /// Reads the exact current retention head bytes.
    ///
    /// Returns `None` only when no retention head is published and no
    /// retention artifact contradicts that absence: no root or manifest pool
    /// entry and no fixed retention stage. Any fixed retention stage means
    /// retention recovery is required and must refuse. A present head is read
    /// without following links and bounded by its fixed 144-byte canonical
    /// length, plus at most one byte to detect trailing data.
    ///
    /// # Errors
    ///
    /// Returns the exact open, kind, read, contradiction, or
    /// recovery-required failure.
    fn read_retention_head(&mut self) -> io::Result<Option<Vec<u8>>>;

    /// Reads the exact immutable manifest that `head` selects.
    ///
    /// The read is bounded by `head.manifest_length()`. An absent manifest is
    /// an error, never an empty manifest.
    ///
    /// # Errors
    ///
    /// Returns the exact open, kind, absence, or read failure.
    fn read_retention_manifest(&mut self, head: &RetentionHead) -> io::Result<Vec<u8>>;

    /// Reads the exact immutable root that one manifest entry selects.
    ///
    /// The read is bounded by the version-2 root grammar's maximum encoded
    /// length. An absent root is an error, never an empty anchor set.
    ///
    /// # Errors
    ///
    /// Returns the exact namespace, open, kind, absence, or read failure.
    fn read_retained_root(&mut self, entry: RetentionManifestEntry) -> io::Result<Vec<u8>>;

    /// Returns the catalog protocol transitions after retained-closure admission.
    ///
    /// The method must not perform I/O; it only grants access to the stages.
    fn catalog_stages(&mut self, admission: &RetainedClosureAdmission) -> &mut Self::Stages;
}
