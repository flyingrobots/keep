//! This module owns retained identity evidence for one documentation source.

use std::fs::{File, Metadata};
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

use super::CorpusKind;
use crate::documentation_integrity::error::DocumentationError;
use crate::repository_file::{OpenRepositoryFileError, RepositoryRoot};

pub(super) struct AdmittedSource {
    identity: SourceIdentity,
    path: String,
    relative: PathBuf,
}

#[derive(Eq, PartialEq)]
struct SourceIdentity {
    device: u64,
    inode: u64,
    bytes: u64,
    modified_seconds: i64,
    modified_nanoseconds: i64,
    changed_seconds: i64,
    changed_nanoseconds: i64,
}

impl AdmittedSource {
    pub(super) fn admit(
        file: &File,
        path: String,
        relative: PathBuf,
        kind: CorpusKind,
    ) -> Result<Self, DocumentationError> {
        let metadata = metadata(file, kind, &path)?;
        Ok(Self {
            identity: SourceIdentity::from(&metadata),
            path,
            relative,
        })
    }

    pub(super) const fn bytes(&self) -> u64 {
        self.identity.bytes
    }

    pub(super) fn path(&self) -> &str {
        &self.path
    }

    pub(super) fn verify(
        &self,
        repository_root: &RepositoryRoot,
        kind: CorpusKind,
    ) -> Result<(), DocumentationError> {
        let current = match repository_root.open_file(&self.relative) {
            Ok(file) => file,
            Err(OpenRepositoryFileError::Io(source))
                if source.kind() == io::ErrorKind::NotFound =>
            {
                return Err(changed(kind, &self.path));
            }
            Err(OpenRepositoryFileError::Io(source)) => {
                return Err(DocumentationError::Inspect {
                    corpus: kind.label(),
                    path: self.path.clone(),
                    source,
                });
            }
            Err(OpenRepositoryFileError::NonRegular) => {
                return Err(changed(kind, &self.path));
            }
        };
        let current = SourceIdentity::from(&metadata(&current, kind, &self.path)?);
        if current == self.identity {
            Ok(())
        } else {
            Err(changed(kind, &self.path))
        }
    }
}

impl From<&Metadata> for SourceIdentity {
    fn from(metadata: &Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
            bytes: metadata.len(),
            modified_seconds: metadata.mtime(),
            modified_nanoseconds: metadata.mtime_nsec(),
            changed_seconds: metadata.ctime(),
            changed_nanoseconds: metadata.ctime_nsec(),
        }
    }
}

fn metadata(file: &File, kind: CorpusKind, path: &str) -> Result<Metadata, DocumentationError> {
    file.metadata()
        .map_err(|source| DocumentationError::Inspect {
            corpus: kind.label(),
            path: path.to_owned(),
            source,
        })
}

fn changed(kind: CorpusKind, path: &str) -> DocumentationError {
    DocumentationError::CorpusChanged {
        corpus: kind.label(),
        path: path.to_owned(),
    }
}
