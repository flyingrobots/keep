//! This module owns exact capability-relative restart artifact reads and
//! bounded, exact-transfer streaming.

use std::io::{self, Read, Write};
use std::path::Path;

use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, File, OpenOptions};

use super::{CatalogRestartArtifact, CatalogRestartError, CatalogRestartPhase};

const CATALOG_RESTART_READ_BUFFER_LENGTH: usize = 8_192;

#[derive(Clone, Copy, Debug)]
pub(super) struct ExactTransfer {
    artifact: CatalogRestartArtifact,
    phase: CatalogRestartPhase,
    expected: u64,
}

impl ExactTransfer {
    pub(super) const fn new(
        artifact: CatalogRestartArtifact,
        phase: CatalogRestartPhase,
        expected: u64,
    ) -> Self {
        Self {
            artifact,
            phase,
            expected,
        }
    }

    pub(super) const fn artifact(&self) -> CatalogRestartArtifact {
        self.artifact
    }

    pub(super) const fn phase(&self) -> CatalogRestartPhase {
        self.phase
    }

    pub(super) const fn expected(&self) -> u64 {
        self.expected
    }
}

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

pub(super) fn read_exact(
    mut file: File,
    artifact: CatalogRestartArtifact,
    phase: CatalogRestartPhase,
    expected: u64,
) -> Result<Vec<u8>, CatalogRestartError> {
    let transfer = ExactTransfer::new(artifact, phase, expected);
    let host_length = usize::try_from(transfer.expected()).map_err(|_source| {
        CatalogRestartError::Allocation {
            artifact: transfer.artifact(),
            byte_count: transfer.expected(),
            source: None,
        }
    })?;

    let mut encoded = Vec::new();
    encoded
        .try_reserve_exact(host_length)
        .map_err(|source| CatalogRestartError::Allocation {
            artifact: transfer.artifact(),
            byte_count: expected,
            source: Some(source),
        })?;
    let mut sink = VecWrite {
        encoded: &mut encoded,
    };
    copy_exact(&mut file, &mut sink, transfer)?;
    Ok(encoded)
}

pub(super) fn copy_exact<R, W>(
    source: &mut R,
    destination: &mut W,
    transfer: ExactTransfer,
) -> Result<u64, CatalogRestartError>
where
    R: Read,
    W: io::Write,
{
    copy_exact_to_chunks(source, transfer, |chunk| {
        destination
            .write_all(chunk)
            .map_err(|source| CatalogRestartError::io(transfer.phase(), source))
    })
}

pub(super) fn copy_exact_to_chunks<R, F>(
    source: &mut R,
    transfer: ExactTransfer,
    mut on_chunk: F,
) -> Result<u64, CatalogRestartError>
where
    R: Read,
    F: FnMut(&[u8]) -> Result<(), CatalogRestartError>,
{
    let artifact = transfer.artifact();
    let phase = transfer.phase();
    let expected = transfer.expected();

    let mut observed = 0_u64;
    let mut buffer = [0_u8; CATALOG_RESTART_READ_BUFFER_LENGTH];
    let chunk_length = u64::try_from(buffer.len())
        .map_err(|_source| CatalogRestartError::LengthArithmetic { artifact, expected })?;

    while observed < expected {
        let remaining = expected
            .checked_sub(observed)
            .ok_or(CatalogRestartError::LengthArithmetic { artifact, expected })?;
        let offered = remaining
            .min(chunk_length)
            .try_into()
            .map_err(|_source| CatalogRestartError::LengthArithmetic { artifact, expected })?;
        let read_buffer = buffer
            .get_mut(..offered)
            .ok_or(CatalogRestartError::LengthArithmetic { artifact, expected })?;
        match source.read(read_buffer) {
            Ok(0) => {
                return Err(CatalogRestartError::io(
                    phase,
                    io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "restart artifact ended before the expected boundary",
                    ),
                ));
            }
            Ok(count) => {
                let bytes = read_buffer
                    .get(..count)
                    .ok_or(CatalogRestartError::LengthArithmetic { artifact, expected })?;
                on_chunk(bytes)?;
                let increment = u64::try_from(count).map_err(|_source| {
                    CatalogRestartError::LengthArithmetic { artifact, expected }
                })?;
                observed = observed
                    .checked_add(increment)
                    .ok_or(CatalogRestartError::LengthArithmetic { artifact, expected })?;
            }
            Err(source) if source.kind() == io::ErrorKind::Interrupted => {}
            Err(source) => return Err(CatalogRestartError::io(phase, source)),
        }
    }
    reject_trailing_bytes(source, artifact, phase, expected)?;
    Ok(observed)
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

