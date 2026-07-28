//! Deterministic short-write sink.

use std::io::{self, ErrorKind, Write};

/// Writer that cycles through exact short-write widths and injects one retry.
pub(crate) struct PartitionWriter<'a> {
    bytes: Vec<u8>,
    widths: std::iter::Cycle<std::slice::Iter<'a, usize>>,
    interrupt_next: bool,
}

/// Writer that deterministically refuses every byte.
pub(crate) struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(
            ErrorKind::PermissionDenied,
            "fixture refusal",
        ))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Broken writer that reports one byte beyond the supplied buffer.
pub(crate) struct LyingWriter;

impl Write for LyingWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        buffer
            .len()
            .checked_add(1)
            .ok_or_else(|| io::Error::other("fixture count overflow"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Writer that lawfully reports no progress for nonempty input.
pub(crate) struct ZeroWriter;

impl Write for ZeroWriter {
    fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
        Ok(0)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> PartitionWriter<'a> {
    /// Constructs an empty sink with a repeated nonempty width plan.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorKind::InvalidInput`] for an empty plan or a zero width.
    pub(crate) fn new(widths: &'a [usize]) -> io::Result<Self> {
        super::validate_partition_widths(widths)?;
        Ok(Self {
            bytes: Vec::new(),
            widths: widths.iter().cycle(),
            interrupt_next: true,
        })
    }

    /// Returns every byte accepted by the sink.
    #[must_use]
    pub(crate) fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl Write for PartitionWriter<'_> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if self.interrupt_next {
            self.interrupt_next = false;
            return Err(io::Error::new(
                ErrorKind::Interrupted,
                "fixture interruption",
            ));
        }
        let width = self
            .widths
            .next()
            .copied()
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "empty partition plan"))?;
        let count = width.min(buffer.len());
        let accepted = buffer
            .get(..count)
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "invalid write buffer"))?;
        self.bytes.extend_from_slice(accepted);
        Ok(count)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
