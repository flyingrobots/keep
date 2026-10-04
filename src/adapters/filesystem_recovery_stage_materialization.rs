//! This module owns exact writable recovery-stage materialization.

use std::io::{self, Read, Seek, SeekFrom};

use cap_std::fs::File;

use super::{FilesystemRecoveryStageError, RecoveryStage, RecoveryStageLength};

pub(super) fn read_and_position(
    file: &mut File,
    stage: RecoveryStage,
    length: RecoveryStageLength,
) -> Result<Box<[u8]>, FilesystemRecoveryStageError> {
    let mut encoded = allocate(stage, length)?;
    file.seek(SeekFrom::Start(0))
        .map_err(|source| FilesystemRecoveryStageError::Position { stage, source })?;
    read_exact(file, stage, length, &mut encoded)?;
    verify_position(file, stage, length)?;
    Ok(encoded.into_boxed_slice())
}

fn allocate(
    stage: RecoveryStage,
    length: RecoveryStageLength,
) -> Result<Vec<u8>, FilesystemRecoveryStageError> {
    let host_length = usize::try_from(length.get()).map_err(|_source| {
        FilesystemRecoveryStageError::MaterializeAddressSpace {
            stage,
            byte_count: length.get(),
        }
    })?;
    let mut encoded = Vec::new();
    encoded.try_reserve_exact(host_length).map_err(|source| {
        FilesystemRecoveryStageError::MaterializeAllocation {
            stage,
            byte_count: length.get(),
            source,
        }
    })?;
    Ok(encoded)
}

fn read_exact(
    file: &mut File,
    stage: RecoveryStage,
    length: RecoveryStageLength,
    encoded: &mut Vec<u8>,
) -> Result<(), FilesystemRecoveryStageError> {
    let expected = length.get();
    let observed = file
        .by_ref()
        .take(expected)
        .read_to_end(encoded)
        .map_err(|source| FilesystemRecoveryStageError::Materialize {
            stage,
            expected: length,
            source,
        })?;
    let observed = match u64::try_from(observed) {
        Ok(observed) => observed,
        Err(_source) => {
            return Err(FilesystemRecoveryStageError::LengthChanged {
                stage,
                expected: length,
                observed: observed_length(file, stage, length)?,
            });
        }
    };
    if observed < expected {
        return Err(FilesystemRecoveryStageError::Materialize {
            stage,
            expected: length,
            source: io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "recovery stage ended before the expected boundary",
            ),
        });
    }
    reject_trailing_bytes(file, stage, length)?;
    Ok(())
}

fn reject_trailing_bytes(
    file: &mut File,
    stage: RecoveryStage,
    expected: RecoveryStageLength,
) -> Result<(), FilesystemRecoveryStageError> {
    let mut trailing = [0_u8; 1];
    loop {
        match file.read(&mut trailing) {
            Ok(0) => return Ok(()),
            Ok(_trailing_bytes) => {
                return Err(FilesystemRecoveryStageError::LengthChanged {
                    stage,
                    expected,
                    observed: observed_length(file, stage, expected)?,
                });
            }
            Err(source) if source.kind() == io::ErrorKind::Interrupted => {}
            Err(source) => {
                return Err(FilesystemRecoveryStageError::Materialize {
                    stage,
                    expected,
                    source,
                });
            }
        }
    }
}

/// Reports the stage's actual length when its bytes disagree with `expected`.
///
/// `LengthChanged.observed` names what the filesystem holds now, not a guess
/// derived from the read that detected the disagreement.
fn observed_length(
    file: &File,
    stage: RecoveryStage,
    expected: RecoveryStageLength,
) -> Result<u64, FilesystemRecoveryStageError> {
    file.metadata()
        .map(|metadata| metadata.len())
        .map_err(|source| FilesystemRecoveryStageError::Materialize {
            stage,
            expected,
            source,
        })
}

pub(super) fn verify_position(
    file: &mut File,
    stage: RecoveryStage,
    expected: RecoveryStageLength,
) -> Result<(), FilesystemRecoveryStageError> {
    let observed = file
        .stream_position()
        .map_err(|source| FilesystemRecoveryStageError::Position { stage, source })?;
    if observed == expected.get() {
        Ok(())
    } else {
        Err(FilesystemRecoveryStageError::PositionMismatch {
            stage,
            expected,
            observed,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::io;
    use std::path::Path;

    use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
    use cap_std::fs::OpenOptions;
    use cap_std::{ambient_authority, fs::Dir};

    use super::*;
    use crate::adapters::filesystem_test_sandbox::TestDirectory;

    #[test]
    fn read_exact_reads_expected_bytes_without_trailing() -> Result<(), Box<dyn Error>> {
        let sandbox = TestDirectory::create("stage-materialization-exact")?;
        let path = sandbox.path().join("stage.bin");
        fs::write(&path, b"abcdef")?;
        let mut file = open_for_tests(&path)?;
        let encoded = super::read_and_position(
            &mut file,
            RecoveryStage::Segment,
            RecoveryStageLength::from_validated(6),
        )?;
        assert_eq!(encoded.as_ref(), b"abcdef");
        drop(file);
        sandbox.remove()?;
        Ok(())
    }

    #[test]
    fn read_and_position_rejects_short_stage() -> Result<(), Box<dyn Error>> {
        let sandbox = TestDirectory::create("stage-materialization-short")?;
        let path = sandbox.path().join("stage.bin");
        fs::write(&path, b"abc")?;
        let mut file = open_for_tests(&path)?;
        let Err(error) = super::read_and_position(
            &mut file,
            RecoveryStage::Segment,
            RecoveryStageLength::from_validated(5),
        ) else {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidData,
                "short stage materialization should have failed",
            )));
        };

        assert!(matches!(
            error,
            FilesystemRecoveryStageError::Materialize {
                stage: RecoveryStage::Segment,
                expected,
                source,
            } if expected.get() == 5 && source.kind() == std::io::ErrorKind::UnexpectedEof
        ));
        drop(file);
        sandbox.remove()?;
        Ok(())
    }

    #[test]
    fn read_and_position_rejects_trailing_bytes() -> Result<(), Box<dyn Error>> {
        let sandbox = TestDirectory::create("stage-materialization-trailing")?;
        let path = sandbox.path().join("stage.bin");
        fs::write(&path, b"abcdef")?;
        let mut file = open_for_tests(&path)?;
        let Err(error) = super::read_and_position(
            &mut file,
            RecoveryStage::Segment,
            RecoveryStageLength::from_validated(3),
        ) else {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidData,
                "trailing stage materialization should have failed",
            )));
        };

        assert!(matches!(
            error,
            FilesystemRecoveryStageError::LengthChanged {
                stage: RecoveryStage::Segment,
                expected,
                observed: 6,
            } if expected.get() == 3
        ));
        drop(file);
        sandbox.remove()?;
        Ok(())
    }

    fn open_for_tests(path: &Path) -> Result<File, Box<dyn Error>> {
        let directory_path = path.parent().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "stage fixture path does not have a parent directory",
            )
        })?;
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "stage fixture path has no UTF-8 file name",
                )
            })?;
        let directory = Dir::open_ambient_dir(directory_path, ambient_authority())?;
        let mut options = OpenOptions::new();
        options.read(true).follow(FollowSymlinks::No).nonblock(true);
        let file = directory.open_with(file_name, &options)?;
        Ok(file)
    }
}
