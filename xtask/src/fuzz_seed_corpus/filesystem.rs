//! This module owns capability-bound seed source reads and derived-state writes.

use std::fs::File;
use std::io::{self, Read, Take, Write};
use std::path::{Path, PathBuf};

use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};

use super::{FuzzSeedError, Seed};

const CORPUS_PATH: &str = "fuzz/corpus";

/// Capability-bound repository access for derived fuzz seed material.
///
/// The opened [`Dir`] is the authority for subsequent relative I/O; `root` is
/// retained only to render diagnostics. Operations are synchronous and may
/// block on ordinary filesystem latency, while no-follow, nonblocking opens
/// prevent links and special files from introducing ambiguous reads.
pub(super) struct RepositoryFiles {
    directory: Dir,
    root: PathBuf,
}

struct SeedStage<'a> {
    directory: &'a Dir,
    relative: PathBuf,
    active: bool,
}

impl RepositoryFiles {
    /// Open and retain the repository directory capability.
    ///
    /// # Errors
    ///
    /// Returns [`FuzzSeedError`] when the ambient repository directory cannot
    /// be opened. Opening performs filesystem I/O but allocates no content
    /// buffer and establishes no durability transition.
    pub(super) fn open(root: &Path) -> Result<Self, FuzzSeedError> {
        let directory = Dir::open_ambient_dir(root, ambient_authority())
            .map_err(|source| FuzzSeedError::io("open repository", root, source))?;
        Ok(Self {
            directory,
            root: root.to_path_buf(),
        })
    }

    /// Read one no-follow regular file under an explicit allocation bound.
    ///
    /// The source is opened capability-relative and nonblocking, must be a
    /// regular file, and is read through a `maximum + 1` byte ceiling. The
    /// observed length must equal the pre-read metadata length; same-length
    /// concurrent rewrites are outside this derived-seed verification layer.
    ///
    /// # Errors
    ///
    /// Returns [`FuzzSeedError`] for open, metadata, or read failures; links or
    /// non-regular files; arithmetic overflow; bound violations; or a detected
    /// length change. Allocation is bounded by `maximum + 1`.
    pub(super) fn read_bounded(
        &self,
        relative: &Path,
        maximum: usize,
    ) -> Result<Vec<u8>, FuzzSeedError> {
        let path = self.root.join(relative);
        let file = self.open_regular(relative, &path)?;
        let expected = file
            .metadata()
            .map_err(|source| FuzzSeedError::io("inspect seed source", &path, source))?
            .len();
        bounded_bytes(file, expected, maximum, &path)
    }

    /// Publish deterministic seeds through synced per-file stages.
    ///
    /// Each seed removes a recoverable fixed-name stage, creates a no-follow
    /// regular stage, writes and `sync_all`s its bytes, then atomically renames
    /// that stage over the derived destination. Post-create failures receive
    /// best-effort stage cleanup. Publication is one-writer, per-file, and not
    /// batch-atomic. The containing directory is not synced, so rename
    /// persistence across power loss is not claimed; the ignored corpus remains
    /// regenerable derived state.
    ///
    /// # Errors
    ///
    /// Returns [`FuzzSeedError`] when destination directories are links or
    /// non-directories, a stale stage is not removable, staging I/O or sync
    /// fails, or the atomic rename cannot publish a seed. Prior seeds from the
    /// same call may already be published when a later seed fails.
    pub(super) fn write_seeds(&self, seeds: &[Seed]) -> Result<(), FuzzSeedError> {
        let corpus = self.corpus_directory()?;
        for seed in seeds {
            let target = child_directory(&corpus, seed.target, &self.root.join(CORPUS_PATH))?;
            write_seed(&target, seed, &self.root.join(CORPUS_PATH))?;
        }
        Ok(())
    }

    fn corpus_directory(&self) -> Result<Dir, FuzzSeedError> {
        ensure_directory(&self.directory, Path::new(CORPUS_PATH), &self.root)?;
        self.directory
            .open_dir_nofollow(CORPUS_PATH)
            .map_err(|source| {
                FuzzSeedError::io("open fuzz corpus", self.root.join(CORPUS_PATH), source)
            })
    }

    fn open_regular(&self, relative: &Path, path: &Path) -> Result<File, FuzzSeedError> {
        let mut options = OpenOptions::new();
        options.read(true).follow(FollowSymlinks::No).nonblock(true);
        let file = self
            .directory
            .open_with(relative, &options)
            .map(cap_std::fs::File::into_std)
            .map_err(|source| FuzzSeedError::io("open seed source", path, source))?;
        let metadata = file
            .metadata()
            .map_err(|source| FuzzSeedError::io("inspect seed source", path, source))?;
        if metadata.is_file() {
            Ok(file)
        } else {
            Err(FuzzSeedError::violation(format!(
                "seed source is not a regular file: {}",
                path.display()
            )))
        }
    }
}

