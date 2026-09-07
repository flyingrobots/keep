//! This module owns joint admission of the three fixed version-two migration records.

use std::io::{self, Read};

use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::fs::{Dir, OpenOptions};

use super::store_migration::{
    FORMAT_MARKER_LENGTH, MIGRATION_INTENT_LENGTH, MIGRATION_RECEIPT_LENGTH,
};
use super::{
    AdmittedStoreFormatMarker, AdmittedStoreMigrationIntent, AdmittedStoreMigrationReceipt,
};

const MARKER_NAME: &str = "FORMAT";
const INTENT_NAME: &str = "migration.intent";
const RECEIPT_NAME: &str = "migration.receipt";

/// Root identity coordinates bound into an admitted `migration.intent`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct BoundRootIdentity {
    device: u64,
    mount: u64,
    file: u64,
}

impl BoundRootIdentity {
    pub(super) const fn new(device: u64, mount: u64, file: u64) -> Self {
        Self {
            device,
            mount,
            file,
        }
    }

    pub(super) const fn device(self) -> u64 {
        self.device
    }

    pub(super) const fn mount(self) -> u64 {
        self.mount
    }

    pub(super) const fn file(self) -> u64 {
        self.file
    }
}

/// Reads and jointly admits `FORMAT`, `migration.intent`, and `migration.receipt`.
///
/// Each record is reopened without following links, bounded to its exact
/// canonical length, and decoded. The receipt is admitted only against the
/// decoded intent and marker, so a record set that is individually
/// well-formed but mutually inconsistent refuses. Writer authority over a
/// version-two root must not be returned before this admission succeeds. The
/// intent's bound root coordinates are returned for identity comparison.
pub(super) fn admit(root: &Dir) -> io::Result<BoundRootIdentity> {
    let marker_bytes = read_exact(root, MARKER_NAME, FORMAT_MARKER_LENGTH)?;
    let intent_bytes = read_exact(root, INTENT_NAME, MIGRATION_INTENT_LENGTH)?;
    let receipt_bytes = read_exact(root, RECEIPT_NAME, MIGRATION_RECEIPT_LENGTH)?;
    let marker = AdmittedStoreFormatMarker::decode(&marker_bytes)
        .map_err(|source| invalid_data(MARKER_NAME, &source))?;
    let intent = AdmittedStoreMigrationIntent::decode(&intent_bytes)
        .map_err(|source| invalid_data(INTENT_NAME, &source))?;
    let _receipt = AdmittedStoreMigrationReceipt::decode(&receipt_bytes, &intent, &marker)
        .map_err(|source| invalid_data(RECEIPT_NAME, &source))?;
    Ok(BoundRootIdentity::new(
        intent.root_device_identity().get(),
        intent.root_mount_identity().get(),
        intent.root_file_identity().get(),
    ))
}

fn read_exact(root: &Dir, name: &str, length: usize) -> io::Result<Vec<u8>> {
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No).nonblock(true);
    let mut file = root.open_with(name, &options)?;
    let expected_length = u64::try_from(length)
        .map_err(|_source| invalid_data(name, &"record length exceeded u64"))?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() != expected_length {
        return Err(invalid_data(name, &"record kind or length disagreed"));
    }
    let mut bytes = vec![0_u8; length];
    file.read_exact(&mut bytes)?;
    let mut trailing = [0_u8; 1];
    if file.read(&mut trailing)? != 0 {
        return Err(invalid_data(name, &"record carried trailing bytes"));
    }
    Ok(bytes)
}

fn invalid_data(name: &str, source: &dyn std::fmt::Display) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("version-two record {name} refused admission: {source}"),
    )
}
