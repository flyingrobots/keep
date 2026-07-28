//! Checked completion of caller-provided blocking writes.

use std::io::{self, ErrorKind, Write};

pub(super) fn write_all<W>(
    output: &mut W,
    bytes: &[u8],
    written: &mut u64,
) -> Result<(), OutputWriteError>
where
    W: Write + ?Sized,
{
    let mut remaining = bytes;
    while !remaining.is_empty() {
        match output.write(remaining) {
            Ok(0) => {
                return Err(OutputWriteError::WriteZero {
                    bytes_written: *written,
                });
            }
            Ok(observed) => {
                remaining =
                    remaining
                        .get(observed..)
                        .ok_or(OutputWriteError::InvalidWriteCount {
                            maximum: remaining.len(),
                            observed,
                            bytes_written: *written,
                        })?;
                *written = checked_written(*written, observed)?;
            }
            Err(source) if source.kind() == ErrorKind::Interrupted => {}
            Err(source) => {
                return Err(OutputWriteError::Write {
                    bytes_written: *written,
                    source,
                });
            }
        }
    }
    Ok(())
}

fn checked_written(written: u64, incoming: usize) -> Result<u64, OutputWriteError> {
    let incoming_u64 =
        u64::try_from(incoming).map_err(|_source| OutputWriteError::LengthOverflow {
            bytes_written: written,
            incoming,
        })?;
    written
        .checked_add(incoming_u64)
        .ok_or(OutputWriteError::LengthOverflow {
            bytes_written: written,
            incoming,
        })
}

#[derive(Debug)]
pub(super) enum OutputWriteError {
    WriteZero {
        bytes_written: u64,
    },
    InvalidWriteCount {
        maximum: usize,
        observed: usize,
        bytes_written: u64,
    },
    Write {
        bytes_written: u64,
        source: io::Error,
    },
    LengthOverflow {
        bytes_written: u64,
        incoming: usize,
    },
}
