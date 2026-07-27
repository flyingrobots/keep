//! This module owns collision-resistant, scoped filesystem test directories.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const CREATION_ATTEMPTS: u16 = 1_024;
static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

pub(crate) struct TestDirectory {
    path: PathBuf,
    active: bool,
}

impl TestDirectory {
    pub(crate) fn create(label: &str) -> Result<Self, io::Error> {
        for _ in 0_u16..CREATION_ATTEMPTS {
            let sequence = next_sequence()?;
            let path = std::env::temp_dir()
                .join(format!("keep-{label}-{}-{sequence}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path, active: true }),
                Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {}
                Err(source) => return Err(source),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "test directory collision bound exhausted",
        ))
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn close(mut self) -> Result<(), io::Error> {
        let removal = match fs::remove_dir_all(&self.path) {
            Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
            result => result,
        };
        removal?;
        self.active = false;
        Ok(())
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        if self.active {
            drop(fs::remove_dir_all(&self.path));
        }
    }
}

fn next_sequence() -> Result<u64, io::Error> {
    NEXT_DIRECTORY
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .map_err(|_| io::Error::other("test directory sequence exhausted"))
}

#[cfg(test)]
mod tests {
    use super::TestDirectory;

    #[test]
    fn scoped_test_directories_do_not_reuse_live_paths() -> Result<(), std::io::Error> {
        let first = TestDirectory::create("collision-law")?;
        let second = TestDirectory::create("collision-law")?;
        assert_ne!(first.path(), second.path());
        first.close()?;
        second.close()
    }
}
