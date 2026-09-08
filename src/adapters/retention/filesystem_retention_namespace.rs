//! This module owns exact admission of the `retention` protocol namespace.

use std::io;

use cap_fs_ext::DirExt;
use cap_std::fs::Dir;

use super::AdmittedRetentionRoot;
use super::RetentionCurrentStateRefusal as Refusal;
use super::filesystem_retention_pool_name as pool_name;
use crate::{RetentionGenerationExpectation, RetentionManifest};

const CANONICAL_ENTRIES: [&str; 3] = [pool_name::HEAD, pool_name::ROOTS, pool_name::MANIFESTS];

/// Bounded observation of the admitted retention namespace.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RetentionNamespaceCensus {
    /// Digest-named root namespace directories currently present.
    pub(super) namespace_count: u32,
    /// Canonical manifest pool entries currently present.
    pub(super) manifest_count: u32,
}

impl RetentionNamespaceCensus {
    /// Returns whether no retention artifact exists in either pool.
    pub(super) const fn is_empty(self) -> bool {
        self.namespace_count == 0 && self.manifest_count == 0
    }
}

/// Admits the complete `retention` namespace before any forward write.
///
/// Every `retention` entry must be one of `HEAD`, `roots`, or `manifests`;
/// every `roots` entry must be a 64-lowercase-hex directory whose entries are
/// regular `<generation>-<digest>.root` files; every `manifests` entry must be
/// a regular `<generation>-<digest>.manifest` file. Kinds are observed without
/// following links. Any other entry is unrecoverable ambiguity and refuses.
///
/// The census is bounded: it visits every `roots` entry, every root pool, and
/// the manifest pool exactly once, retains only two counters, reads no entry
/// bytes, and classifies each entry from the directory listing's own file type.
/// Its work is therefore proportional to the entry count, which the 4,096
/// namespace ceiling and the pools' generation histories bound.
pub(super) fn admit(
    retention: &Dir,
    roots: &Dir,
    manifests: &Dir,
) -> io::Result<RetentionNamespaceCensus> {
    for entry in retention.entries()? {
        let name = entry?.file_name();
        if !CANONICAL_ENTRIES.iter().any(|canonical| name == *canonical) {
            return Err(Refusal::UnknownRetentionEntry.into_io());
        }
    }
    let mut namespace_count = 0_u32;
    for entry in roots.entries()? {
        let entry = entry?;
        let name = entry.file_name();
        if !pool_name::is_namespace_name(&name) || !entry.file_type()?.is_dir() {
            return Err(Refusal::NonNamespaceEntry.into_io());
        }
        namespace_count = namespace_count
            .checked_add(1)
            .ok_or_else(|| Refusal::NamespaceCapacity.into_io())?;
        let namespace = roots.open_dir_nofollow(&name)?;
        let _roots = admit_pool(&namespace, pool_name::ROOT_SUFFIX, "root pool")?;
    }
    let manifest_count = admit_pool(manifests, pool_name::MANIFEST_SUFFIX, "manifest pool")?;
    Ok(RetentionNamespaceCensus {
        namespace_count,
        manifest_count,
    })
}

/// Refuses a candidate that would create a namespace beyond the format ceiling.
///
/// Orphan namespace directories protected by recovery count exactly like
/// manifest entries: a candidate whose namespace directory is absent may be
/// admitted only while the observed count is below
/// [`RetentionManifest::MAXIMUM_ENTRY_COUNT`]. A candidate whose namespace
/// already exists creates nothing and is not bounded here.
pub(super) fn admit_capacity(
    census: RetentionNamespaceCensus,
    roots: &Dir,
    candidate: &AdmittedRetentionRoot<'_>,
) -> io::Result<()> {
    let name = pool_name::namespace(candidate.root().namespace().digest());
    match roots.symlink_metadata(&name) {
        Ok(_) => Ok(()),
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            if census.namespace_count < RetentionManifest::MAXIMUM_ENTRY_COUNT {
                Ok(())
            } else {
                Err(Refusal::NamespaceCapacity.into_io())
            }
        }
        Err(source) => Err(source),
    }
}

/// Requires the candidate's namespace directory to match the claimed expectation.
///
/// An `Absent` expectation asserts the namespace has never been published, so
/// any existing directory is an orphan or a substitution and refuses. A
/// `Current` expectation asserts a published generation, so an absent
/// directory means the claimed predecessor is unavailable and refuses.
pub(super) fn admit_expectation(
    roots: &Dir,
    candidate: &AdmittedRetentionRoot<'_>,
    expected: RetentionGenerationExpectation,
) -> io::Result<()> {
    let name = pool_name::namespace(candidate.root().namespace().digest());
    let observed = match roots.symlink_metadata(&name) {
        Ok(metadata) => Some(metadata.is_dir()),
        Err(source) if source.kind() == io::ErrorKind::NotFound => None,
        Err(source) => return Err(source),
    };
    match (expected, observed) {
        (RetentionGenerationExpectation::Absent, None)
        | (RetentionGenerationExpectation::Current(_), Some(true)) => Ok(()),
        (RetentionGenerationExpectation::Absent, Some(_))
        | (RetentionGenerationExpectation::Current(_), None | Some(false)) => {
            Err(Refusal::NamespaceExpectationViolated.into_io())
        }
    }
}

fn admit_pool(directory: &Dir, suffix: &str, pool: &'static str) -> io::Result<u32> {
    let mut count = 0_u32;
    for entry in directory.entries()? {
        let entry = entry?;
        if !pool_name::is_pool_name(&entry.file_name(), suffix) || !entry.file_type()?.is_file() {
            return Err(Refusal::NoncanonicalPoolEntry { pool }.into_io());
        }
        count = count
            .checked_add(1)
            .ok_or_else(|| Refusal::NamespaceCapacity.into_io())?;
    }
    Ok(count)
}
