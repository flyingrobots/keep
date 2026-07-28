//! Allocation-free exact output byte accounting.

use std::io::{self, Write};

#[derive(Default)]
pub(super) struct CountingWriter {
    bytes_written: u64,
}

impl CountingWriter {
    pub(super) const fn bytes_written(&self) -> u64 {
        self.bytes_written
    }
}

impl Write for CountingWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let incoming = u64::try_from(buffer.len())
            .map_err(|_source| io::Error::other("output write count exceeds u64"))?;
        self.bytes_written = self
            .bytes_written
            .checked_add(incoming)
            .ok_or_else(|| io::Error::other("output byte counter overflowed"))?;
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
