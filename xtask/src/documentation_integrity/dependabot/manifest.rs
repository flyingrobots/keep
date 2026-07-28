//! This module owns tracked dependency-manifest scope discovery.

use std::collections::BTreeSet;
use xtask::protocol_admission::posix_relative_path;

use crate::git_inventory::{GitPath, paths_with};
use crate::repository_file::RepositoryProcessDirectory;

use super::DependencyScope;
use crate::documentation_integrity::error::DocumentationError;

const MANIFEST_ARGUMENTS: [&str; 5] = [
    "ls-files",
    "-z",
    "--",
    ":(glob)**/Cargo.toml",
    ":(glob)**/package.json",
];

pub(super) fn tracked_scopes(
    process_directory: &RepositoryProcessDirectory,
) -> Result<BTreeSet<DependencyScope>, DocumentationError> {
    paths_with(
        &MANIFEST_ARGUMENTS,
        "list tracked dependency manifests",
        |command| process_directory.spawn(command),
    )?
    .iter()
    .map(manifest_scope)
    .chain([Ok(DependencyScope::new("github-actions", "/"))])
    .collect()
}

fn manifest_scope(path: &GitPath) -> Result<DependencyScope, DocumentationError> {
    let text = String::from_utf8(path.as_bytes().to_vec()).map_err(|source| {
        DocumentationError::PathEncoding {
            corpus: "dependency manifest",
            source,
        }
    })?;
    posix_relative_path(&text).map_err(|_| DocumentationError::InvalidPath {
        corpus: "dependency manifest",
        path: text.clone(),
    })?;
    let (directory, filename) = text.rsplit_once('/').unwrap_or(("", &text));
    let ecosystem = match filename {
        "Cargo.toml" => "cargo",
        "package.json" => "npm",
        _ => {
            return Err(DocumentationError::InvalidPath {
                corpus: "dependency manifest",
                path: text,
            });
        }
    };
    let directory = if directory.is_empty() {
        String::from("/")
    } else {
        format!("/{directory}")
    };
    Ok(DependencyScope::new(ecosystem, &directory))
}
