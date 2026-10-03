//! Filesystem restart fixtures and complete evidence witnesses for migration refusal.

use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use super::filesystem_migration_test_fixture::{maximum_policy, open_authority};
use super::migration_resumption::{MigrationRecords, execute_phase};
use super::{FilesystemStoreMigrationAuthority, StoreMigrationPhase, recover_store_migration};
use crate::adapters::filesystem_test_sandbox::TestDirectory;

pub(super) fn prefix(
    name: &str,
    last: StoreMigrationPhase,
) -> Result<TestDirectory, Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority(name)?;
    let intent = authority.observe_intent()?;
    authority.verify_current(&intent)?;
    let records = MigrationRecords::for_intent(&intent);
    for phase in StoreMigrationPhase::ALL {
        execute_phase(&mut authority, phase, &records)?;
        if phase == last {
            return Ok(sandbox);
        }
    }
    Err("requested migration prefix was not executed".into())
}

pub(super) fn refusal(root: &Path) -> Result<Box<dyn Error>, Box<dyn Error>> {
    let before = witness(root)?;
    let error = restart(root)
        .err()
        .ok_or("ambiguous migration restarted successfully")?;
    assert_eq!(
        witness(root)?,
        before,
        "recovery refusal must preserve every name, inode and byte"
    );
    Ok(error)
}

fn restart(root: &Path) -> Result<(), Box<dyn Error>> {
    let mut authority = FilesystemStoreMigrationAuthority::reopen_for_recovery_unchecked_for_tests(
        root,
        maximum_policy(),
    )?;
    let expected = authority.observe_intent()?;
    let _receipt = recover_store_migration(&mut authority, &expected)?;
    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
pub(super) enum Contents {
    Directory,
    File(Vec<u8>),
    Link(PathBuf),
}

type Witness = BTreeMap<PathBuf, (u64, u64, Contents)>;

fn witness(root: &Path) -> Result<Witness, Box<dyn Error>> {
    let mut entries = BTreeMap::new();
    visit(root, Path::new(""), &mut entries)?;
    Ok(entries)
}

fn visit(root: &Path, relative: &Path, entries: &mut Witness) -> Result<(), Box<dyn Error>> {
    let path = root.join(relative);
    let metadata = fs::symlink_metadata(&path)?;
    let contents = if metadata.is_dir() {
        for entry in fs::read_dir(&path)? {
            visit(root, &relative.join(entry?.file_name()), entries)?;
        }
        Contents::Directory
    } else if metadata.is_file() {
        Contents::File(fs::read(&path)?)
    } else if metadata.is_symlink() {
        Contents::Link(fs::read_link(&path)?)
    } else {
        return Err("unexpected evidence kind in owned fixture".into());
    };
    entries.insert(
        relative.to_path_buf(),
        (metadata.dev(), metadata.ino(), contents),
    );
    Ok(())
}
