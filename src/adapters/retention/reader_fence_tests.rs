//! This module owns reader-fence identity and length refusal during acquisition.

use std::error::Error;
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use super::ReaderFence;
use crate::adapters::retention::filesystem_retention_test_fixture::migrated_store;
use crate::adapters::{FilesystemEntryIdentity, ReaderFenceRefusal};
use cap_std::fs::Dir;

fn identity(path: &Path) -> io::Result<FilesystemEntryIdentity> {
    let metadata = fs::symlink_metadata(path)?;
    Ok(FilesystemEntryIdentity::new(metadata.dev(), metadata.ino()))
}

fn fence_refusal(error: Option<&io::Error>) -> Option<(io::ErrorKind, ReaderFenceRefusal)> {
    let error = error?;
    let refusal = error
        .get_ref()
        .and_then(|source| source.downcast_ref::<ReaderFenceRefusal>())
        .copied()?;
    Some((error.kind(), refusal))
}

// Size: medium (owned filesystem). Oracle: a fence must lock the same inode
// its directory entry names after acquisition, even when both files are empty.
// Delete when reader fencing is removed or stronger scheduling evidence subsumes this law.
#[test]
fn replacing_the_reader_lock_during_acquisition_refuses_the_fence() -> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("reader-fence-replaced-during-acquisition")?;
    let root = Dir::open_ambient_dir(sandbox.path(), cap_std::ambient_authority())?;
    let path = sandbox.path().join("reader.lock");
    let displaced = sandbox.path().join("reader.displaced");
    let expected = identity(&path)?;
    let mut observed = None;

    let error = ReaderFence::acquire_with(&root, || {
        fs::rename(&path, &displaced)?;
        let _replacement = fs::File::create_new(&path)?;
        observed = Some(identity(&path)?);
        Ok(())
    })
    .err();

    let observed = observed.ok_or("replacement checkpoint did not run")?;
    assert_eq!(
        fence_refusal(error.as_ref()),
        Some((
            io::ErrorKind::InvalidData,
            ReaderFenceRefusal::Identity { expected, observed }
        )),
        "a fence acquired on the displaced inode must refuse with both identities: {error:?}"
    );
    Ok(())
}

// Size: medium (owned filesystem). Oracle: the persistent fence is empty by protocol.
// Delete when reader fencing is removed.
#[test]
fn a_nonempty_reader_lock_refuses_with_both_observed_lengths() -> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("reader-fence-nonempty")?;
    fs::write(sandbox.path().join("reader.lock"), b"evidence")?;
    let root = Dir::open_ambient_dir(sandbox.path(), cap_std::ambient_authority())?;

    let error = ReaderFence::acquire(&root).err();

    assert_eq!(
        fence_refusal(error.as_ref()),
        Some((
            io::ErrorKind::InvalidData,
            ReaderFenceRefusal::Length {
                observed_handle: 8,
                observed_entry: 8
            }
        )),
        "a nonempty fence must refuse with its observed lengths: {error:?}"
    );
    Ok(())
}
