//! This module owns the exact-record filesystem primitives shared by stage
//! publishers and fixed-record readers.
//!
//! Every open here is read-only, follows no links, and does not block, so a
//! FIFO or device planted at a protocol name refuses by kind instead of
//! hanging under the writer lock. Every read is bounded by the caller's exact
//! expected length and refuses trailing bytes. Callers map each
//! [`ExactRecordRefusal`] to their own typed refusal or message, so the
//! primitives carry no protocol vocabulary of their own.

use std::error::Error;
use std::fmt;
use std::io::{self, Read};

use cap_fs_ext::{FollowSymlinks, MetadataExt, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::fs::{Dir, File, Metadata, OpenOptions};

/// Device and inode identity of one directory entry or open handle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct EntryIdentity {
    device: u64,
    inode: u64,
}

impl EntryIdentity {
    /// Reads the identity behind an open file handle.
    pub(super) fn of_file(file: &File) -> io::Result<Self> {
        file.metadata().map(|metadata| Self::from(&metadata))
    }
}

impl From<&Metadata> for EntryIdentity {
    fn from(metadata: &Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
        }
    }
}

/// Why an exact record refused, independent of which protocol named it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ExactRecordRefusal {
    /// The expected length does not fit the filesystem's `u64` length.
    LengthOverflow,
    /// The entry is not a regular file of the expected length.
    KindOrLength,
    /// The entry's kind, length, or device and inode identity disagreed.
    KindLengthOrIdentity,
    /// The entry's bytes disagreed with the expected record.
    Bytes,
    /// The entry carried bytes beyond the expected length.
    TrailingBytes,
    /// An entry that must be absent is still visible.
    RemainedVisible,
}

/// An exact-record failure: the filesystem's own error or a typed refusal.
#[derive(Debug)]
pub(super) enum ExactRecordError {
    /// The filesystem refused the operation.
    Io(io::Error),
    /// The entry exists but is not the expected record.
    Refused(ExactRecordRefusal),
}

impl fmt::Display for ExactRecordRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::LengthOverflow => "exact record length exceeded the filesystem's range",
            Self::KindOrLength => "exact record kind or length disagreed",
            Self::KindLengthOrIdentity => "exact record kind, length, or identity disagreed",
            Self::Bytes => "exact record bytes disagreed",
            Self::TrailingBytes => "exact record carried trailing bytes",
            Self::RemainedVisible => "removed exact record remained visible",
        })
    }
}

impl fmt::Display for ExactRecordError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(source) => write!(formatter, "exact record I/O failed: {source}"),
            Self::Refused(refusal) => fmt::Display::fmt(refusal, formatter),
        }
    }
}

impl Error for ExactRecordError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(source) => Some(source),
            Self::Refused(_) => None,
        }
    }
}

impl From<io::Error> for ExactRecordError {
    fn from(source: io::Error) -> Self {
        Self::Io(source)
    }
}

impl From<ExactRecordRefusal> for ExactRecordError {
    fn from(refusal: ExactRecordRefusal) -> Self {
        Self::Refused(refusal)
    }
}

/// Reads exactly `length` bytes of the regular file `name`, or `None` if absent.
///
/// The open follows no links and does not block. A present entry that is not
/// a regular file of exactly `length` bytes refuses before any byte is read,
/// and bytes beyond `length` refuse after.
pub(super) fn read_exact_optional(
    directory: &Dir,
    name: &str,
    length: usize,
) -> Result<Option<Vec<u8>>, ExactRecordError> {
    let mut file = match open_read(directory, name) {
        Ok(file) => file,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(source.into()),
    };
    read_opened_exactly(&mut file, length).map(Some)
}

/// Reads exactly `length` bytes of the regular file `name`; absence is the
/// filesystem's own `NotFound` error, since the record is required.
pub(super) fn read_exact_regular(
    directory: &Dir,
    name: &str,
    length: usize,
) -> Result<Vec<u8>, ExactRecordError> {
    let mut file = open_read(directory, name)?;
    read_opened_exactly(&mut file, length)
}

fn read_opened_exactly(file: &mut File, length: usize) -> Result<Vec<u8>, ExactRecordError> {
    let expected_length = exact_length(length)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() != expected_length {
        return Err(ExactRecordRefusal::KindOrLength.into());
    }
    let mut bytes = vec![0_u8; length];
    file.read_exact(&mut bytes)?;
    require_no_trailing_bytes(file)?;
    Ok(bytes)
}

/// Reverifies that `name` is exactly `expected` with `identity`, before and after reading.
///
/// Both the opened handle and the directory entry are checked for kind,
/// length, and identity on either side of the read, so a replaced or
/// byte-equal substituted entry refuses instead of being admitted.
pub(super) fn verify_named(
    directory: &Dir,
    name: &str,
    expected: &[u8],
    identity: EntryIdentity,
) -> Result<(), ExactRecordError> {
    let mut file = open_read(directory, name)?;
    require_metadata(&file.metadata()?, expected.len(), identity)?;
    require_metadata(&directory.symlink_metadata(name)?, expected.len(), identity)?;
    let mut observed = vec![0_u8; expected.len()];
    file.read_exact(&mut observed)?;
    if observed != expected {
        return Err(ExactRecordRefusal::Bytes.into());
    }
    require_no_trailing_bytes(&mut file)?;
    require_metadata(&file.metadata()?, expected.len(), identity)?;
    require_metadata(&directory.symlink_metadata(name)?, expected.len(), identity)
}

/// Requires that no entry named `name` remains visible.
pub(super) fn require_absent(directory: &Dir, name: &str) -> Result<(), ExactRecordError> {
    match directory.symlink_metadata(name) {
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
        Ok(_) => Err(ExactRecordRefusal::RemainedVisible.into()),
        Err(source) => Err(source.into()),
    }
}

/// Hard-links `source_name` to `destination_name`, admitting an existing target.
///
/// An `AlreadyExists` target is left untouched for the caller to verify; this
/// primitive never replaces an entry.
pub(super) fn link_without_replacement(
    source_directory: &Dir,
    source_name: &str,
    destination_directory: &Dir,
    destination_name: &str,
) -> io::Result<()> {
    match source_directory.hard_link(source_name, destination_directory, destination_name) {
        Ok(()) => Ok(()),
        Err(source) if source.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(source) => Err(source),
    }
}

fn open_read(directory: &Dir, name: &str) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No).nonblock(true);
    directory.open_with(name, &options)
}

fn exact_length(length: usize) -> Result<u64, ExactRecordError> {
    u64::try_from(length).map_err(|_source| ExactRecordRefusal::LengthOverflow.into())
}

fn require_metadata(
    metadata: &Metadata,
    expected_length: usize,
    expected_identity: EntryIdentity,
) -> Result<(), ExactRecordError> {
    let expected_length = exact_length(expected_length)?;
    if metadata.is_file()
        && metadata.len() == expected_length
        && EntryIdentity::from(metadata) == expected_identity
    {
        Ok(())
    } else {
        Err(ExactRecordRefusal::KindLengthOrIdentity.into())
    }
}

fn require_no_trailing_bytes(file: &mut File) -> Result<(), ExactRecordError> {
    let mut trailing = [0_u8; 1];
    if file.read(&mut trailing)? == 0 {
        Ok(())
    } else {
        Err(ExactRecordRefusal::TrailingBytes.into())
    }
}

#[cfg(test)]
#[path = "filesystem_exact_record_tests.rs"]
mod tests;
