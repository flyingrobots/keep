//! Checked completion of one blocking segment-stage write.

use std::io::ErrorKind;

use super::{SegmentStage, SegmentWriteError, SegmentWritePhase};

pub(super) fn write_all<S>(
    stage: &mut S,
    bytes: &[u8],
    phase: SegmentWritePhase,
    written: &mut u64,
) -> Result<(), SegmentWriteError>
where
    S: SegmentStage,
{
    let mut remaining = bytes;
    while !remaining.is_empty() {
        match stage.write(remaining) {
            Ok(0) => {
                return Err(SegmentWriteError::WriteZero {
                    phase,
                    bytes_written: *written,
                });
            }
            Ok(observed) => {
                remaining =
                    remaining
                        .get(observed..)
                        .ok_or(SegmentWriteError::InvalidWriteCount {
                            phase,
                            maximum: remaining.len(),
                            observed,
                            bytes_written: *written,
                        })?;
                *written = checked_written(*written, observed, phase)?;
            }
            Err(source) if source.kind() == ErrorKind::Interrupted => {}
            Err(source) => {
                return Err(SegmentWriteError::Write {
                    phase,
                    bytes_written: *written,
                    source,
                });
            }
        }
    }
    Ok(())
}

fn checked_written(
    written: u64,
    incoming: usize,
    phase: SegmentWritePhase,
) -> Result<u64, SegmentWriteError> {
    let incoming_u64 =
        u64::try_from(incoming).map_err(|_source| SegmentWriteError::WriteLengthArithmetic {
            phase,
            bytes_written: written,
            incoming,
        })?;
    written
        .checked_add(incoming_u64)
        .ok_or(SegmentWriteError::WriteLengthArithmetic {
            phase,
            bytes_written: written,
            incoming,
        })
}
