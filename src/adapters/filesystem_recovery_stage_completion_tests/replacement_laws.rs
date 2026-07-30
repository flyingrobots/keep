//! Filesystem complete-stage replacement refusal laws.

use std::error::Error;
use std::fs;

use super::super::{FilesystemRecoveryStageError, RecoveryStage};
use super::fixture::{CompletionFixture, catalog_bytes, request, segment_bytes};
use super::refusal_laws::filesystem_storage_source;

#[test]
fn replacement_after_stage_open_is_preserved_and_refused() -> Result<(), Box<dyn Error>> {
    let fixture = CompletionFixture::new("filesystem-stage-completion-stage-replaced")?;
    let bytes = segment_bytes()?;
    let request = request(RecoveryStage::Segment, &bytes)?;
    let stage_path = fixture.stage_path(RecoveryStage::Segment);
    let retained_path = fixture.root().join("retained-stage");
    fs::write(&stage_path, &bytes)?;
    let completer = fixture.completer()?;
    let mut hook_result = Ok(());

    let result = completer.synchronize_stage_if_present_with(request, || {
        hook_result =
            fs::rename(&stage_path, &retained_path).and_then(|()| fs::write(&stage_path, b"new"));
    });

    hook_result?;
    let error = result.err().ok_or("replaced stage was synchronized")?;
    assert!(matches!(
        filesystem_storage_source(&error)?,
        FilesystemRecoveryStageError::Replaced {
            stage: RecoveryStage::Segment,
        }
    ));
    assert_eq!(fs::read(&stage_path)?, b"new");
    assert_eq!(fs::read(&retained_path)?, bytes);
    drop(completer);
    fixture.remove()?;
    Ok(())
}

#[test]
fn replacement_after_pool_open_is_preserved_and_refused() -> Result<(), Box<dyn Error>> {
    let fixture = CompletionFixture::new("filesystem-stage-completion-pool-replaced")?;
    let bytes = catalog_bytes()?;
    let request = request(RecoveryStage::Catalog, &bytes)?;
    let pool_path = fixture.pool_path(request);
    let retained_path = fixture.root().join("retained-pool");
    fs::write(&pool_path, &bytes)?;
    let completer = fixture.completer()?;
    let mut hook_result = Ok(());

    let result = completer.verify_pool_with(request, || {
        hook_result =
            fs::rename(&pool_path, &retained_path).and_then(|()| fs::write(&pool_path, b"new"));
    });

    hook_result?;
    let error = result.err().ok_or("replaced pool was admitted")?;
    assert!(matches!(
        filesystem_storage_source(&error)?,
        FilesystemRecoveryStageError::Replaced {
            stage: RecoveryStage::Catalog,
        }
    ));
    assert_eq!(fs::read(&pool_path)?, b"new");
    assert_eq!(fs::read(&retained_path)?, bytes);
    drop(completer);
    fixture.remove()?;
    Ok(())
}
