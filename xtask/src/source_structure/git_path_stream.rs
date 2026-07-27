//! This module owns bounded decoding of Git's NUL-framed path stream.

use std::collections::BTreeSet;
use std::io::Read;

use super::{GitOutputUnit, SourceStructureError};

const GIT_PATH_LIMITS: GitPathLimits = GitPathLimits {
    path_bytes: 4_096,
    stream_bytes: 16_777_216,
    paths: 100_000,
};

#[derive(Clone, Copy)]
struct GitPathLimits {
    path_bytes: usize,
    stream_bytes: usize,
    paths: usize,
}

pub(super) fn read_paths(
    reader: impl Read,
    operation: &'static str,
) -> Result<BTreeSet<GitPathRecord>, SourceStructureError> {
    read_paths_with(reader, operation, GIT_PATH_LIMITS)
}

fn read_paths_with(
    mut reader: impl Read,
    operation: &'static str,
    limits: GitPathLimits,
) -> Result<BTreeSet<GitPathRecord>, SourceStructureError> {
    let mut decoder = GitPathDecoder::new(operation, limits);
    let mut buffer = [0_u8; 4_096];
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|source| SourceStructureError::RunGit {
                operation,
                action: "read paths from",
                source,
            })?;
        if read == 0 {
            break;
        }
        let bytes = buffer
            .get(..read)
            .ok_or(SourceStructureError::GitOutputBound {
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
    paths: BTreeSet<GitPathRecord>,
}

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct GitPathRecord(Vec<u8>);

impl GitPathRecord {
    pub(super) const fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub(super) fn as_bytes(&self) -> &[u8] {
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

    fn admit(&mut self, bytes: &[u8]) -> Result<(), SourceStructureError> {
        self.observed_bytes = self.observed_bytes.checked_add(bytes.len()).ok_or(
            SourceStructureError::GitOutputBound {
                operation: self.operation,
                stream: "path stream bytes",
                maximum: self.limits.stream_bytes,
                unit: GitOutputUnit::Bytes,
            },
        )?;
        if self.observed_bytes > self.limits.stream_bytes {
            return Err(SourceStructureError::GitOutputBound {
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

    fn admit_byte(&mut self, byte: u8) -> Result<(), SourceStructureError> {
        if byte == 0 {
            return self.admit_path();
        }
        if self.current.len() >= self.limits.path_bytes {
            return Err(SourceStructureError::GitOutputBound {
                operation: self.operation,
                stream: "path bytes",
                maximum: self.limits.path_bytes,
                unit: GitOutputUnit::Bytes,
            });
        }
        self.current.push(byte);
        Ok(())
    }

    fn admit_path(&mut self) -> Result<(), SourceStructureError> {
        if self.current.is_empty() {
            return Err(SourceStructureError::GitOutputFraming {
                operation: self.operation,
            });
        }
        self.observed_paths =
            self.observed_paths
                .checked_add(1)
                .ok_or(SourceStructureError::GitOutputBound {
                    operation: self.operation,
                    stream: "path count",
                    maximum: self.limits.paths,
                    unit: GitOutputUnit::Items,
                })?;
        if self.observed_paths > self.limits.paths {
            return Err(SourceStructureError::GitOutputBound {
                operation: self.operation,
                stream: "path count",
                maximum: self.limits.paths,
                unit: GitOutputUnit::Items,
            });
        }
        let path = GitPathRecord::new(std::mem::take(&mut self.current));
        if self.paths.insert(path.clone()) {
            Ok(())
        } else {
            Err(SourceStructureError::DuplicatePath(
                path.as_bytes().to_vec(),
            ))
        }
    }

    fn finish(self) -> Result<BTreeSet<GitPathRecord>, SourceStructureError> {
        if self.current.is_empty() {
            Ok(self.paths)
        } else {
            Err(SourceStructureError::GitOutputFraming {
                operation: self.operation,
            })
        }
    }
}

#[cfg(test)]
#[path = "git_path_stream/tests.rs"]
mod tests;
