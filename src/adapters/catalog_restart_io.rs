//! This module owns exact capability-relative restart artifact reads.

use std::io::{self, Read};
use std::path::Path;

use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, File, OpenOptions};

use super::{CatalogRestartArtifact, CatalogRestartError, CatalogRestartPhase};

pub(super) fn open_root(root: &Path) -> Result<Dir, CatalogRestartError> {
    Dir::open_ambient_dir(root, ambient_authority())
        .map_err(|source| CatalogRestartError::io(CatalogRestartPhase::OpenRoot, source))
}

pub(super) fn open_regular(
    directory: &Dir,
    name: &str,
    artifact: CatalogRestartArtifact,
    phase: CatalogRestartPhase,
) -> Result<(File, u64), CatalogRestartError> {
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No).nonblock(true);
    let file = directory
        .open_with(name, &options)
        .map_err(|source| CatalogRestartError::io(phase, source))?;
    let metadata = file
        .metadata()
        .map_err(|source| CatalogRestartError::io(phase, source))?;
    if !metadata.is_file() {
        return Err(CatalogRestartError::NotRegular { artifact });
    }
    Ok((file, metadata.len()))
}

/// Reads exactly `expected` bytes into one pre-reserved buffer and refuses any trailing byte.
///
/// The complete artifact is reserved before the first read, so an artifact
/// that cannot fit in process memory refuses with
/// [`CatalogRestartError::Allocation`] and never allocates. A short source
/// refuses with the `phase` I/O error, and a longer source refuses with the
/// exact observed length.
pub(super) fn read_exact<R: Read>(
    mut source: R,
    artifact: CatalogRestartArtifact,
    phase: CatalogRestartPhase,
    expected: u64,
) -> Result<Vec<u8>, CatalogRestartError> {
    let host_length =
        usize::try_from(expected).map_err(|_source| CatalogRestartError::Allocation {
            artifact,
            byte_count: expected,
            source: None,
        })?;
    let mut encoded = Vec::new();
    encoded
        .try_reserve_exact(host_length)
        .map_err(|source| CatalogRestartError::Allocation {
            artifact,
            byte_count: expected,
            source: Some(source),
        })?;
    encoded.resize(host_length, 0);
    source
        .read_exact(&mut encoded)
        .map_err(|source| CatalogRestartError::io(phase, source))?;
    reject_trailing_bytes(&mut source, artifact, phase, expected)?;
    Ok(encoded)
}

fn reject_trailing_bytes<R: Read>(
    source: &mut R,
    artifact: CatalogRestartArtifact,
    phase: CatalogRestartPhase,
    expected: u64,
) -> Result<(), CatalogRestartError> {
    let mut trailing = [0_u8; 1];
    loop {
        match source.read(&mut trailing) {
            Ok(0) => return Ok(()),
            Ok(observed) => {
                let increment = u64::try_from(observed).map_err(|_source| {
                    CatalogRestartError::LengthArithmetic { artifact, expected }
                })?;
                let observed = expected
                    .checked_add(increment)
                    .ok_or(CatalogRestartError::LengthArithmetic { artifact, expected })?;
                return Err(CatalogRestartError::Length {
                    artifact,
                    minimum: expected,
                    maximum: expected,
                    observed,
                });
            }
            Err(source) if source.kind() == io::ErrorKind::Interrupted => {}
            Err(source) => return Err(CatalogRestartError::io(phase, source)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io::{self, Cursor, ErrorKind};

    use super::*;

    #[test]
    fn read_exact_returns_exact_bytes() -> Result<(), Box<dyn Error>> {
        let source = Cursor::new(b"abcdefg".to_vec());

        let encoded = read_exact(
            source,
            CatalogRestartArtifact::Head,
            CatalogRestartPhase::ReadCatalog,
            7,
        )?;

        assert_eq!(encoded, b"abcdefg");
        Ok(())
    }

    #[test]
    fn read_exact_rejects_short_artifacts() -> Result<(), Box<dyn Error>> {
        let source = Cursor::new(vec![b'a', b'b']);

        let result = read_exact(
            source,
            CatalogRestartArtifact::Head,
            CatalogRestartPhase::ReadCatalog,
            4,
        );

        let Err(error) = result else {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidData,
                "short artifact should have been rejected",
            )));
        };
        assert!(matches!(
            error,
            CatalogRestartError::Io {
                phase: CatalogRestartPhase::ReadCatalog,
                ref source,
            } if source.kind() == ErrorKind::UnexpectedEof
        ));
        Ok(())
    }

    #[test]
    fn read_exact_rejects_trailing_bytes() -> Result<(), Box<dyn Error>> {
        let source = Cursor::new(vec![b'a', b'b', b'c']);

        let result = read_exact(
            source,
            CatalogRestartArtifact::Head,
            CatalogRestartPhase::ReadCatalog,
            2,
        );

        let Err(error) = result else {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidData,
                "trailing bytes should have been rejected",
            )));
        };
        let expected = 2_u64;
        assert!(matches!(
            error,
            CatalogRestartError::Length {
                artifact: CatalogRestartArtifact::Head,
                minimum,
                maximum,
                observed: 3
            } if minimum == expected && maximum == expected
        ));
        Ok(())
    }
}
