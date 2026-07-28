//! This module owns bounded admission of retained fuzz corpus state.

mod error;
#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

pub(crate) use error::CorpusError;

use super::policy::CampaignPolicy;
use super::target;

#[derive(Debug, Eq, PartialEq)]
pub(super) struct CorpusStats {
    files: u64,
    bytes: u64,
}

impl CorpusStats {
    pub(super) const fn files(&self) -> u64 {
        self.files
    }

    pub(super) const fn bytes(&self) -> u64 {
        self.bytes
    }
}

pub(super) fn audit(
    root: &Path,
    repository_root: &Path,
    policy: &CampaignPolicy,
) -> Result<CorpusStats, CorpusError> {
    let root_metadata = match fs::symlink_metadata(root) {
        Ok(metadata) => metadata,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            return Ok(CorpusStats { files: 0, bytes: 0 });
        }
        Err(source) => {
            return Err(CorpusError::Inspect {
                path: root.to_owned(),
                source,
            });
        }
    };
    if !root_metadata.file_type().is_dir() {
        return Err(CorpusError::RootNotDirectory(root.to_owned()));
    }
    let expected = target::harnesses(repository_root)?
        .into_iter()
        .map(|target| target.as_str().to_owned())
        .collect::<BTreeSet<_>>();
    admit_targets(root, &expected, policy)
}

fn admit_targets(
    root: &Path,
    expected: &BTreeSet<String>,
    policy: &CampaignPolicy,
) -> Result<CorpusStats, CorpusError> {
    let mut stats = CorpusStats { files: 0, bytes: 0 };
    for entry in sorted_entries(root)? {
        let target_path = entry.path();
        let metadata = inspect(&target_path)?;
        let known = target_path
            .file_name()
            .is_some_and(|name| expected.iter().any(|target| name == target.as_str()));
        if !metadata.file_type().is_dir() || !known {
            return Err(CorpusError::UnexpectedTarget(target_path));
        }
        for corpus_entry in sorted_entries(&target_path)? {
            admit_file(corpus_entry.path(), policy, &mut stats)?;
        }
    }
    Ok(stats)
}

fn admit_file(
    path: PathBuf,
    policy: &CampaignPolicy,
    stats: &mut CorpusStats,
) -> Result<(), CorpusError> {
    let metadata = inspect(&path)?;
    if !metadata.file_type().is_file() {
        return Err(CorpusError::NonRegular(path));
    }
    let size = metadata.len();
    if size > policy.max_input_bytes() {
        return Err(CorpusError::InputBound {
            path,
            maximum: policy.max_input_bytes(),
            observed: size,
        });
    }
    stats.files = stats
        .files
        .checked_add(1)
        .ok_or(CorpusError::FileCountOverflow)?;
    stats.bytes = stats
        .bytes
        .checked_add(size)
        .ok_or(CorpusError::ByteCountOverflow)?;
    if stats.files > policy.corpus_max_files() {
        return Err(CorpusError::FileCountBound {
            maximum: policy.corpus_max_files(),
        });
    }
    if stats.bytes > policy.corpus_max_bytes() {
        return Err(CorpusError::ByteCountBound {
            maximum: policy.corpus_max_bytes(),
        });
    }
    Ok(())
}

fn sorted_entries(directory: &Path) -> Result<Vec<DirEntry>, CorpusError> {
    let entries = fs::read_dir(directory).map_err(|source| CorpusError::ReadDirectory {
        path: directory.to_owned(),
        source,
    })?;
    let mut admitted = Vec::new();
    for entry in entries {
        admitted.push(entry.map_err(|source| CorpusError::ReadEntry {
            path: directory.to_owned(),
            source,
        })?);
    }
    admitted.sort_by_key(DirEntry::file_name);
    Ok(admitted)
}

fn inspect(path: &Path) -> Result<fs::Metadata, CorpusError> {
    fs::symlink_metadata(path).map_err(|source| CorpusError::Inspect {
        path: path.to_owned(),
        source,
    })
}
