//! This module owns exact variable-length retention stage publication.

use std::io::{self, Write};

use cap_std::fs::{Dir, File};

use crate::adapters::filesystem_catalog_artifact;
use crate::adapters::filesystem_exact_record::{
    self as exact_record, EntryIdentity, ExactRecordError, ExactRecordRefusal,
};

/// One exclusively created, verified, and retained retention stage file.
///
/// The stage retains its opened handle and recorded device and inode identity
/// for its whole lifetime. Every transition reverifies both the handle and the
/// named entry, so a replaced or byte-equal substituted file refuses instead of
/// being admitted.
pub(super) struct FilesystemRetentionStage {
    name: &'static str,
    expected: Box<[u8]>,
    identity: EntryIdentity,
    file: File,
}

impl FilesystemRetentionStage {
    /// Exclusively creates the named stage and writes its complete bytes.
    pub(super) fn create(root: &Dir, name: &'static str, expected: &[u8]) -> io::Result<Self> {
        let mut file = filesystem_catalog_artifact::create_exclusive(root, name)?;
        let identity = EntryIdentity::of_file(&file)?;
        file.write_all(expected)?;
        file.flush()?;
        Ok(Self {
            name,
            expected: Box::from(expected),
            identity,
            file,
        })
    }

    /// Reopens a retained stage whose exact bytes restart already read.
    ///
    /// The handle and the named entry are verified against `expected` and
    /// bound to the entry's identity, so every later transition refuses a
    /// substituted or replaced stage exactly as a freshly created one would.
    pub(super) fn reopen(root: &Dir, name: &'static str, expected: &[u8]) -> io::Result<Self> {
        let file = exact_record::open_read(root, name)?;
        let identity = EntryIdentity::of_file(&file)?;
        verify_named_record(root, name, expected, identity)?;
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
        exact_record::link_without_replacement(root, self.name, target, name)?;
        self.verify_stage(root)?;
        verify_named_record(target, name, &self.expected, self.identity)
    }

    /// Removes only the retained stage after confirming its linked target.
    pub(super) fn remove(self, root: &Dir, target: &Dir, name: &str) -> io::Result<()> {
        verify_named_record(target, name, &self.expected, self.identity)?;
        root.remove_file(self.name)?;
        exact_record::require_absent(root, self.name).map_err(retention_error)?;
        verify_named_record(target, name, &self.expected, self.identity)
    }

    /// Renames the verified stage onto `name`, replacing it atomically.
    pub(super) fn replace(self, root: &Dir, name: &str) -> io::Result<()> {
        self.verify_stage(root)?;
        root.rename(self.name, root, name)?;
        exact_record::require_absent(root, self.name).map_err(retention_error)?;
        verify_named_record(root, name, &self.expected, self.identity)
    }

    fn require_handle(&self) -> io::Result<()> {
        if EntryIdentity::of_file(&self.file)? == self.identity {
            Ok(())
        } else {
            Err(invalid_data("retention stage handle changed identity"))
        }
    }

    fn verify_stage(&self, root: &Dir) -> io::Result<()> {
        self.require_handle()?;
        verify_named_record(root, self.name, &self.expected, self.identity)
    }
}

fn verify_named_record(
    directory: &Dir,
    name: &str,
    expected: &[u8],
    identity: EntryIdentity,
) -> io::Result<()> {
    exact_record::verify_named(directory, name, expected, identity).map_err(retention_error)
}

/// Maps a shared exact-record failure onto this protocol's refusal messages.
fn retention_error(error: ExactRecordError) -> io::Error {
    match error {
        ExactRecordError::Io(source) => source,
        ExactRecordError::Refused(refusal) => invalid_data(match refusal {
            ExactRecordRefusal::LengthOverflow => "retention record length exceeded u64",
            ExactRecordRefusal::KindOrLength | ExactRecordRefusal::KindLengthOrIdentity => {
                "retention record kind, length, or identity disagreed"
            }
            ExactRecordRefusal::Bytes | ExactRecordRefusal::TrailingBytes => {
                "retention record bytes disagreed"
            }
            ExactRecordRefusal::RemainedVisible => "removed retention stage remained visible",
        }),
    }
}

pub(super) fn invalid_data(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
