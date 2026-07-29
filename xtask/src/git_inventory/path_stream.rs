//! This module owns bounded decoding of Git's NUL-framed path stream.

use std::collections::BTreeSet;
use std::io::Read;

use super::{GitInventoryError, GitOutputUnit};

/// Maximum retained bytes in one complete Git path stream.
pub(super) const GIT_PATH_STREAM_LIMIT_BYTES: usize = 16_777_216;

const GIT_PATH_LIMITS: GitPathLimits = GitPathLimits {
    path_bytes: 4_096,
    stream_bytes: GIT_PATH_STREAM_LIMIT_BYTES,
    paths: 100_000,
};

#[derive(Clone, Copy)]
struct GitPathLimits {
    path_bytes: usize,
    stream_bytes: usize,
    paths: usize,
}

/// Decodes Git's NUL-framed path output into deterministic bytewise order.
///
/// The call blocks while reading and refuses empty, duplicate, unterminated,
/// oversized, or excessive path records. Individual paths are limited to 4,096
/// bytes; the stream is limited to 16 MiB and 100,000 records.
pub(super) fn read_paths(
    reader: impl Read,
    operation: &'static str,
) -> Result<BTreeSet<GitPath>, GitInventoryError> {
    read_paths_with(reader, operation, GIT_PATH_LIMITS)
}

fn read_paths_with(
    mut reader: impl Read,
    operation: &'static str,
    limits: GitPathLimits,
) -> Result<BTreeSet<GitPath>, GitInventoryError> {
    let mut decoder = GitPathDecoder::new(operation, limits);
    let mut buffer = [0_u8; 4_096];
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|source| GitInventoryError::Run {
                operation,
                action: "read paths from",
                source,
            })?;
        if read == 0 {
            break;
        }
        let bytes = buffer.get(..read).ok_or(GitInventoryError::OutputBound {
            operation,
            stream: "path read",
            maximum: buffer.len(),
            unit: GitOutputUnit::Bytes,
        })?;
        decoder.admit(bytes)?;
    }
    decoder.finish()
}

struct GitPathDecoder {
    current: Vec<u8>,
    limits: GitPathLimits,
    observed_bytes: usize,
    observed_paths: usize,
    operation: &'static str,
    paths: BTreeSet<GitPath>,
}

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
/// An opaque repository path admitted from Git's byte-oriented output.
///
/// Ordering is bytewise and therefore independent of locale and filesystem
/// enumeration order.
pub(crate) struct GitPath(Vec<u8>);

impl GitPath {
    /// Wraps path bytes that the stream decoder has already admitted.
    pub(crate) const fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Returns the exact repository-relative bytes reported by Git.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl GitPathDecoder {
    const fn new(operation: &'static str, limits: GitPathLimits) -> Self {
        Self {
            current: Vec::new(),
            limits,
            observed_bytes: 0,
            observed_paths: 0,
            operation,
            paths: BTreeSet::new(),
        }
    }

    fn admit(&mut self, bytes: &[u8]) -> Result<(), GitInventoryError> {
        self.observed_bytes =
            self.observed_bytes
                .checked_add(bytes.len())
                .ok_or(GitInventoryError::OutputBound {
                    operation: self.operation,
                    stream: "path stream bytes",
                    maximum: self.limits.stream_bytes,
                    unit: GitOutputUnit::Bytes,
                })?;
        if self.observed_bytes > self.limits.stream_bytes {
            return Err(GitInventoryError::OutputBound {
                operation: self.operation,
                stream: "path stream bytes",
                maximum: self.limits.stream_bytes,
                unit: GitOutputUnit::Bytes,
            });
        }
        for byte in bytes {
            self.admit_byte(*byte)?;
        }
        Ok(())
    }

    fn admit_byte(&mut self, byte: u8) -> Result<(), GitInventoryError> {
        if byte == 0 {
            return self.admit_path();
        }
        if self.current.len() >= self.limits.path_bytes {
            return Err(GitInventoryError::OutputBound {
                operation: self.operation,
                stream: "path bytes",
                maximum: self.limits.path_bytes,
                unit: GitOutputUnit::Bytes,
            });
        }
        self.current.push(byte);
        Ok(())
    }

    fn admit_path(&mut self) -> Result<(), GitInventoryError> {
        if self.current.is_empty() {
            return Err(GitInventoryError::EmptyPath {
                operation: self.operation,
            });
        }
        self.observed_paths =
            self.observed_paths
                .checked_add(1)
                .ok_or(GitInventoryError::OutputBound {
                    operation: self.operation,
                    stream: "path count",
                    maximum: self.limits.paths,
                    unit: GitOutputUnit::Items,
                })?;
        if self.observed_paths > self.limits.paths {
            return Err(GitInventoryError::OutputBound {
                operation: self.operation,
                stream: "path count",
                maximum: self.limits.paths,
                unit: GitOutputUnit::Items,
            });
        }
        let path = GitPath::new(std::mem::take(&mut self.current));
        if self.paths.insert(path.clone()) {
            Ok(())
        } else {
            Err(GitInventoryError::DuplicatePath(path.as_bytes().to_vec()))
        }
    }

    fn finish(self) -> Result<BTreeSet<GitPath>, GitInventoryError> {
        if self.current.is_empty() {
            Ok(self.paths)
        } else {
            Err(GitInventoryError::OutputFraming {
                operation: self.operation,
            })
        }
    }
}

#[cfg(test)]
#[path = "path_stream/tests.rs"]
mod tests;
