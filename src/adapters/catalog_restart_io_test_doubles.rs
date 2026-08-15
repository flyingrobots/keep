//! This module owns bounded streaming doubles for catalog restart I/O laws.

use std::io::{self, Read, Write};

use super::{CatalogRestartArtifact, CatalogRestartError};

pub(super) struct SyntheticStreamingReader {
    remaining: u64,
    emit_stride: u64,
}

impl SyntheticStreamingReader {
    pub(super) fn new(total: u64, emit_stride: u64) -> Self {
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

        let Ok(sink_capacity) = u64::try_from(sink.len()) else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "sink capacity exceeds supported range",
            ));
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

        let read_window = sink
            .get_mut(..emitted)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "read window overflow"))?;
        read_window.fill(0x5a);
        let emitted_u64 = u64::try_from(emitted)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "emit size overflow"))?;
        self.remaining = self
            .remaining
            .checked_sub(emitted_u64)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "emit underflow"))?;
        Ok(emitted)
    }
}

pub(super) struct StreamingCallbackBudget {
    observed_bytes: u64,
    total_chunks: u64,
    max_chunk: usize,
    callback_limit: usize,
    expected_total: u64,
}

impl StreamingCallbackBudget {
    pub(super) fn new(expected_total: u64, callback_limit: usize) -> Self {
        Self {
            observed_bytes: 0,
            total_chunks: 0,
            max_chunk: 0,
            callback_limit,
            expected_total,
        }
    }

    pub(super) fn consume(&mut self, chunk: &[u8]) -> Result<(), CatalogRestartError> {
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

    pub(super) fn observed_bytes(&self) -> u64 {
        self.observed_bytes
    }

    pub(super) fn max_chunk(&self) -> usize {
        self.max_chunk
    }

    pub(super) fn callback_limit(&self) -> usize {
        self.callback_limit
    }

    pub(super) fn total_chunks(&self) -> u64 {
        self.total_chunks
    }
}

pub(super) struct StreamingWriteSink {
    observed_bytes: u64,
    observed_chunks: u64,
    max_chunk: usize,
    writer_memory_limit: usize,
}

impl StreamingWriteSink {
    pub(super) fn new(writer_memory_limit: usize) -> Self {
        Self {
            observed_bytes: 0,
            observed_chunks: 0,
            max_chunk: 0,
            writer_memory_limit,
        }
    }

    pub(super) fn observed_bytes(&self) -> u64 {
        self.observed_bytes
    }

    pub(super) fn total_chunks(&self) -> u64 {
        self.observed_chunks
    }

    pub(super) fn max_chunk(&self) -> usize {
        self.max_chunk
    }
}

impl Write for StreamingWriteSink {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.observed_chunks = self
            .observed_chunks
            .checked_add(1)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "chunk count overflow"))?;
        let observed = u64::try_from(bytes.len())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "chunk length overflow"))?;
        self.observed_bytes = self
            .observed_bytes
            .checked_add(observed)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "write count overflow"))?;
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
