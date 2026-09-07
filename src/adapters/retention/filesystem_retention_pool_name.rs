//! Exact retention immutable-pool and namespace filename emission.

use std::fmt;

use crate::{
    LivenessGeneration, RetentionManifestDigest, RetentionNamespaceDigest, RetentionRootDigest,
    RootGeneration,
};

pub(super) const RETENTION: &str = "retention";
pub(super) const ROOTS: &str = "roots";
pub(super) const MANIFESTS: &str = "manifests";
pub(super) const HEAD: &str = "HEAD";
pub(super) const ROOT_STAGE: &str = "root.next";
pub(super) const MANIFEST_STAGE: &str = "manifest.next";
pub(super) const HEAD_STAGE: &str = "head.next";

pub(super) fn namespace(digest: RetentionNamespaceDigest) -> String {
    DigestHex(digest.as_bytes()).to_string()
}

pub(super) fn root(generation: RootGeneration, digest: RetentionRootDigest) -> String {
    format!(
        "{:016x}-{}.root",
        generation.get(),
        DigestHex(digest.as_bytes())
    )
}

pub(super) fn manifest(generation: LivenessGeneration, digest: RetentionManifestDigest) -> String {
    format!(
        "{:016x}-{}.manifest",
        generation.get(),
        DigestHex(digest.as_bytes())
    )
}

struct DigestHex<'a>(&'a [u8; 32]);

impl fmt::Display for DigestHex<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}
