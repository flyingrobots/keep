//! This module owns capability-bound seed source reads and derived-state writes.

use std::fs::File;
use std::io::{self, Read, Take, Write};
use std::path::{Path, PathBuf};

use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};

use super::{FuzzSeedError, Seed};

const CORPUS_PATH: &str = "fuzz/corpus";

pub(super) struct RepositoryFiles {
    directory: Dir,
    root: PathBuf,
}

impl RepositoryFiles {
    pub(super) fn open(root: &Path) -> Result<Self, FuzzSeedError> {
        let directory = Dir::open_ambient_dir(root, ambient_authority())
            .map_err(|source| FuzzSeedError::io("open repository", root, source))?;
        Ok(Self {
            directory,
            root: root.to_path_buf(),
        })
    }

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
    let mut options = OpenOptions::new();
    options
        .write(true)
        .create(true)
        .truncate(true)
        .follow(FollowSymlinks::No)
        .nonblock(true);
    let mut file = directory
        .open_with(seed.name, &options)
        .map(cap_std::fs::File::into_std)
        .map_err(|source| FuzzSeedError::io("open seed destination", &path, source))?;
    let metadata = file
        .metadata()
        .map_err(|source| FuzzSeedError::io("inspect seed destination", &path, source))?;
    if !metadata.is_file() {
        return Err(FuzzSeedError::violation(format!(
            "seed destination is not a regular file: {}",
            path.display()
        )));
    }
    file.write_all(&seed.content)
        .map_err(|source| FuzzSeedError::io("write seed", &path, source))?;
    file.flush()
        .map_err(|source| FuzzSeedError::io("flush seed", path, source))
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