struct VecWrite<'a> {
    encoded: &'a mut Vec<u8>,
}

impl Write for VecWrite<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.encoded.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io;
    use std::io::{Cursor, ErrorKind};
    use std::mem::size_of;

    use super::super::catalog_restart_io_test_doubles::{
        StreamingCallbackBudget, StreamingWriteSink, SyntheticStreamingReader,
    };
    use super::*;

    #[test]
    fn read_exact_to_streams_large_virtual_file_with_small_callback_memory()
    -> Result<(), Box<dyn Error>> {
        const TOTAL_BYTES: u64 = 64_u64 * 1024 * 1024;
        const READER_STRIDE_BYTES: usize = 2_usize * 1024;
        const CALLBACK_BUDGET_BYTES: usize = 16 * 1024;
        let Ok(reader_stride) = u64::try_from(READER_STRIDE_BYTES) else {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidData,
                "reader stride is outside supported range",
            )));
        };

        let mut source = SyntheticStreamingReader::new(TOTAL_BYTES, reader_stride);
        let mut budget = StreamingCallbackBudget::new(TOTAL_BYTES, CALLBACK_BUDGET_BYTES);

        let _observed = copy_exact_to_chunks(
            &mut source,
            ExactTransfer::new(
                CatalogRestartArtifact::Head,
                CatalogRestartPhase::ReadCatalog,
                TOTAL_BYTES,
            ),
            |chunk| budget.consume(chunk),
        )?;

        assert!(budget.observed_bytes() > 0);
        assert_eq!(budget.observed_bytes(), TOTAL_BYTES);
        assert!(
            budget.max_chunk() >= READER_STRIDE_BYTES,
            "reader stride should be observed"
        );
        assert!(
            budget.max_chunk() <= budget.callback_limit(),
            "callback should remain in budget"
        );
        assert!(
            budget.max_chunk() <= CATALOG_RESTART_READ_BUFFER_LENGTH,
            "read buffer bounds should hold"
        );
        assert!(budget.total_chunks() > 0);
        assert!(
            size_of::<StreamingCallbackBudget>() < 128,
            "callback state should stay compact"
        );
        assert_eq!(budget.observed_bytes(), TOTAL_BYTES);
        Ok(())
    }

    #[test]
    fn copy_exact_streams_large_virtual_file_with_small_writer_state() -> Result<(), Box<dyn Error>>
    {
        const TOTAL_BYTES: u64 = 64_u64 * 1024 * 1024;
        const READER_STRIDE_BYTES: usize = 2_usize * 1024;
        const WRITER_BUDGET_BYTES: usize = 4 * 1024;
        let Ok(reader_stride) = u64::try_from(READER_STRIDE_BYTES) else {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidData,
                "reader stride is outside supported range",
            )));
        };

        let mut source = SyntheticStreamingReader::new(TOTAL_BYTES, reader_stride);
        let mut sink = StreamingWriteSink::new(WRITER_BUDGET_BYTES);

        let observed = copy_exact(
            &mut source,
            &mut sink,
            ExactTransfer::new(
                CatalogRestartArtifact::Head,
                CatalogRestartPhase::ReadCatalog,
                TOTAL_BYTES,
            ),
        )?;
        assert_eq!(observed, TOTAL_BYTES);
        assert_eq!(sink.observed_bytes(), TOTAL_BYTES);
        assert!(sink.total_chunks() > 0);
        assert!(sink.max_chunk() <= WRITER_BUDGET_BYTES);
        assert!(sink.max_chunk() <= CATALOG_RESTART_READ_BUFFER_LENGTH);
        assert!(sink.max_chunk() >= READER_STRIDE_BYTES);
        assert!(size_of::<StreamingWriteSink>() < 64);
        Ok(())
    }

    #[test]
    fn copy_exact_to_chunks_streams() -> Result<(), Box<dyn Error>> {
        let mut source = Cursor::new(vec![b'a', b'b', b'c', b'd', b'e', b'f', b'g']);
        let mut observed = Vec::<Vec<u8>>::new();

        let _observed = copy_exact_to_chunks(
            &mut source,
            ExactTransfer::new(
                CatalogRestartArtifact::Head,
                CatalogRestartPhase::ReadCatalog,
                7,
            ),
            |chunk| {
                observed.push(chunk.to_vec());
                Ok(())
            },
        )?;

        assert_eq!(observed.concat(), b"abcdefg");
        Ok(())
    }

    #[test]
    fn copy_exact_to_chunks_rejects_short_artifacts() -> Result<(), Box<dyn Error>> {
        let mut source = Cursor::new(vec![b'a', b'b']);
        let mut seen = 0_u8;

        let result = copy_exact_to_chunks(
            &mut source,
            ExactTransfer::new(
                CatalogRestartArtifact::Head,
                CatalogRestartPhase::ReadCatalog,
                4,
            ),
            |_chunk| {
                seen = match seen.checked_add(1) {
                    Some(total) => total,
                    None => {
                        return Err(CatalogRestartError::LengthArithmetic {
                            artifact: CatalogRestartArtifact::Head,
                            expected: 4,
                        });
                    }
                };
                Ok(())
            },
        );

        let Err(error) = result else {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidData,
                "short artifact should have been rejected",
            )));
        };
        assert_eq!(seen, 1);
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
    fn copy_exact_to_chunks_rejects_trailing_bytes() -> Result<(), Box<dyn Error>> {
        let mut source = Cursor::new(vec![b'a', b'b', b'c']);

        let result = copy_exact_to_chunks(
            &mut source,
            ExactTransfer::new(
                CatalogRestartArtifact::Head,
                CatalogRestartPhase::ReadCatalog,
                2,
            ),
            |_| Ok(()),
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

    #[test]
    fn copy_exact_rejects_short_artifacts() -> Result<(), Box<dyn Error>> {
        let mut source = Cursor::new(vec![b'a', b'b']);
        let mut sink = StreamingWriteSink::new(16 * 1024);

        let result = copy_exact(
            &mut source,
            &mut sink,
            ExactTransfer::new(
                CatalogRestartArtifact::Head,
                CatalogRestartPhase::ReadCatalog,
                4,
            ),
        );

        let Err(error) = result else {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidData,
                "short artifact should have been rejected",
            )));
        };
        assert_eq!(sink.observed_bytes(), 2);
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
    fn copy_exact_rejects_trailing_bytes() -> Result<(), Box<dyn Error>> {
        let mut source = Cursor::new(vec![b'a', b'b', b'c']);
        let mut sink = StreamingWriteSink::new(16 * 1024);

        let result = copy_exact(
            &mut source,
            &mut sink,
            ExactTransfer::new(
                CatalogRestartArtifact::Head,
                CatalogRestartPhase::ReadCatalog,
                2,
            ),
        );

        let Err(error) = result else {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidData,
                "trailing bytes should have been rejected",
            )));
        };
        let expected = 2_u64;
        assert_eq!(sink.observed_bytes(), 2);
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
