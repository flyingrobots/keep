//! Pinned namespace laws for filesystem next-head finalization.

use std::error::Error;
use std::fs;

use super::super::{
    FilesystemRecoveryStageError, RecoveryNextHeadFinalizationError,
    RecoveryNextHeadFinalizationStorageError, execute_recovery_next_head_finalization,
};
use super::fixture::FinalizationFixture;

#[test]
fn replaced_catalog_namespace_refuses_candidate_loading() -> Result<(), Box<dyn Error>> {
    let fixture = FinalizationFixture::new("filesystem-next-head-catalog-replaced")?;
    let request = fixture.install_generation_one_candidate()?;
    let mut finalizer = fixture.finalizer()?;
    let catalog_directory = fixture.root().join("catalogs");
    let pinned_directory = fixture.root().join("catalogs.pinned");
    fs::rename(&catalog_directory, &pinned_directory)?;
    fs::create_dir(&catalog_directory)?;
    for entry in fs::read_dir(&pinned_directory)? {
        let entry = entry?;
        fs::copy(entry.path(), catalog_directory.join(entry.file_name()))?;
    }

    let error = execute_recovery_next_head_finalization(&mut finalizer, request)
        .err()
        .ok_or("candidate used a replaced catalog namespace")?;

    let RecoveryNextHeadFinalizationError::Verify { source, .. } = error else {
        return Err("namespace refusal lost the verification phase".into());
    };
    let RecoveryNextHeadFinalizationStorageError::Stage { source } = source.as_ref() else {
        return Err("namespace refusal lost the stage boundary".into());
    };
    assert!(matches!(
        source.as_ref(),
        FilesystemRecoveryStageError::Namespace { .. }
    ));
    assert!(!fixture.head_path().exists());
    drop(finalizer);
    fixture.remove()?;
    Ok(())
}
