//! This module owns deterministic documentation source selection.

mod byte_budget;
mod selection;
mod source_witness;
#[cfg(test)]
pub(super) mod test_repository;

use std::cmp::Ordering;
use std::io;
use std::path::Path;

use xtask::protocol_admission::posix_relative_path;

use super::error::DocumentationError;
use crate::git_inventory::{GitPath, paths_with};
use crate::repository_file::{OpenRepositoryFileError, RepositoryProcessDirectory, RepositoryRoot};
use byte_budget::CorpusByteBudget;
#[cfg(test)]
use byte_budget::{CORPUS_FILE_MAX_BYTES, CORPUS_MAX_BYTES};
use selection::CorpusKind;
use source_witness::AdmittedSource;

pub(super) struct SourceCorpus {
    kind: CorpusKind,
    paths: Vec<String>,
    sources: Vec<AdmittedSource>,
}

impl SourceCorpus {
    pub(super) fn markdown(
        repository_root: &RepositoryRoot,
        process_directory: &RepositoryProcessDirectory,
    ) -> Result<Self, DocumentationError> {
        Self::read(repository_root, process_directory, CorpusKind::Markdown)
    }

    pub(super) fn workflow(
        repository_root: &RepositoryRoot,
        process_directory: &RepositoryProcessDirectory,
    ) -> Result<Self, DocumentationError> {
        Self::read(repository_root, process_directory, CorpusKind::Workflow)
    }

    pub(super) fn paths(&self) -> &[String] {
        &self.paths
    }

    /// Copies every admitted source into its repository-relative snapshot path.
    ///
    /// Each copy streams from the retained source descriptor, remains within
    /// the corpus byte bounds admitted during construction, and revalidates the
    /// source identity around the copy. Filesystem or identity failures are
    /// returned through [`DocumentationError`].
    pub(super) fn materialize(&self, snapshot_root: &Path) -> Result<(), DocumentationError> {
        for source in &self.sources {
            source.materialize(snapshot_root, self.kind)?;
        }
        Ok(())
    }

    /// Revalidates every admitted source identity and the exact corpus membership.
    ///
    /// Each retained source must still match the device, inode, byte length,
    /// modification time, and change time admitted for this corpus kind. The
    /// method then reruns the bounded Git present/deleted inventories for the
    /// same kind and requires the sorted selected path set to remain identical.
    ///
    /// This operation performs repository metadata I/O and starts bounded Git
    /// child processes. It reports path replacement, in-place mutation, added
    /// or removed membership, unsafe or non-UTF-8 paths, nonregular sources, and
    /// Git or filesystem failures through [`DocumentationError`]. Successful
    /// completion confirms that every source and the corpus set remain
    /// unchanged; it does not reread source contents.
    pub(super) fn verify_unchanged(
        &self,
        repository_root: &RepositoryRoot,
        process_directory: &RepositoryProcessDirectory,
    ) -> Result<(), DocumentationError> {
        for source in &self.sources {
            source.verify(repository_root, self.kind)?;
        }
        let current = Self::read(repository_root, process_directory, self.kind)?;
        if let Some(path) = membership_change(&self.paths, &current.paths) {
            return Err(DocumentationError::CorpusChanged {
                corpus: self.kind.label(),
                path,
            });
        }
        Ok(())
    }

    fn read(
        repository_root: &RepositoryRoot,
        process_directory: &RepositoryProcessDirectory,
        kind: CorpusKind,
    ) -> Result<Self, DocumentationError> {
        let present = paths_with(
            kind.present_arguments(),
            kind.present_operation(),
            |command| process_directory.spawn(command),
        )?;
        let deleted = paths_with(
            kind.deleted_arguments(),
            kind.deleted_operation(),
            |command| process_directory.spawn(command),
        )?;
        let selected = present.difference(&deleted);
        let sources = admit_paths(repository_root, selected, kind)?;
        if sources.is_empty() {
            Err(DocumentationError::EmptyCorpus(kind.label()))
        } else {
            let paths = sources
                .iter()
                .map(|source| source.path().to_owned())
                .collect();
            Ok(Self {
                kind,
                paths,
                sources,
            })
        }
    }
}

fn membership_change(expected: &[String], observed: &[String]) -> Option<String> {
    let mut expected = expected.iter().peekable();
    let mut observed = observed.iter().peekable();
    loop {
        match (expected.peek(), observed.peek()) {
            (Some(left), Some(right)) => match left.cmp(right) {
                Ordering::Equal => {
                    expected.next();
                    observed.next();
                }
                Ordering::Less => return Some((*left).clone()),
                Ordering::Greater => return Some((*right).clone()),
            },
            (Some(path), None) | (None, Some(path)) => return Some((*path).clone()),
            (None, None) => return None,
        }
    }
}

fn admit_paths<'a>(
    repository_root: &RepositoryRoot,
    paths: impl Iterator<Item = &'a GitPath>,
    kind: CorpusKind,
) -> Result<Vec<AdmittedSource>, DocumentationError> {
    let mut admitted = Vec::new();
    let mut budget = CorpusByteBudget::default();
    for path in paths {
        if let Some(source) = admit_path(repository_root, path, kind)? {
            budget.admit(kind, source.path(), source.bytes())?;
            admitted.push(source);
        }
    }
    Ok(admitted)
}

fn admit_path(
    repository_root: &RepositoryRoot,
    path: &GitPath,
    kind: CorpusKind,
) -> Result<Option<AdmittedSource>, DocumentationError> {
    let text = String::from_utf8(path.as_bytes().to_vec()).map_err(|source| {
        DocumentationError::PathEncoding {
            corpus: kind.label(),
            source,
        }
    })?;
    let relative = posix_relative_path(&text).map_err(|_| DocumentationError::InvalidPath {
        corpus: kind.label(),
        path: text.clone(),
    })?;
    match repository_root.open_file(&relative) {
        Ok(file) => Ok(Some(AdmittedSource::admit(file, text, relative, kind)?)),
        Err(OpenRepositoryFileError::Io(source)) if source.kind() == io::ErrorKind::NotFound => {
            Ok(None)
        }
        Err(OpenRepositoryFileError::Io(source)) => Err(DocumentationError::Inspect {
            corpus: kind.label(),
            path: text,
            source,
        }),
        Err(OpenRepositoryFileError::NonRegular) => Err(DocumentationError::NonRegular {
            corpus: kind.label(),
            path: text,
        }),
    }
}

#[cfg(test)]
#[path = "corpus/replacement_tests.rs"]
mod replacement_tests;
#[cfg(test)]
#[path = "corpus/tests.rs"]
mod tests;
