//! This module owns faithful non-corpus snapshot namespace materialization.

use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use xtask::protocol_admission::posix_relative_path;

use crate::documentation_integrity::error::DocumentationError;
use crate::git_inventory::{GitPath, paths_with};
use crate::repository_file::{
    OpenRepositoryFileError, RepositoryFileIdentity, RepositoryProcessDirectory, RepositoryRoot,
    copy_exact,
};

const CORPUS: &str = "documentation snapshot namespace";
const FILE_LIMIT_BYTES: u64 = 4 * 1_024 * 1_024;
const NAMESPACE_DELETED: [&str; 3] = ["ls-files", "-z", "--deleted"];
const NAMESPACE_PRESENT: [&str; 5] = [
    "ls-files",
    "-z",
    "--cached",
    "--others",
    "--exclude-per-directory=.gitignore",
];
const TOTAL_LIMIT_BYTES: u64 = 64 * 1_024 * 1_024;

pub(super) fn materialize(
    destination: &Path,
    source_root: &RepositoryRoot,
    process_directory: &RepositoryProcessDirectory,
    materialized: &BTreeSet<PathBuf>,
) -> Result<(), DocumentationError> {
    let present = paths_with(
        &NAMESPACE_PRESENT,
        "git documentation snapshot present paths",
        |command| process_directory.spawn(command),
    )?;
    let deleted = paths_with(
        &NAMESPACE_DELETED,
        "git documentation snapshot deleted paths",
        |command| process_directory.spawn(command),
    )?;
    let mut total = 0_u64;
    for path in present.difference(&deleted) {
        if let Some((relative, text)) = snapshot_relative(path)?
            && !materialized.contains(&relative)
        {
            total = materialize_file(destination, source_root, &relative, &text, total)?;
        }
    }
    Ok(())
}

fn materialize_file(
    destination: &Path,
    source_root: &RepositoryRoot,
    relative: &Path,
    path: &str,
    total: u64,
) -> Result<u64, DocumentationError> {
    let file = open_source(source_root, relative, path)?;
    let admitted = identity(&file, path)?;
    refuse_file_bound(path, admitted.bytes())?;
    let next_total = total
        .checked_add(admitted.bytes())
        .ok_or(DocumentationError::CorpusSizeOverflow(CORPUS))?;
    refuse_total_bound(next_total)?;
    let mut output = create_destination(destination, relative)?;
    copy_exact(&file, &mut output, admitted.bytes())
        .map_err(|source| snapshot_io("copy documentation snapshot namespace file", source))?;
    verify_identity(&file, source_root, relative, path, &admitted)?;
    Ok(next_total)
}

fn open_source(
    source_root: &RepositoryRoot,
    relative: &Path,
    path: &str,
) -> Result<File, DocumentationError> {
    match source_root.open_file(relative) {
        Ok(file) => Ok(file),
        Err(OpenRepositoryFileError::Io(source)) => Err(DocumentationError::Inspect {
            corpus: CORPUS,
            path: path.to_owned(),
            source,
        }),
        Err(OpenRepositoryFileError::NonRegular) => Err(DocumentationError::NonRegular {
            corpus: CORPUS,
            path: path.to_owned(),
        }),
    }
}

fn create_destination(destination: &Path, relative: &Path) -> Result<File, DocumentationError> {
    let path = destination.join(relative);
    let parent = path.parent().ok_or_else(|| {
        snapshot_io(
            "resolve documentation snapshot namespace parent",
            io::Error::other("namespace path has no parent"),
        )
    })?;
    fs::create_dir_all(parent)
        .map_err(|source| snapshot_io("create documentation snapshot namespace", source))?;
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o400)
        .open(path)
        .map_err(|source| snapshot_io("create documentation snapshot namespace file", source))
}

fn verify_identity(
    retained: &File,
    source_root: &RepositoryRoot,
    relative: &Path,
    path: &str,
    admitted: &RepositoryFileIdentity,
) -> Result<(), DocumentationError> {
    let retained_identity = identity(retained, path)?;
    let current = open_source(source_root, relative, path)?;
    let current_identity = identity(&current, path)?;
    if &retained_identity == admitted && &current_identity == admitted {
        Ok(())
    } else {
        Err(DocumentationError::CorpusChanged {
            corpus: CORPUS,
            path: path.to_owned(),
        })
    }
}

fn identity(file: &File, path: &str) -> Result<RepositoryFileIdentity, DocumentationError> {
    RepositoryFileIdentity::read(file).map_err(|source| DocumentationError::Inspect {
        corpus: CORPUS,
        path: path.to_owned(),
        source,
    })
}

fn refuse_file_bound(path: &str, observed: u64) -> Result<(), DocumentationError> {
    if observed <= FILE_LIMIT_BYTES {
        Ok(())
    } else {
        Err(DocumentationError::CorpusFileTooLarge {
            corpus: CORPUS,
            path: path.to_owned(),
            maximum: FILE_LIMIT_BYTES,
            observed,
        })
    }
}

const fn refuse_total_bound(observed: u64) -> Result<(), DocumentationError> {
    if observed <= TOTAL_LIMIT_BYTES {
        Ok(())
    } else {
        Err(DocumentationError::CorpusTooLarge {
            corpus: CORPUS,
            maximum: TOTAL_LIMIT_BYTES,
            observed,
        })
    }
}

fn snapshot_relative(path: &GitPath) -> Result<Option<(PathBuf, String)>, DocumentationError> {
    let Ok(text) = std::str::from_utf8(path.as_bytes()) else {
        return Ok(None);
    };
    posix_relative_path(text)
        .map(|relative| Some((relative, text.to_owned())))
        .map_err(|_| DocumentationError::InvalidPath {
            corpus: CORPUS,
            path: text.to_owned(),
        })
}

const fn snapshot_io(action: &'static str, source: io::Error) -> DocumentationError {
    DocumentationError::Snapshot { action, source }
}
