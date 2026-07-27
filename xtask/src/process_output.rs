//! This module owns bounded collection of external-process byte streams.

use std::io::{self, Read};

pub(crate) struct BoundedBytes {
    pub(crate) bytes: Vec<u8>,
    pub(crate) exceeded: bool,
}

pub(crate) fn bounded_bytes(
    mut reader: impl Read,
    maximum: usize,
) -> Result<BoundedBytes, io::Error> {
    let mut buffer = [0_u8; 4_096];
    let mut bytes = Vec::new();
    let mut exceeded = false;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        let remaining = maximum.saturating_sub(bytes.len());
        let admitted = read.min(remaining);
        let chunk = buffer
            .get(..admitted)
            .ok_or_else(|| io::Error::other("process output admission overflow"))?;
        bytes.extend_from_slice(chunk);
        exceeded |= admitted < read;
    }
    Ok(BoundedBytes { bytes, exceeded })
}
