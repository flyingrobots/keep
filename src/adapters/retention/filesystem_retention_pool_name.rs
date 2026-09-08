//! Exact retention immutable-pool and namespace filename emission and admission.
//!
//! The emitters and the predicates that admit their output live together so
//! the on-disk grammar cannot drift between writing and census.

use std::ffi::OsStr;

use crate::adapters::digest_hex::DigestHex;
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
pub(super) const ROOT_SUFFIX: &str = ".root";
pub(super) const MANIFEST_SUFFIX: &str = ".manifest";
const DIGEST_HEX: usize = 64;
const GENERATION_HEX: usize = 16;

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

/// Whether `name` is a 64-lowercase-hex namespace directory name.
pub(super) fn is_namespace_name(name: &OsStr) -> bool {
    is_lower_hex(name, DIGEST_HEX)
}

/// Whether `name` is a canonical `<generation>-<digest><suffix>` pool entry name.
pub(super) fn is_pool_name(name: &OsStr, suffix: &str) -> bool {
    let Some(name) = name.to_str() else {
        return false;
    };
    let Some(stem) = name.strip_suffix(suffix) else {
        return false;
    };
    let Some((generation, digest)) = stem.split_once('-') else {
        return false;
    };
    is_lower_hex(OsStr::new(generation), GENERATION_HEX)
        && is_lower_hex(OsStr::new(digest), DIGEST_HEX)
}

fn is_lower_hex(name: &OsStr, length: usize) -> bool {
    name.to_str().is_some_and(|text| {
        text.len() == length
            && text
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}
