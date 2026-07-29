//! Capability-relative filesystem recovery-stage evidence laws.

use std::error::Error;
use std::fs;

use super::{
    FilesystemRecoveryInventoryReader, FilesystemRecoveryStageError, RecoveryStage,
    RecoveryStageMetadataError, filesystem_test_sandbox::TestDirectory,
};

#[test]
fn pinned_stage_is_fingerprinted_without_mutation() -> Result<(), Box<dyn Error>> {
    let fixture = StageFixture::new("recovery-stage-fingerprint")?;
    let cases: [(RecoveryStage, &[u8]); 3] = [
        (RecoveryStage::Segment, b"retained segment evidence"),
        (RecoveryStage::Catalog, b"retained catalog evidence"),
        (RecoveryStage::NextHead, b"retained head evidence"),
    ];
    for (stage, bytes) in cases {
        fs::write(fixture.stage_path(stage), bytes)?;
    }
    let reader = fixture.reader()?;

    for (stage, bytes) in cases {
        let evidence = reader.fingerprint_stage(stage)?;
        assert_eq!(evidence.stage(), stage);
        assert_eq!(evidence.length().get(), u64::try_from(bytes.len())?);
        assert_eq!(fs::read(fixture.stage_path(stage))?, bytes);
    }
    drop(reader);
    fixture.remove()?;
    Ok(())
}

#[test]
fn fixed_stage_symbolic_link_is_never_followed() -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::symlink;

    let fixture = StageFixture::new("recovery-stage-symlink")?;
    let target = fixture.root().join("outside");
    fs::write(&target, b"outside bytes")?;
    symlink(&target, fixture.segment_path())?;
    let reader = fixture.reader()?;

    let Err(error) = reader.fingerprint_stage(RecoveryStage::Segment) else {
        return Err("symbolic stage was followed".into());
    };

    assert!(matches!(
        error,
        FilesystemRecoveryStageError::Open {
            stage: RecoveryStage::Segment,
            ..
        }
    ));
    drop(reader);
    fixture.remove()?;
    Ok(())
}

#[test]
fn non_regular_stage_is_refused_before_fingerprinting() -> Result<(), Box<dyn Error>> {
    let fixture = StageFixture::new("recovery-stage-non-regular")?;
    fs::create_dir(fixture.segment_path())?;
    let reader = fixture.reader()?;

    let Err(error) = reader.fingerprint_stage(RecoveryStage::Segment) else {
        return Err("directory stage was admitted".into());
    };

    assert!(matches!(
        error,
        FilesystemRecoveryStageError::NonRegular {
            stage: RecoveryStage::Segment,
        }
    ));
    drop(reader);
    fixture.remove()?;
    Ok(())
}

#[test]
fn oversized_sparse_stage_refuses_from_metadata() -> Result<(), Box<dyn Error>> {
    let fixture = StageFixture::new("recovery-stage-oversized")?;
    let file = fs::File::create(fixture.segment_path())?;
    let observed = RecoveryStage::Segment
        .maximum_length()
        .checked_add(1)
        .ok_or("test segment maximum overflow")?;
    file.set_len(observed)?;
    let reader = fixture.reader()?;

    let Err(error) = reader.fingerprint_stage(RecoveryStage::Segment) else {
        return Err("oversized sparse stage was admitted".into());
    };

    assert!(matches!(
        error,
        FilesystemRecoveryStageError::MetadataAdmission {
            stage: RecoveryStage::Segment,
            source: RecoveryStageMetadataError::Oversized {
                observed: actual,
                ..
            },
        } if actual == observed
    ));
    drop(reader);
    fixture.remove()?;
    Ok(())
}

#[test]
fn replaced_stage_entry_refuses_the_opened_evidence() -> Result<(), Box<dyn Error>> {
    let fixture = StageFixture::new("recovery-stage-replaced")?;
    fs::write(fixture.segment_path(), b"original")?;
    let replacement_path = fixture.segment_path();
    let retained_path = fixture.root().join("retained-stage");
    let reader = fixture.reader()?;

    let mut hook_result = Ok(());
    let result = reader.fingerprint_stage_with(RecoveryStage::Segment, || {
        hook_result = fs::rename(&replacement_path, &retained_path)
            .and_then(|()| fs::write(&replacement_path, b"replacement"));
    });
    hook_result?;
    let Err(error) = result else {
        return Err("replaced stage entry was admitted".into());
    };

    assert!(matches!(
        error,
        FilesystemRecoveryStageError::Replaced {
            stage: RecoveryStage::Segment,
        }
    ));
    drop(reader);
    fixture.remove()?;
    Ok(())
}

#[test]
fn stage_length_drift_refuses_the_streamed_evidence() -> Result<(), Box<dyn Error>> {
    let fixture = StageFixture::new("recovery-stage-length-drift")?;
    fs::write(fixture.segment_path(), b"old")?;
    let path = fixture.segment_path();
    let reader = fixture.reader()?;

    let mut hook_result = Ok(());
    let result = reader.fingerprint_stage_with(RecoveryStage::Segment, || {
        hook_result = fs::write(&path, b"new length");
    });
    hook_result?;
    let Err(error) = result else {
        return Err("length-drifted stage evidence was admitted".into());
    };

    assert!(matches!(
        error,
        FilesystemRecoveryStageError::LengthChanged {
            stage: RecoveryStage::Segment,
            expected,
            observed: 10,
        } if expected.get() == 3
    ));
    drop(reader);
    fixture.remove()?;
    Ok(())
}

struct StageFixture {
    directory: TestDirectory,
}

impl StageFixture {
    fn new(name: &str) -> Result<Self, Box<dyn Error>> {
        let directory = TestDirectory::create(name)?;
        for name in ["staging", "segments", "catalogs"] {
            fs::create_dir(directory.path().join(name))?;
        }
        Ok(Self { directory })
    }

    fn root(&self) -> &std::path::Path {
        self.directory.path()
    }

    fn segment_path(&self) -> std::path::PathBuf {
        self.stage_path(RecoveryStage::Segment)
    }

    fn stage_path(&self, stage: RecoveryStage) -> std::path::PathBuf {
        match stage {
            RecoveryStage::Segment => self.root().join("staging/current.seg"),
            RecoveryStage::Catalog => self.root().join("staging/current.cat"),
            RecoveryStage::NextHead => self.root().join("head.next"),
        }
    }

    fn reader(&self) -> Result<FilesystemRecoveryInventoryReader, Box<dyn Error>> {
        Ok(FilesystemRecoveryInventoryReader::open_unchecked_for_tests(
            self.root(),
        )?)
    }

    fn remove(self) -> std::io::Result<()> {
        self.directory.remove()
    }
}
