//! This module owns bounded stage bytes and the opened evidence behind them.

use std::io::{self, Read};

use cap_std::fs::{Dir, File};

use super::filesystem_exact_record::{
    self as exact_record, EntryIdentity, ExactRecordError, ExactRecordRefusal,
};

/// An observation keeps the inode alive until its resumed operation completes.
/// Identity is checked again by the owning stage capability before effects.
pub(super) struct StageObservation {
    bytes: Box<[u8]>,
    identity: EntryIdentity,
    _file: File,
}

impl StageObservation {
    /// Reads at most `limit` bytes, without following links or opening a FIFO.
    /// The caller includes an extra byte when its classifier needs to distinguish
    /// an exact maximum-length record from an overlong one.
    pub(super) fn read(directory: &Dir, name: &str, limit: usize) -> io::Result<Option<Self>> {
        let mut file = match exact_record::open_read(directory, name) {
            Ok(file) => file,
            Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(source) => return Err(source),
        };
        let metadata = file.metadata()?;
        if !metadata.is_file() {
            return Err(ExactRecordError::Refused(ExactRecordRefusal::KindOrLength).into_io());
        }
        let identity = EntryIdentity::from(&metadata);
        let limit = u64::try_from(limit).map_err(|_source| {
            ExactRecordError::Refused(ExactRecordRefusal::LengthOverflow).into_io()
        })?;
        let mut bytes = Vec::new();
        file.by_ref().take(limit).read_to_end(&mut bytes)?;
        Ok(Some(Self {
            bytes: bytes.into_boxed_slice(),
            identity,
            _file: file,
        }))
    }

    pub(super) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(super) const fn identity(&self) -> EntryIdentity {
        self.identity
    }
}
