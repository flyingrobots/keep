//! This module owns bounded filesystem recovery name scanning.

use std::io;

use cap_std::fs::Dir;

use super::FilesystemOperationRefusal as Refusal;
use super::{RecoveryEntryName, RecoveryInventoryLimit};

pub(super) fn count_entries(directory: &Dir, remaining: u64) -> io::Result<u64> {
    require_protocol_bound(remaining)?;
    let ceiling = remaining.checked_add(1).ok_or_else(|| {
        invalid_input(Refusal::InventoryCountOverflow {
            observed: remaining,
        })
    })?;
    let mut observed = 0_u64;
    for entry in directory.entries()? {
        let _entry = entry?;
        observed = observed
            .checked_add(1)
            .ok_or_else(|| invalid_input(Refusal::InventoryCountOverflow { observed }))?;
        if observed == ceiling {
            break;
        }
    }
    Ok(observed)
}

pub(super) fn read_entry_names(
    directory: &Dir,
    expected_count: u64,
) -> io::Result<Vec<RecoveryEntryName>> {
    require_protocol_bound(expected_count)?;
    let expected = usize::try_from(expected_count).map_err(|source| {
        invalid_input(Refusal::InventoryAddressSpace {
            observed: expected_count,
            source,
        })
    })?;
    let capacity = expected
        .checked_add(1)
        .ok_or_else(|| invalid_input(Refusal::InventoryCapacityOverflow { observed: expected }))?;
    let mut names = Vec::with_capacity(capacity);
    for entry in directory.entries()? {
        names.push(entry_name(&entry?)?);
        if names.len() == capacity {
            break;
        }
    }
    Ok(names)
}

fn require_protocol_bound(observed: u64) -> io::Result<()> {
    let maximum = RecoveryInventoryLimit::PROTOCOL_MAXIMUM;
    if observed > maximum {
        Err(invalid_input(Refusal::InventoryLimit { maximum, observed }))
    } else {
        Ok(())
    }
}

#[cfg(unix)]
fn entry_name(entry: &cap_std::fs::DirEntry) -> io::Result<RecoveryEntryName> {
    use std::os::unix::ffi::OsStrExt;

    RecoveryEntryName::new(entry.file_name().as_bytes().to_vec())
        .map_err(|source| io::Error::new(io::ErrorKind::InvalidData, source))
}

#[cfg(not(unix))]
fn entry_name(_entry: &cap_std::fs::DirEntry) -> io::Result<RecoveryEntryName> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        Refusal::RawNamesUnsupported,
    ))
}

fn invalid_input(refusal: Refusal) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, refusal)
}
