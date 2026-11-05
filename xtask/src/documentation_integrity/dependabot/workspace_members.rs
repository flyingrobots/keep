//! This module owns workspace-member discovery from the root Cargo manifest.

use std::collections::BTreeSet;

use xtask::protocol_admission::posix_relative_path;

use super::DependencyScope;
use crate::documentation_integrity::error::DocumentationError;

pub(super) const ROOT_MANIFEST_PATH: &str = "Cargo.toml";

/// Returns the Cargo scopes that belong to the root workspace, excluding the root.
///
/// Xtask has no TOML parser and reads one fixed manifest, so this admits a single
/// inline list of exact paths. A glob or any other spelling is refused, because a
/// guess could drop a standalone manifest from Dependabot coverage.
pub(super) fn admit(manifest: &str) -> Result<BTreeSet<DependencyScope>, DocumentationError> {
    let mut lists = manifest
        .lines()
        .skip_while(|line| *line != "[workspace]")
        .skip(1)
        .take_while(|line| !line.starts_with('['))
        .filter(|line| line.starts_with("members"));
    let (Some(list), None) = (lists.next(), lists.next()) else {
        return Err(contract("workspace declares exactly one member list"));
    };
    let entries = list
        .strip_prefix("members = [")
        .and_then(|rest| rest.strip_suffix(']'))
        .ok_or_else(|| contract("workspace members are one inline list"))?;
    let mut members = BTreeSet::new();
    for entry in entries.split(", ") {
        let path = entry
            .strip_prefix('"')
            .and_then(|rest| rest.strip_suffix('"'))
            .ok_or_else(|| contract("workspace member is one quoted path"))?;
        if path == "." {
            continue;
        }
        if path.contains('*') || posix_relative_path(path).is_err() {
            return Err(contract("workspace member is one exact relative path"));
        }
        members.insert(DependencyScope::new("cargo", &format!("/{path}")));
    }
    Ok(members)
}

const fn contract(requirement: &'static str) -> DocumentationError {
    DocumentationError::RepositoryContract {
        path: ROOT_MANIFEST_PATH,
        requirement,
    }
}
