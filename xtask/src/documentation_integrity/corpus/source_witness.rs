//! This module owns retained identity evidence for one documentation source.

use std::fs::File;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{FileExt, OpenOptionsExt};
use std::path::Path;
use std::path::PathBuf;

use super::CorpusKind;
use crate::documentation_integrity::error::DocumentationError;
use crate::repository_file::{OpenRepositoryFileError, RepositoryFileIdentity, RepositoryRoot};

pub(super) struct AdmittedSource {
    file: File,
    identity: RepositoryFileIdentity,
    path: String,
    relative: PathBuf,
}

impl AdmittedSource {
    pub(super) fn admit(
        file: File,
        path: String,
        relative: PathBuf,
        kind: CorpusKind,
    ) -> Result<Self, DocumentationError> {
        let identity = identity(&file, kind, &path)?;
        Ok(Self {
            file,
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

    pub(super) fn materialize(
        &self,
        snapshot_root: &Path,
        kind: CorpusKind,
    ) -> Result<(), DocumentationError> {
        self.verify_retained(kind)?;
        let destination = snapshot_root.join(&self.relative);
        let parent = destination.parent().ok_or_else(|| {
            snapshot_io(
                "resolve documentation snapshot parent",
                io::Error::other("source path has no parent"),
            )
        })?;
        fs::create_dir_all(parent)
            .map_err(|source| snapshot_io("create documentation snapshot directory", source))?;
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o400)
            .open(&destination)
            .map_err(|source| snapshot_io("create documentation snapshot source", source))?;
        copy_exact(&self.file, &mut output, self.identity.bytes())
            .map_err(|source| snapshot_io("copy documentation snapshot source", source))?;
        self.verify_retained(kind)
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

    fn verify_retained(&self, kind: CorpusKind) -> Result<(), DocumentationError> {
        let current = identity(&self.file, kind, &self.path)?;
        if current == self.identity {
            Ok(())
        } else {
            Err(changed(kind, &self.path))
        }
    }
}

fn copy_exact(source: &File, destination: &mut File, expected: u64) -> Result<(), io::Error> {
    let mut offset = 0_u64;
    let mut buffer = [0_u8; 16_384];
    while offset < expected {
        let remaining = expected
            .checked_sub(offset)
            .ok_or_else(|| io::Error::other("snapshot source offset exceeded its length"))?;
        let limit =
            usize::try_from(remaining).map_or(buffer.len(), |bytes| bytes.min(buffer.len()));
        let chunk = buffer
            .get_mut(..limit)
            .ok_or_else(|| io::Error::other("snapshot read bound exceeded its buffer"))?;
        let read = source.read_at(chunk, offset)?;
        if read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "snapshot source ended before its admitted length",
            ));
        }
        let copied = buffer
            .get(..read)
            .ok_or_else(|| io::Error::other("snapshot write bound exceeded its buffer"))?;
        destination.write_all(copied)?;
        offset = offset
            .checked_add(u64::try_from(read).map_err(|_| {
                io::Error::other("snapshot source read length is not representable")
            })?)
            .ok_or_else(|| io::Error::other("snapshot source offset overflowed"))?;
    }
    Ok(())
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

const fn snapshot_io(action: &'static str, source: io::Error) -> DocumentationError {
    DocumentationError::Snapshot { action, source }
}
