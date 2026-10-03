//! Restart-stable root identity remains binding even when every byte agrees.

use super::filesystem_migration_restart_test_fixture::{prefix, refusal};
use super::{StoreMigrationPhase, StoreMigrationRecoveryAmbiguity, StoreMigrationRecoveryError};
use crate::adapters::filesystem_test_sandbox::TestDirectory;
use std::{error::Error, fs, path::Path};

// Size: medium. Oracle: persisted migration intent binds the original root inode;
// copying the entire store must not authorize continuation under a new root.
// Delete only if root binding is explicitly removed or stronger restart evidence subsumes it.
#[test]
fn a_byte_equal_store_copy_refuses_the_persisted_root_identity() -> Result<(), Box<dyn Error>> {
    let original = prefix(
        "restart-original-root",
        StoreMigrationPhase::RemoveIntentStage,
    )?;
    let copied = TestDirectory::create("restart-substituted-root")?;
    copy_directory(original.path(), copied.path())?;
    let error = refusal(copied.path())?;
    assert!(
        matches!(
            error.downcast_ref::<StoreMigrationRecoveryError>(),
            Some(StoreMigrationRecoveryError::Ambiguity {
                source: StoreMigrationRecoveryAmbiguity::IntentDiffers
            })
        ),
        "copied root must not inherit migration authority: {error:?}"
    );
    copied.remove()?;
    original.remove()?;
    Ok(())
}

fn copy_directory(source: &Path, destination: &Path) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            fs::create_dir(&target)?;
            copy_directory(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
