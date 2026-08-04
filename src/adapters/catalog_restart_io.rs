//! This module owns exact capability-relative restart artifact reads and
//! bounded streaming writes.

use std::io::{self, Read};
use std::path::Path;

use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, File, OpenOptions};

use super::{CatalogRestartArtifact, CatalogRestartError, CatalogRestartPhase};

const CATALOG_RESTART_READ_BUFFER_LENGTH: usize = 8_192;

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
    read_exact_to(&mut file, artifact, phase, expected, |chunk| {
        encoded.extend_from_slice(chunk);
        Ok(())
    })?;
    Ok(encoded)
}

#[cfg(test)]
pub(super) fn write_exact_to<R, W>(
    source: &mut R,
    destination: &mut W,
    artifact: CatalogRestartArtifact,
    phase: CatalogRestartPhase,
    expected: u64,
) -> Result<(), CatalogRestartError>
where
    R: Read,
    W: io::Write,
{
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
                destination
                    .write_all(bytes)
                    .map_err(|source| CatalogRestartError::io(phase, source))?;
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
    Ok(())
}

pub(super) fn read_exact_to<R, F>(
    source: &mut R,
    artifact: CatalogRestartArtifact,
    phase: CatalogRestartPhase,
    expected: u64,
    mut on_chunk: F,
) -> Result<(), CatalogRestartError>
where
    R: Read,
    F: FnMut(&[u8]) -> Result<(), CatalogRestartError>,
{
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
    Ok(())
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
    use std::io;
    use std::io::{Cursor, ErrorKind, Read, Write};
    use std::mem::size_of;

    use super::*;

    #[test]
    fn read_exact_to_streams_large_virtual_file_with_small_callback_memory() {
        const TOTAL_BYTES: u64 = 64_u64 * 1024 * 1024;
        const READER_STRIDE: u64 = 2_u64 * 1024;
        const CALLBACK_BUDGET_BYTES: usize = 16 * 1024;

        let mut source = SyntheticStreamingReader::new(TOTAL_BYTES, READER_STRIDE);
        let mut budget = StreamingCallbackBudget::new(TOTAL_BYTES, CALLBACK_BUDGET_BYTES);

        let result = read_exact_to(
            &mut source,
            CatalogRestartArtifact::Head,
            CatalogRestartPhase::ReadCatalog,
            TOTAL_BYTES,
            |chunk| budget.consume(chunk),
        );

        assert!(result.is_ok(), "{result:?}");
        assert!(budget.observed_bytes() > 0);
        assert_eq!(budget.observed_bytes(), TOTAL_BYTES);
        assert!(
            budget.max_chunk() >= READER_STRIDE as usize,
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
    }

    #[test]
    fn write_exact_to_streams_large_virtual_file_with_small_writer_state() {
        const TOTAL_BYTES: u64 = 64_u64 * 1024 * 1024;
        const READER_STRIDE: u64 = 2_u64 * 1024;
        const WRITER_BUDGET_BYTES: usize = 4 * 1024;

        let mut source = SyntheticStreamingReader::new(TOTAL_BYTES, READER_STRIDE);
        let mut sink = StreamingWriteSink::new(WRITER_BUDGET_BYTES);

        let result = write_exact_to(
            &mut source,
            &mut sink,
            CatalogRestartArtifact::Head,
            CatalogRestartPhase::ReadCatalog,
            TOTAL_BYTES,
        );

        assert!(result.is_ok(), "{result:?}");
        assert_eq!(sink.observed_bytes(), TOTAL_BYTES);
        assert!(sink.total_chunks() > 0);
        assert!(sink.max_chunk() <= WRITER_BUDGET_BYTES);
        assert!(sink.max_chunk() <= CATALOG_RESTART_READ_BUFFER_LENGTH);
        assert!(sink.max_chunk() >= READER_STRIDE as usize);
        assert!(size_of::<StreamingWriteSink>() < 64);
    }

    #[test]
    fn read_exact_to_streams_chunks() {
        let mut source = Cursor::new(vec![b'a', b'b', b'c', b'd', b'e', b'f', b'g']);
        let mut observed = Vec::<Vec<u8>>::new();

        let result = read_exact_to(
            &mut source,
            CatalogRestartArtifact::Head,
            CatalogRestartPhase::ReadCatalog,
            7,
            |chunk| {
                observed.push(chunk.to_vec());
                Ok(())
            },
        );

        assert!(result.is_ok());
        assert_eq!(observed.concat(), b"abcdefg");
    }

    #[test]
    fn read_exact_to_rejects_short_artifacts() {
        let mut source = Cursor::new(vec![b'a', b'b']);
        let mut seen = 0_u8;

        let result = read_exact_to(
            &mut source,
            CatalogRestartArtifact::Head,
            CatalogRestartPhase::ReadCatalog,
            4,
            |_chunk| {
                seen = seen.checked_add(1).expect("unexpected chunk overflow");
                Ok(())
            },
        );

        let error = result.unwrap_err();
        assert_eq!(seen, 1);
        assert!(matches!(
            error,
            CatalogRestartError::Io {
                phase: CatalogRestartPhase::ReadCatalog,
                ref source,
            } if source.kind() == ErrorKind::UnexpectedEof
        ));
    }

    #[test]
    fn read_exact_to_rejects_trailing_bytes() {
        let mut source = Cursor::new(vec![b'a', b'b', b'c']);

        let result = read_exact_to(
            &mut source,
            CatalogRestartArtifact::Head,
            CatalogRestartPhase::ReadCatalog,
            2,
            |_| Ok(()),
        );

        let error = result.unwrap_err();
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
    }

    #[test]
    fn write_exact_to_rejects_short_artifacts() {
        let mut source = Cursor::new(vec![b'a', b'b']);
        let mut sink = StreamingWriteSink::new(16 * 1024);

        let result = write_exact_to(
            &mut source,
            &mut sink,
            CatalogRestartArtifact::Head,
            CatalogRestartPhase::ReadCatalog,
            4,
        );

        let error = result.unwrap_err();
        assert_eq!(sink.observed_bytes(), 2);
        assert!(matches!(
            error,
            CatalogRestartError::Io {
                phase: CatalogRestartPhase::ReadCatalog,
                ref source,
            } if source.kind() == ErrorKind::UnexpectedEof
        ));
    }

    #[test]
    fn write_exact_to_rejects_trailing_bytes() {
        let mut source = Cursor::new(vec![b'a', b'b', b'c']);
        let mut sink = StreamingWriteSink::new(16 * 1024);

        let result = write_exact_to(
            &mut source,
            &mut sink,
            CatalogRestartArtifact::Head,
            CatalogRestartPhase::ReadCatalog,
            2,
        );

        let error = result.unwrap_err();
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
    }

    struct SyntheticStreamingReader {
        remaining: u64,
        emit_stride: u64,
    }

    impl SyntheticStreamingReader {
        fn new(total: u64, emit_stride: u64) -> Self {
            Self {
                remaining: total,
                emit_stride,
            }
        }
    }

    impl Read for SyntheticStreamingReader {
        fn read(&mut self, sink: &mut [u8]) -> io::Result<usize> {
            if self.remaining == 0 {
                return Ok(0);
            }

            let sink_capacity = match u64::try_from(sink.len()) {
                Ok(capacity) => capacity,
                Err(_) => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "sink capacity exceeds supported range",
                    ));
                }
            };
            let emitted: usize = match self
                .emit_stride
                .min(self.remaining)
                .min(sink_capacity)
                .try_into()
            {
                Ok(size) => size,
                Err(_) => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "requested read size exceeds supported range",
                    ));
                }
            };

            sink[..emitted].fill(0x5a);
            self.remaining -= u64::try_from(emitted)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "emit size overflow"))?;
            Ok(emitted)
        }
    }

    struct StreamingCallbackBudget {
        observed_bytes: u64,
        total_chunks: u64,
        max_chunk: usize,
        callback_limit: usize,
        expected_total: u64,
    }

    impl StreamingCallbackBudget {
        fn new(expected_total: u64, callback_limit: usize) -> Self {
            Self {
                observed_bytes: 0,
                total_chunks: 0,
                max_chunk: 0,
                callback_limit,
                expected_total,
            }
        }

        fn consume(&mut self, chunk: &[u8]) -> Result<(), CatalogRestartError> {
            self.total_chunks =
                self.total_chunks
                    .checked_add(1)
                    .ok_or(CatalogRestartError::LengthArithmetic {
                        artifact: CatalogRestartArtifact::Head,
                        expected: self.expected_total,
                    })?;

            self.max_chunk = self.max_chunk.max(chunk.len());

            self.observed_bytes = self
                .observed_bytes
                .checked_add(u64::try_from(chunk.len()).map_err(|_source| {
                    CatalogRestartError::LengthArithmetic {
                        artifact: CatalogRestartArtifact::Head,
                        expected: self.expected_total,
                    }
                })?)
                .ok_or(CatalogRestartError::LengthArithmetic {
                    artifact: CatalogRestartArtifact::Head,
                    expected: self.expected_total,
                })?;

            Ok(())
        }

        fn observed_bytes(&self) -> u64 {
            self.observed_bytes
        }

        fn max_chunk(&self) -> usize {
            self.max_chunk
        }

        fn callback_limit(&self) -> usize {
            self.callback_limit
        }

        fn total_chunks(&self) -> u64 {
            self.total_chunks
        }
    }

    struct StreamingWriteSink {
        observed_bytes: u64,
        observed_chunks: u64,
        max_chunk: usize,
        writer_memory_limit: usize,
    }

    impl StreamingWriteSink {
        fn new(writer_memory_limit: usize) -> Self {
            Self {
                observed_bytes: 0,
                observed_chunks: 0,
                max_chunk: 0,
                writer_memory_limit,
            }
        }

        fn observed_bytes(&self) -> u64 {
            self.observed_bytes
        }

        fn total_chunks(&self) -> u64 {
            self.observed_chunks
        }

        fn max_chunk(&self) -> usize {
            self.max_chunk
        }
    }

    impl Write for StreamingWriteSink {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.observed_chunks = self.observed_chunks.checked_add(1).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "chunk count overflow")
            })?;
            let observed = u64::try_from(bytes.len()).map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "chunk length overflow")
            })?;
            self.observed_bytes = self.observed_bytes.checked_add(observed).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "write count overflow")
            })?;
            self.max_chunk = self.max_chunk.max(bytes.len());
            if bytes.len() > self.writer_memory_limit {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "sink memory budget exceeded",
                ));
            }
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
}
