//! This module owns bounded filesystem recovery name scanning.

use std::io;

use cap_std::fs::Dir;

use super::{RecoveryEntryName, RecoveryInventoryLimit};

pub(super) fn count_entries(directory: &Dir, remaining: u64) -> io::Result<u64> {
    if remaining > RecoveryInventoryLimit::PROTOCOL_MAXIMUM {
        return Err(invalid_input(
            "recovery count budget exceeds protocol maximum",
        ));
    }
    let ceiling = remaining
        .checked_add(1)
        .ok_or_else(|| invalid_input("recovery count budget cannot admit a drift witness"))?;
    let mut observed = 0_u64;
    for entry in directory.entries()? {
        let _entry = entry?;
        observed = observed
            .checked_add(1)
            .ok_or_else(|| invalid_input("recovery entry count overflowed"))?;
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
    if expected_count > RecoveryInventoryLimit::PROTOCOL_MAXIMUM {
        return Err(invalid_input(
            "recovery expected count exceeds protocol maximum",
        ));
    }
    let expected = usize::try_from(expected_count)
        .map_err(|_| invalid_input("recovery expected count does not fit the address space"))?;
    let capacity = expected
        .checked_add(1)
        .ok_or_else(|| invalid_input("recovery name capacity overflowed"))?;
    let mut names = Vec::with_capacity(capacity);
    for entry in directory.entries()? {
        names.push(entry_name(&entry?)?);
        if names.len() == capacity {
            break;
        }
    }
    Ok(names)
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
        "raw recovery entry names currently require a Unix platform",
    ))
}

fn invalid_input(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
