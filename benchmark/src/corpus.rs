//! Named ownership of the generated benchmark input corpus.

use keep::BlobId;

/// Complete deterministic input corpus for the streaming CAS benchmark.
///
/// Every byte is generated locally from fixed repository-owned rules. The
/// corpus embeds no third-party source or binary payload.
pub struct BenchmarkCorpus {
    pub(super) large_text: Vec<u8>,
    pub(super) large_binary: Vec<u8>,
    pub(super) edit_base: Vec<u8>,
    pub(super) early_insertion: Vec<u8>,
    pub(super) early_deletion: Vec<u8>,
    pub(super) near_neighbor: Vec<u8>,
    pub(super) zero_dedup: Vec<u8>,
    pub(super) tiny_blobs: Vec<Box<[u8]>>,
    pub(super) identities: [BlobId; Self::MEMBER_COUNT],
    pub(super) total_bytes: usize,
}

impl BenchmarkCorpus {
    /// Maximum aggregate bytes retained by one generated corpus.
    pub const TOTAL_BYTE_LIMIT: usize = 8_388_608;
    /// Number of named large or edit-oriented corpus members.
    pub const MEMBER_COUNT: usize = 7;

    /// Returns all named large or edit-oriented members in canonical order.
    #[must_use]
    pub fn members(&self) -> [&[u8]; Self::MEMBER_COUNT] {
        [
            &self.large_text,
            &self.large_binary,
            &self.edit_base,
            &self.early_insertion,
            &self.early_deletion,
            &self.near_neighbor,
            &self.zero_dedup,
        ]
    }

    /// Returns canonical member names aligned with [`Self::members`].
    #[must_use]
    pub const fn member_names(&self) -> [&'static str; Self::MEMBER_COUNT] {
        [
            "large-text",
            "large-binary",
            "edit-base",
            "early-insertion",
            "early-deletion",
            "near-neighbor",
            "zero-dedup",
        ]
    }

    /// Returns exact logical identities aligned with [`Self::members`].
    #[must_use]
    pub const fn identities(&self) -> &[BlobId; Self::MEMBER_COUNT] {
        &self.identities
    }

    /// Returns aggregate retained input bytes across every corpus member.
    #[must_use]
    pub const fn total_bytes(&self) -> usize {
        self.total_bytes
    }

    /// Returns the generated large source-like text member.
    #[must_use]
    pub fn large_text(&self) -> &[u8] {
        &self.large_text
    }

    /// Returns the generated large opaque binary member.
    #[must_use]
    pub fn large_binary(&self) -> &[u8] {
        &self.large_binary
    }

    /// Returns the base bytes for edit-reuse comparisons.
    #[must_use]
    pub fn edit_base(&self) -> &[u8] {
        &self.edit_base
    }

    /// Returns the base with a deterministic early insertion.
    #[must_use]
    pub fn early_insertion(&self) -> &[u8] {
        &self.early_insertion
    }

    /// Returns the base with a deterministic early deletion.
    #[must_use]
    pub fn early_deletion(&self) -> &[u8] {
        &self.early_deletion
    }

    /// Returns the base with one deterministic near-neighbor substitution.
    #[must_use]
    pub fn near_neighbor(&self) -> &[u8] {
        &self.near_neighbor
    }

    /// Returns deterministic bytes designed to share no intentional content.
    #[must_use]
    pub fn zero_dedup(&self) -> &[u8] {
        &self.zero_dedup
    }

    /// Returns the deterministic many-tiny-blob corpus.
    #[must_use]
    pub fn tiny_blobs(&self) -> &[Box<[u8]>] {
        &self.tiny_blobs
    }
}

#[cfg(test)]
#[path = "corpus_tests.rs"]
mod tests;
