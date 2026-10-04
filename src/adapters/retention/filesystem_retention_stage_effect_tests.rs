//! This module owns retained evidence and effects at the shared stage I/O contract.

use super::{
    Boundary, Durability, Effect, FilesystemRetentionStage as Stage, RetentionStorageError,
    StageReplacement,
};
use crate::adapters::filesystem_test_sandbox::TestDirectory;
use crate::adapters::retention::RetentionRecordRefusal;
use cap_std::{ambient_authority, fs::Dir};
use std::{
    error::Error,
    fs,
    io::{self, Write},
};

// Size: medium (owned filesystem). Oracle: creation is a known namespace effect
// even if a later identity/write/flush operation fails; directory durability is unknown.
// Injected I/O failures are not kernel ENOSPC or physical power-loss evidence.
// Delete only if stronger shared-capability failure laws subsume these outcomes.
#[test]
fn identity_failure_preserves_created_stage() -> Result<(), Box<dyn Error>> {
    require_creation_failure(Boundary::StageIdentity, b"")
}

// Size: medium. Oracle/deletion: shared creation contract above.
#[test]
fn write_failure_preserves_partial_stage() -> Result<(), Box<dyn Error>> {
    require_creation_failure(Boundary::StageWrite, b"ab")
}

// Size: medium. Oracle/deletion: shared creation contract above.
#[test]
fn flush_failure_preserves_written_stage() -> Result<(), Box<dyn Error>> {
    require_creation_failure(Boundary::StageFlush, b"abcd")
}

fn require_creation_failure(boundary: Boundary, retained: &[u8]) -> Result<(), Box<dyn Error>> {
        let sandbox = TestDirectory::create(&format!("stage-create-{boundary:?}"))?;
        let dir = Dir::open_ambient_dir(sandbox.path(), ambient_authority())?;
        let result = Stage::create_with(&dir, "stage", b"abcd", |point, file| {
            if point != boundary {
                return Ok(());
            }
            if point == Boundary::StageWrite {
                file.write_all(b"ab")?;
            }
            Err(io::Error::from_raw_os_error(28))
        });
        let error = match result {
            Err(error) => error,
            Ok(_) => return Err("failure was swallowed".into()),
        };
        assert_eq!(
            fs::read(sandbox.path().join("stage"))?,
            retained,
            "retained bytes at {boundary:?}"
        );
        require_progress(&error, boundary, Some(Effect::StageCreated))?;
        match error {
            RetentionStorageError::Operation { source, .. } => match *source {
                RetentionStorageError::Io { source } => {
                    assert_eq!(source.raw_os_error(), Some(28), "original I/O cause")
                }
                other => return Err(format!("wrong cause: {other:?}").into()),
            },
            other => return Err(format!("missing progress: {other:?}").into()),
        }
    Ok(())
}

// Size: medium. Oracle: exclusive-create refusal preserves the existing pathname
// and reports no newly created stage. Delete if a stronger equivalent law replaces it.
#[test]
fn an_existing_stage_is_preserved_on_exclusive_creation_refusal() -> Result<(), Box<dyn Error>> {
    let sandbox = TestDirectory::create("stage-exclusive-refusal")?;
    let dir = Dir::open_ambient_dir(sandbox.path(), ambient_authority())?;
    fs::write(sandbox.path().join("stage"), b"original")?;
    let error = match Stage::create(&dir, "stage", b"replacement") {
        Err(error) => error,
        Ok(_) => return Err("exclusive creation replaced evidence".into()),
    };
    assert_eq!(fs::read(sandbox.path().join("stage"))?, b"original");
    require_progress(&error, Boundary::StageCreation, None)?;
    let source = Error::source(&error).and_then(|s| s.downcast_ref::<RetentionStorageError>());
    assert!(matches!(source, Some(RetentionStorageError::Io { source }) if source.kind() == io::ErrorKind::AlreadyExists), "original exclusive-create refusal");
    Ok(())
}

// Size: medium. Oracle: the failed verification identifies the actual replaced
// record; a successful rename is not rolled back. Mutation after rename is a
// deterministic fault, not isolation against raw concurrent writers.
// Delete only when stronger head and receipt laws cover this boundary.
#[test]
fn replacement_failure_identifies_the_published_record_that_changed() -> Result<(), Box<dyn Error>> {
    for (purpose, name, boundary, effect) in [
        (StageReplacement::Head, "HEAD", Boundary::HeadVerification, Effect::HeadReplaced),
        (StageReplacement::Receipt, "receipt", Boundary::ReceiptVerification, Effect::ReceiptReplaced),
    ] {
        let sandbox = TestDirectory::create(&format!("stage-replace-{name}"))?;
        let dir = Dir::open_ambient_dir(sandbox.path(), ambient_authority())?;
        fs::write(sandbox.path().join(name), b"old!")?;
        let stage = Stage::create(&dir, "stage", b"next")?;
        let result = stage.replace_with(&dir, name, purpose, || fs::write(sandbox.path().join(name), b"bad!"));
        let error = match result { Err(error) => error, Ok(()) => return Err("post-rename corruption must refuse".into()) };
        require_progress(&error, boundary, Some(effect))?;
        assert_eq!(fs::read(sandbox.path().join(name))?, b"bad!", "replacement was not rolled back");
        assert_eq!(fs::metadata(sandbox.path().join("stage")).err().map(|e| e.kind()), Some(io::ErrorKind::NotFound), "source was consumed by rename");
        assert!(matches!(error, RetentionStorageError::Operation { source, .. } if matches!(*source, RetentionStorageError::Refused { source: RetentionRecordRefusal::Bytes })), "exact byte-refusal cause");
    }
    Ok(())
}

fn require_progress(error: &RetentionStorageError, boundary: Boundary, effect: Option<Effect>) -> Result<(), Box<dyn Error>> {
    let progress = error.progress().ok_or("known filesystem outcome lacks progress")?;
    assert_eq!(progress.boundary(), boundary, "failed capability boundary");
    let actual: Vec<_> = progress.known_effects().iter().map(|e| (e.effect(), e.durability())).collect();
    let expected: Vec<_> = effect.into_iter().map(|e| (e, Durability::Unconfirmed)).collect();
    assert_eq!(actual, expected, "known effects and directory durability");
    assert_eq!(progress.uncertain_effect(), None, "observed result must not become uncertain");
    Ok(())
}
