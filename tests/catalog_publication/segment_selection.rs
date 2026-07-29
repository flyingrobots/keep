//! Checked closed-stage publication selections for catalog tests.

use std::error::Error;
use std::io::{self, Write};

use keep::{AdmittedSegment, SegmentPublication, SegmentRecordLimit, SegmentStage, StagedSegment};

/// Reconstructs and closes the exact admitted segment for publication tests.
///
/// # Errors
///
/// Returns the exact record iteration, staging, sealing, or receipt-binding
/// failure.
pub fn for_segment<'selection, 'records>(
    segment: &'selection AdmittedSegment<'records>,
) -> Result<SegmentPublication<'selection, 'records>, Box<dyn Error>> {
    let mut staged = StagedSegment::begin(MemoryStage::default(), SegmentRecordLimit::MAXIMUM)?;
    for record in segment.records() {
        staged = staged.append(record?)?;
    }
    SegmentPublication::one(staged.seal()?.close(), segment).map_err(Into::into)
}

#[derive(Default)]
struct MemoryStage {
    bytes: Vec<u8>,
}

impl Write for MemoryStage {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl SegmentStage for MemoryStage {
    fn synchronize(&mut self) -> io::Result<()> {
        Ok(())
    }
}
