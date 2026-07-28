//! Deterministically partitioned input with exact boundary byte accounting.

use std::io::{self, Read};

use crate::ScenarioError;

pub(super) struct MeteredReader<'a> {
    source: &'a [u8],
    widths: &'static [usize],
    position: usize,
    width_index: usize,
    bytes_read: u64,
}

impl<'a> MeteredReader<'a> {
    pub(super) fn new(source: &'a [u8], widths: &'static [usize]) -> Result<Self, ScenarioError> {
        if widths.is_empty() || widths.contains(&0) {
            return Err(ScenarioError::CorpusRangeUnavailable {
                target: "nonzero-input-partition",
                available: widths.len(),
            });
        }
        Ok(Self {
            source,
            widths,
            position: 0,
            width_index: 0,
            bytes_read: 0,
        })
    }

    pub(super) const fn bytes_read(&self) -> u64 {
        self.bytes_read
    }
}

impl Read for MeteredReader<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if self.position == self.source.len() || output.is_empty() {
            return Ok(0);
        }
        let width =
            self.widths.get(self.width_index).copied().ok_or_else(|| {
                io::Error::other("input partition index escaped its fixed widths")
            })?;
        let remaining = self
            .source
            .len()
            .checked_sub(self.position)
            .ok_or_else(|| io::Error::other("input partition position exceeded source length"))?;
        let accepted = width.min(remaining).min(output.len());
        let end = self
            .position
            .checked_add(accepted)
            .ok_or_else(|| io::Error::other("input partition coordinate overflowed"))?;
        let source = self
            .source
            .get(self.position..end)
            .ok_or_else(|| io::Error::other("input partition source range was unavailable"))?;
        let destination = output
            .get_mut(..accepted)
            .ok_or_else(|| io::Error::other("input partition output range was unavailable"))?;
        destination.copy_from_slice(source);
        self.position = end;
        self.bytes_read = self
            .bytes_read
            .checked_add(
                u64::try_from(accepted)
                    .map_err(|_source| io::Error::other("input read count exceeds u64"))?,
            )
            .ok_or_else(|| io::Error::other("input byte counter overflowed"))?;
        let next_width = self
            .width_index
            .checked_add(1)
            .ok_or_else(|| io::Error::other("input width index overflowed"))?;
        self.width_index = if next_width == self.widths.len() {
            0
        } else {
            next_width
        };
        Ok(accepted)
    }
}
