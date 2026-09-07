//! This module owns exact variable-length retention stage publication.

use std::io::{self, Read, Write};

use cap_fs_ext::{FollowSymlinks, MetadataExt, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::fs::{Dir, File, Metadata, OpenOptions};

use crate::adapters::filesystem_catalog_artifact;

/// One exclusively created, verified, and retained retention stage file.
///
/// The stage retains its opened handle and recorded device and inode identity
/// for its whole lifetime. Every transition reverifies both the handle and the
/// named entry, so a replaced or byte-equal substituted file refuses instead of
/// being admitted.
pub(super) struct FilesystemRetentionStage {
    name: &'static str,
    expected: Box<[u8]>,
    identity: StageIdentity,
    file: File,
}

impl FilesystemRetentionStage {
    /// Exclusively creates the named stage and writes its complete bytes.
    pub(super) fn create(root: &Dir, name: &'static str, expected: &[u8]) -> io::Result<Self> {
        let mut file = filesystem_catalog_artifact::create_exclusive(root, name)?;
        let identity = StageIdentity::read_file(&file)?;
        file.write_all(expected)?;
        file.flush()?;
        Ok(Self {
            name,
            expected: Box::from(expected),
            identity,
            file,
        })
    }

    /// Synchronizes the complete stage and reverifies its exact bytes.
    pub(super) fn synchronize(&self, root: &Dir) -> io::Result<()> {
        self.require_handle()?;
        self.file.sync_all()?;
        self.verify_stage(root)
    }

    /// Links the verified stage into `target` under `name` without replacement.
    pub(super) fn link(&self, root: &Dir, target: &Dir, name: &str) -> io::Result<()> {
        self.verify_stage(root)?;
        match root.hard_link(self.name, target, name) {
            Ok(()) => {}
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {}
            Err(source) => return Err(source),
        }
        self.verify_stage(root)?;
        verify_name(target, name, &self.expected, self.identity)
    }

    /// Removes only the retained stage after confirming its linked target.
    pub(super) fn remove(self, root: &Dir, target: &Dir, name: &str) -> io::Result<()> {
        verify_name(target, name, &self.expected, self.identity)?;
        root.remove_file(self.name)?;
        require_absent(root, self.name)?;
        verify_name(target, name, &self.expected, self.identity)
    }

    /// Renames the verified stage onto `name`, replacing it atomically.
    pub(super) fn replace(self, root: &Dir, name: &str) -> io::Result<()> {
        self.verify_stage(root)?;
        root.rename(self.name, root, name)?;
        require_absent(root, self.name)?;
        verify_name(root, name, &self.expected, self.identity)
    }

    fn require_handle(&self) -> io::Result<()> {
        if StageIdentity::read_file(&self.file)? == self.identity {
            Ok(())
        } else {
            Err(invalid_data("retention stage handle changed identity"))
        }
    }

    fn verify_stage(&self, root: &Dir) -> io::Result<()> {
        self.require_handle()?;
        verify_name(root, self.name, &self.expected, self.identity)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct StageIdentity {
    device: u64,
    inode: u64,
}

impl StageIdentity {
    fn read_file(file: &File) -> io::Result<Self> {
        file.metadata().map(|metadata| Self::from(&metadata))
    }
}

impl From<&Metadata> for StageIdentity {
    fn from(metadata: &Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
        }
    }
}

fn verify_name(
    directory: &Dir,
    name: &str,
    expected: &[u8],
    identity: StageIdentity,
) -> io::Result<()> {
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No).nonblock(true);
    let mut file = directory.open_with(name, &options)?;
    require_metadata(&file.metadata()?, expected.len(), identity)?;
    require_metadata(&directory.symlink_metadata(name)?, expected.len(), identity)?;
    let mut observed = vec![0_u8; expected.len()];
    file.read_exact(&mut observed)?;
    let mut trailing = [0_u8; 1];
    if observed != expected || file.read(&mut trailing)? != 0 {
        return Err(invalid_data("retention record bytes disagreed"));
    }
    require_metadata(&file.metadata()?, expected.len(), identity)?;
    require_metadata(&directory.symlink_metadata(name)?, expected.len(), identity)
}

fn require_metadata(
    metadata: &Metadata,
    expected_length: usize,
    expected_identity: StageIdentity,
) -> io::Result<()> {
    let expected_length = u64::try_from(expected_length)
        .map_err(|_source| invalid_data("retention record length exceeded u64"))?;
    if metadata.is_file()
        && metadata.len() == expected_length
        && StageIdentity::from(metadata) == expected_identity
    {
        Ok(())
    } else {
        Err(invalid_data(
            "retention record kind, length, or identity disagreed",
        ))
    }
}

fn require_absent(directory: &Dir, name: &str) -> io::Result<()> {
    match directory.symlink_metadata(name) {
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
        Ok(_) => Err(invalid_data("removed retention stage remained visible")),
        Err(source) => Err(source),
    }
}

pub(super) fn invalid_data(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
