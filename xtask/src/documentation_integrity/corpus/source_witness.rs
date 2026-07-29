//! This module owns retained identity evidence for one documentation source.

use std::fs::File;
use std::io;
use std::path::PathBuf;

use super::CorpusKind;
use crate::documentation_integrity::error::DocumentationError;
use crate::repository_file::{OpenRepositoryFileError, RepositoryFileIdentity, RepositoryRoot};

pub(super) struct AdmittedSource {
    identity: RepositoryFileIdentity,
    path: String,
    relative: PathBuf,
}

impl AdmittedSource {
    pub(super) fn admit(
        file: &File,
        path: String,
        relative: PathBuf,
        kind: CorpusKind,
    ) -> Result<Self, DocumentationError> {
        let identity = identity(file, kind, &path)?;
        Ok(Self {
            identity,
            path,
            relative,
        })
    }

    pub(super) const fn bytes(&self) -> u64 {
        self.identity.bytes()
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
        let current = identity(&current, kind, &self.path)?;
        if current == self.identity {
            Ok(())
        } else {
            Err(changed(kind, &self.path))
        }
    }
}

fn identity(
    file: &File,
    kind: CorpusKind,
    path: &str,
) -> Result<RepositoryFileIdentity, DocumentationError> {
    RepositoryFileIdentity::read(file).map_err(|source| DocumentationError::Inspect {
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
