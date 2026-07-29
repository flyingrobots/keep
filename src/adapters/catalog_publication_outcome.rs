//! Final outcome of one synchronized catalog publication attempt.

/// Whether this call published a generation or verified an earlier completion.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogPublicationOutcome {
    /// This call completed the catalog and head publication protocol.
    Published,
    /// The proposed generation was already current and was durably reverified.
    AlreadyPublished,
}
