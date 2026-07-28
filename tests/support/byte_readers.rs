//! Deterministic short-read and failure sources.

use std::io::{self, ErrorKind, Read};

/// Reader that cycles through exact short-read widths and injects one retry.
pub struct PartitionReader<'a> {
    remaining: &'a [u8],
    widths: std::iter::Cycle<std::slice::Iter<'a, usize>>,
    interrupt_next: bool,
}

impl<'a> PartitionReader<'a> {
    /// Constructs a source over `bytes` with a repeated nonempty width plan.
    ///
    /// # Errors
    ///
    /// Returns [`ErrorKind::InvalidInput`] for an empty plan or a zero width.
    pub fn new(bytes: &'a [u8], widths: &'a [usize]) -> io::Result<Self> {
        super::validate_partition_widths(widths)?;
        Ok(Self {
            remaining: bytes,
            widths: widths.iter().cycle(),
            interrupt_next: true,
        })
    }
}

impl Read for PartitionReader<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.interrupt_next {
            self.interrupt_next = false;
            return Err(io::Error::new(
                ErrorKind::Interrupted,
                "fixture interruption",
            ));
        }
        if self.remaining.is_empty() {
            return Ok(0);
        }
        let width = self
            .widths
            .next()
            .copied()
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "empty partition plan"))?;
        let count = width.min(buffer.len()).min(self.remaining.len());
        let (source, remainder) = self
            .remaining
            .split_at_checked(count)
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "invalid fixture partition"))?;
        let destination = buffer
            .get_mut(..count)
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "invalid read buffer"))?;
        destination.copy_from_slice(source);
        self.remaining = remainder;
        Ok(count)
    }
}

/// Reader that deterministically returns a non-interruption source failure.
pub struct FailingReader;

impl Read for FailingReader {
    fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::new(
            ErrorKind::PermissionDenied,
            "fixture refusal",
        ))
    }
}

/// Broken reader that reports one byte beyond the supplied buffer.
pub struct LyingReader;

impl Read for LyingReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        buffer
            .len()
            .checked_add(1)
            .ok_or_else(|| io::Error::other("fixture count overflow"))
    }
}