fn ensure_directory(directory: &Dir, relative: &Path, root: &Path) -> Result<(), FuzzSeedError> {
    match directory.symlink_metadata(relative) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => Ok(()),
        Ok(_) => Err(FuzzSeedError::violation(format!(
            "seed destination is not a real directory: {}",
            root.join(relative).display()
        ))),
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            directory.create_dir_all(relative).map_err(|source| {
                FuzzSeedError::io("create seed directory", root.join(relative), source)
            })
        }
        Err(source) => Err(FuzzSeedError::io(
            "inspect seed directory",
            root.join(relative),
            source,
        )),
    }
}

fn child_directory(parent: &Dir, name: &str, root: &Path) -> Result<Dir, FuzzSeedError> {
    ensure_directory(parent, Path::new(name), root)?;
    parent
        .open_dir_nofollow(name)
        .map_err(|source| FuzzSeedError::io("open seed target", root.join(name), source))
}

fn write_seed(directory: &Dir, seed: &Seed, root: &Path) -> Result<(), FuzzSeedError> {
    let path = root.join(seed.target).join(seed.name);
    let temporary = format!(".{}.keep-tmp", seed.name);
    let temporary_path = root.join(seed.target).join(&temporary);
    remove_stale_stage(directory, &temporary, &temporary_path)?;
    let mut options = OpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No)
        .nonblock(true);
    let mut file = directory
        .open_with(&temporary, &options)
        .map(cap_std::fs::File::into_std)
        .map_err(|source| FuzzSeedError::io("create temporary seed", &temporary_path, source))?;
    let stage = SeedStage::new(directory, &temporary);
    let metadata = file
        .metadata()
        .map_err(|source| FuzzSeedError::io("inspect temporary seed", &temporary_path, source))?;
    if !metadata.is_file() {
        return Err(FuzzSeedError::violation(format!(
            "temporary seed is not a regular file: {}",
            temporary_path.display()
        )));
    }
    file.write_all(&seed.content)
        .map_err(|source| FuzzSeedError::io("write temporary seed", &temporary_path, source))?;
    file.sync_all()
        .map_err(|source| FuzzSeedError::io("sync temporary seed", &temporary_path, source))?;
    drop(file);
    directory
        .rename(&temporary, directory, seed.name)
        .map_err(|source| FuzzSeedError::io("publish seed", path, source))?;
    stage.published();
    Ok(())
}

fn remove_stale_stage(directory: &Dir, temporary: &str, path: &Path) -> Result<(), FuzzSeedError> {
    match directory.symlink_metadata(temporary) {
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(FuzzSeedError::io("inspect temporary seed", path, source)),
        Ok(metadata) if metadata.is_file() || metadata.file_type().is_symlink() => directory
            .remove_file(temporary)
            .map_err(|source| FuzzSeedError::io("remove stale temporary seed", path, source)),
        Ok(_) => Err(FuzzSeedError::violation(format!(
            "temporary seed is not a removable file: {}",
            path.display()
        ))),
    }
}

impl<'a> SeedStage<'a> {
    fn new(directory: &'a Dir, relative: impl Into<PathBuf>) -> Self {
        Self {
            directory,
            relative: relative.into(),
            active: true,
        }
    }

    fn published(mut self) {
        self.active = false;
    }
}

impl Drop for SeedStage<'_> {
    fn drop(&mut self) {
        if self.active {
            drop(self.directory.remove_file(&self.relative));
        }
    }
}

fn bounded_bytes(
    file: File,
    expected: u64,
    maximum: usize,
    path: &Path,
) -> Result<Vec<u8>, FuzzSeedError> {
    let maximum_u64 = u64::try_from(maximum)
        .map_err(|source| FuzzSeedError::violation(format!("seed bound is invalid: {source}")))?;
    if expected > maximum_u64 {
        return Err(FuzzSeedError::violation(format!(
            "seed source exceeds {maximum} bytes: {}",
            path.display()
        )));
    }
    let limit = maximum_u64
        .checked_add(1)
        .ok_or_else(|| FuzzSeedError::violation("seed read bound overflow"))?;
    let mut content = Vec::new();
    let mut bounded: Take<File> = file.take(limit);
    bounded
        .read_to_end(&mut content)
        .map_err(|source| FuzzSeedError::io("read seed source", path, source))?;
    let observed = u64::try_from(content.len()).map_err(|source| {
        FuzzSeedError::violation(format!("seed source length is invalid: {source}"))
    })?;
    if observed == expected {
        Ok(content)
    } else {
        Err(FuzzSeedError::violation(format!(
            "seed source changed while reading: {}",
            path.display()
        )))
    }
}
