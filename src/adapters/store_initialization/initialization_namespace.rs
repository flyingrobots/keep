//! This module owns the root entry names that direct initialization creates or admits.

use std::ffi::OsStr;
use std::io;

use cap_std::fs::Dir;

use crate::adapters::filesystem_store_namespace;

pub(super) const WRITER_LOCK: &str = "writer.lock";
pub(super) const READER_LOCK: &str = "reader.lock";
pub(super) const MARKER: &str = "FORMAT";
pub(super) const MARKER_STAGE: &str = "FORMAT.next";
pub(super) const HEAD: &str = "HEAD";
pub(super) const NEXT_HEAD: &str = "head.next";

/// Every root name an initialization prefix or a committed store may hold.
const ROOT_NAMES: [&str; 12] = [
    WRITER_LOCK,
    READER_LOCK,
    MARKER,
    MARKER_STAGE,
    HEAD,
    NEXT_HEAD,
    "staging",
    "segments",
    "catalogs",
    "retention",
    "gc",
    "recovery",
];

pub(super) fn is_root_name(name: &OsStr) -> bool {
    ROOT_NAMES.iter().any(|candidate| name == *candidate)
}

/// Admits the committed namespace that may still hold `FORMAT.next`.
pub(super) fn admit_linked_namespace(root: &Dir) -> io::Result<()> {
    filesystem_store_namespace::admit_store_marker_linked(root)
}
