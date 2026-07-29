//! This module owns bounded exact descriptor-to-file copying.

use std::fs::File;
use std::io::{self, Write};
use std::os::unix::fs::FileExt;

const COPY_BUFFER_BYTES: usize = 16_384;

/// Copies exactly the admitted byte length from a descriptor at offset zero.
///
/// The fixed-size buffer bounds memory use. Short sources, offset overflow,
/// unrepresentable read lengths, and destination write failures are refused.
pub(crate) fn copy_exact(
    source: &File,
    destination: &mut File,
    expected: u64,
) -> Result<(), io::Error> {
    let mut offset = 0_u64;
    let mut buffer = [0_u8; COPY_BUFFER_BYTES];
    while offset < expected {
        let remaining = expected
            .checked_sub(offset)
            .ok_or_else(|| io::Error::other("source offset exceeded its admitted length"))?;
        let limit =
            usize::try_from(remaining).map_or(buffer.len(), |bytes| bytes.min(buffer.len()));
        let chunk = buffer
            .get_mut(..limit)
            .ok_or_else(|| io::Error::other("read bound exceeded the copy buffer"))?;
        let read = source.read_at(chunk, offset)?;
        if read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "source ended before its admitted length",
            ));
        }
        let copied = buffer
            .get(..read)
            .ok_or_else(|| io::Error::other("write bound exceeded the copy buffer"))?;
        destination.write_all(copied)?;
        offset = offset
            .checked_add(
                u64::try_from(read)
                    .map_err(|_| io::Error::other("source read length is not representable"))?,
            )
            .ok_or_else(|| io::Error::other("source offset overflowed"))?;
    }
    Ok(())
}
