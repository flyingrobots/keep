//! This module owns immutable documentation-tool input snapshots.

mod namespace;

use std::collections::BTreeSet;
use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::documentation_integrity::corpus::SourceCorpus;
use crate::documentation_integrity::error::DocumentationError;
use crate::documentation_integrity::repository_text::{self, RepositoryText};
use crate::repository_file::{RepositoryProcessDirectory, RepositoryRoot};

const CREATION_ATTEMPTS: u16 = 1_024;
const MARKDOWNLINT_CONFIG: &str = ".markdownlint-cli2.yaml";
static NEXT_SNAPSHOT: AtomicU64 = AtomicU64::new(0);

/// Private repository-shaped inputs and process authority for documentation tools.
///
/// Selected sources and the Markdown configuration contain the exact admitted
/// bytes. Other representable present regular files are copied exactly through
/// descriptor-bound, identity-checked reads so offline link validation observes
/// faithful target bytes and file types. The owned directory has no durability
/// role and is removed explicitly.
pub(super) struct DocumentationSnapshot {
    directory: SnapshotDirectory,
    process_directory: RepositoryProcessDirectory,
    repository_root: RepositoryRoot,
}

impl DocumentationSnapshot {
    /// Materializes and revalidates one snapshot from the retained repository.
    ///
    /// The operation allocates bounded deterministic inventories, performs
    /// filesystem writes, and starts bounded Git inventory children. It returns
    /// typed corpus, policy-file, Git, or snapshot I/O failures.
    pub(super) fn create(
        source_root: &RepositoryRoot,
        source_process_directory: &RepositoryProcessDirectory,
        corpora: &[&SourceCorpus],
    ) -> Result<Self, DocumentationError> {
        let config = repository_text::read(source_root, MARKDOWNLINT_CONFIG)?;
        let directory = SnapshotDirectory::create()?;
        let materialized = materialize(directory.path(), corpora, &config)?;
        namespace::materialize(
            directory.path(),
            source_root,
            source_process_directory,
            &materialized,
        )?;
        verify_sources(source_root, source_process_directory, corpora, &config)?;
        let repository_root = open_snapshot_root(directory.path())?;
        let process_directory = repository_root.process_directory().map_err(|source| {
            snapshot_io("open documentation snapshot process directory", source)
        })?;
        Ok(Self {
            directory,
            process_directory,
            repository_root,
        })
    }

    /// Returns the descriptor-backed child working directory for the snapshot.
    pub(super) const fn process_directory(&self) -> &RepositoryProcessDirectory {
        &self.process_directory
    }

    /// Returns the capability-relative root used by in-process snapshot readers.
    pub(super) const fn repository_root(&self) -> &RepositoryRoot {
        &self.repository_root
    }

    /// Closes owned directory descriptors and removes the exact snapshot tree.
    ///
    /// Cleanup does not mutate the source repository and reports removal failure
    /// through [`DocumentationError::Snapshot`].
    pub(super) fn close(self) -> Result<(), DocumentationError> {
        let Self {
            directory,
            process_directory,
            repository_root,
        } = self;
        drop(process_directory);
        drop(repository_root);
        directory
            .close()
            .map_err(|source| snapshot_io("remove documentation snapshot", source))
    }
}

fn materialize(
    destination: &Path,
    corpora: &[&SourceCorpus],
    config: &RepositoryText,
) -> Result<BTreeSet<PathBuf>, DocumentationError> {
    let mut materialized = BTreeSet::new();
    for corpus in corpora {
        corpus.materialize(destination)?;
        for path in corpus.paths() {
            materialized.insert(PathBuf::from(path));
        }
    }
    write_config(destination, config)?;
    materialized.insert(PathBuf::from(MARKDOWNLINT_CONFIG));
    Ok(materialized)
}

fn write_config(destination: &Path, config: &RepositoryText) -> Result<(), DocumentationError> {
    let path = destination.join(MARKDOWNLINT_CONFIG);
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o400)
        .open(path)
        .map_err(|source| snapshot_io("create documentation snapshot configuration", source))?;
    output
        .write_all(config.as_str().as_bytes())
        .map_err(|source| snapshot_io("write documentation snapshot configuration", source))
}

fn verify_sources(
    source_root: &RepositoryRoot,
    process_directory: &RepositoryProcessDirectory,
    corpora: &[&SourceCorpus],
    config: &RepositoryText,
) -> Result<(), DocumentationError> {
    for corpus in corpora {
        corpus.verify_unchanged(source_root, process_directory)?;
    }
    config.verify(source_root)
}

fn open_snapshot_root(path: &Path) -> Result<RepositoryRoot, DocumentationError> {
    RepositoryRoot::open(path)
        .map_err(|source| snapshot_io("open documentation snapshot root", source))
}

struct SnapshotDirectory {
    path: PathBuf,
    active: bool,
}

impl SnapshotDirectory {
    fn create() -> Result<Self, DocumentationError> {
        for _ in 0_u16..CREATION_ATTEMPTS {
            let sequence = next_sequence()?;
            let path = std::env::temp_dir().join(format!(
                "keep-documentation-snapshot-{}-{sequence}",
                std::process::id()
            ));
            let mut builder = DirBuilder::new();
            builder.mode(0o700);
            match builder.create(&path) {
                Ok(()) => return Ok(Self { path, active: true }),
                Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {}
                Err(source) => {
                    return Err(snapshot_io("create documentation snapshot", source));
                }
            }
        }
        Err(snapshot_io(
            "create documentation snapshot",
            io::Error::new(
                io::ErrorKind::AlreadyExists,
                "documentation snapshot collision bound exhausted",
            ),
        ))
    }

    fn path(&self) -> &Path {
        self.path.as_path()
    }

    fn close(mut self) -> Result<(), io::Error> {
        fs::remove_dir_all(&self.path)?;
        self.active = false;
        Ok(())
    }
}

impl Drop for SnapshotDirectory {
    fn drop(&mut self) {
        if self.active {
            drop(fs::remove_dir_all(&self.path));
        }
    }
}

fn next_sequence() -> Result<u64, DocumentationError> {
    NEXT_SNAPSHOT
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .map_err(|_| {
            snapshot_io(
                "allocate documentation snapshot identity",
                io::Error::other("documentation snapshot sequence exhausted"),
            )
        })
}

const fn snapshot_io(action: &'static str, source: io::Error) -> DocumentationError {
    DocumentationError::Snapshot { action, source }
}

#[cfg(test)]
#[path = "snapshot/tests.rs"]
mod tests;
