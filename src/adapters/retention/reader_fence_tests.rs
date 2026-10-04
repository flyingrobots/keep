//! This module owns reader-fence identity refusal during acquisition.

use std::error::Error;
use std::fs;
use std::io;

use super::ReaderFence;
use crate::adapters::retention::filesystem_retention_test_fixture::migrated_store;
use cap_std::fs::Dir;

// Size: medium (owned filesystem). Oracle: a fence must lock the same inode
// its directory entry names after acquisition, even when both files are empty.
// Delete when reader fencing is removed or stronger scheduling evidence subsumes this law.
#[test]
fn replacing_the_reader_lock_during_acquisition_refuses_the_fence() -> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("reader-fence-replaced-during-acquisition")?;
    let root = Dir::open_ambient_dir(sandbox.path(), cap_std::ambient_authority())?;
    let path = sandbox.path().join("reader.lock");

    let error = ReaderFence::acquire_with(&root, || {
        fs::remove_file(&path)?;
        let _replacement = fs::File::create_new(&path)?;
        Ok(())
    })
    .err();

    assert_eq!(
        error.as_ref().map(io::Error::kind),
        Some(io::ErrorKind::InvalidData),
        "a fence acquired on the displaced inode must refuse: {error:?}"
    );
    Ok(())
}
