//! This module owns exact admission of the `retention` protocol namespace.

use std::ffi::OsStr;
use std::io;

use cap_fs_ext::DirExt;
use cap_std::fs::Dir;

use super::filesystem_retention_pool_name as pool_name;
use super::filesystem_retention_stage::invalid_data;

const CANONICAL_ENTRIES: [&str; 3] = [pool_name::HEAD, pool_name::ROOTS, pool_name::MANIFESTS];
const DIGEST_HEX: usize = 64;
const GENERATION_HEX: usize = 16;
const ROOT_SUFFIX: &str = ".root";
const MANIFEST_SUFFIX: &str = ".manifest";

/// Bounded observation of the admitted retention namespace.
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RetentionNamespaceCensus {
    /// Digest-named root namespace directories currently present.
    pub(super) namespace_count: u32,
}

/// Admits the complete `retention` namespace before any forward write.
///
/// Every `retention` entry must be one of `HEAD`, `roots`, or `manifests`;
/// every `roots` entry must be a 64-lowercase-hex directory whose entries are
/// regular `<generation>-<digest>.root` files; every `manifests` entry must be
/// a regular `<generation>-<digest>.manifest` file. Kinds are observed without
/// following links. Any other entry is unrecoverable ambiguity and refuses.
pub(super) fn admit(
    retention: &Dir,
    roots: &Dir,
    manifests: &Dir,
) -> io::Result<RetentionNamespaceCensus> {
    for entry in retention.entries()? {
        let name = entry?.file_name();
        if !CANONICAL_ENTRIES.iter().any(|canonical| name == *canonical) {
            return Err(invalid_data("retention namespace carries an unknown entry"));
        }
    }
    let mut namespace_count = 0_u32;
    for entry in roots.entries()? {
        let entry = entry?;
        let name = entry.file_name();
        if !is_lower_hex(&name, DIGEST_HEX) || !entry.metadata()?.is_dir() {
            return Err(invalid_data(
                "retention roots carries a non-namespace entry",
            ));
        }
        namespace_count = namespace_count
            .checked_add(1)
            .ok_or_else(|| invalid_data("retention namespace count overflowed"))?;
        let namespace = roots.open_dir_nofollow(&name)?;
        admit_pool(&namespace, ROOT_SUFFIX, "retention root pool")?;
    }
    admit_pool(manifests, MANIFEST_SUFFIX, "retention manifest pool")?;
    Ok(RetentionNamespaceCensus { namespace_count })
}

fn admit_pool(pool: &Dir, suffix: &str, label: &'static str) -> io::Result<()> {
    for entry in pool.entries()? {
        let entry = entry?;
        if !is_pool_name(&entry.file_name(), suffix) || !entry.metadata()?.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{label} carries a noncanonical entry"),
            ));
        }
    }
    Ok(())
}

fn is_pool_name(name: &OsStr, suffix: &str) -> bool {
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
