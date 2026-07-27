use std::collections::BTreeSet;
use std::io::{self, Read};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;

use super::SourceStructureError;

const GIT_OUTPUT_LIMITS: GitOutputLimits = GitOutputLimits {
    diagnostic_bytes: 65_536,
    path_bytes: 4_096,
    path_stream_bytes: 16_777_216,
    paths: 100_000,
};

#[derive(Clone, Copy)]
struct GitOutputLimits {
    diagnostic_bytes: usize,
    path_bytes: usize,
    path_stream_bytes: usize,
    paths: usize,
}

pub(super) fn git_paths(
    repository_root: &Path,
    arguments: &[&str],
    operation: &'static str,
) -> Result<BTreeSet<String>, SourceStructureError> {
    let mut child = Command::new("git")
        .args(arguments)
        .current_dir(repository_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| SourceStructureError::RunGit {
            operation,
            action: "start",
            source,
        })?;
    let Some(stdout) = child.stdout.take() else {
        cleanup_child(&mut child, operation)?;
        return Err(SourceStructureError::GitPipe {
            operation,
            stream: "stdout",
        });
    };
    let Some(stderr) = child.stderr.take() else {
        cleanup_child(&mut child, operation)?;
        return Err(SourceStructureError::GitPipe {
            operation,
            stream: "stderr",
        });
    };
    let diagnostic_worker = thread::Builder::new()
        .name(String::from("xtask-git-diagnostic"))
        .spawn(move || bounded_bytes(stderr, GIT_OUTPUT_LIMITS.diagnostic_bytes));
    let diagnostic_worker = match diagnostic_worker {
        Ok(worker) => worker,
        Err(source) => {
            cleanup_child(&mut child, operation)?;
            return Err(SourceStructureError::RunGit {
                operation,
                action: "start the diagnostic reader for",
                source,
            });
        }
    };

    let paths = read_paths(stdout, operation, GIT_OUTPUT_LIMITS);
    let stop = if paths.is_err() {
        request_stop(&mut child, operation)
    } else {
        Ok(())
    };
    let status = child.wait().map_err(|source| SourceStructureError::RunGit {
        operation,
        action: "wait for",
        source,
    });
    let diagnostic = diagnostic_worker
        .join()
        .map_err(|_| SourceStructureError::GitWorker { operation })?;

    stop?;
    let paths = paths?;
    let status = status?;
    let diagnostic = diagnostic.map_err(|source| SourceStructureError::RunGit {
        operation,
        action: "read diagnostics from",
        source,
    })?;
    if diagnostic.exceeded {
        return Err(SourceStructureError::GitOutputBound {
            operation,
            stream: "diagnostic bytes",
            maximum: GIT_OUTPUT_LIMITS.diagnostic_bytes,
        });
    }
    if !status.success() {
        let stderr = String::from_utf8(diagnostic.bytes)
            .map_err(|source| SourceStructureError::GitOutput { operation, source })?;
        return Err(SourceStructureError::GitFailed {
            operation,
            code: status.code(),
            stderr,
        });
    }
    Ok(paths)
}

fn request_stop(child: &mut Child, operation: &'static str) -> Result<(), SourceStructureError> {
    child
        .kill()
        .or_else(|source| {
            if source.kind() == io::ErrorKind::InvalidInput {
                Ok(())
            } else {
                Err(source)
            }
        })
        .map_err(|source| SourceStructureError::RunGit {
            operation,
            action: "stop",
            source,
        })
}

fn cleanup_child(child: &mut Child, operation: &'static str) -> Result<(), SourceStructureError> {
    let stop = request_stop(child, operation);
    let wait = child.wait().map_err(|source| SourceStructureError::RunGit {
        operation,
        action: "wait for",
        source,
    });
    stop?;
    wait.map(|_| ())
}

fn read_paths(
    mut reader: impl Read,
    operation: &'static str,
    limits: GitOutputLimits,
) -> Result<BTreeSet<String>, SourceStructureError> {
    let mut buffer = [0_u8; 4_096];
    let mut current = Vec::new();
    let mut observed_bytes = 0_usize;
    let mut observed_paths = 0_usize;
    let mut paths = BTreeSet::new();
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
        observed_bytes =
            observed_bytes
                .checked_add(read)
                .ok_or(SourceStructureError::GitOutputBound {
                    operation,
                    stream: "path stream bytes",
                    maximum: limits.path_stream_bytes,
                })?;
        if observed_bytes > limits.path_stream_bytes {
            return Err(SourceStructureError::GitOutputBound {
                operation,
                stream: "path stream bytes",
                maximum: limits.path_stream_bytes,
            });
        }
        let bytes = buffer
            .get(..read)
            .ok_or(SourceStructureError::GitOutputBound {
                operation,
                stream: "path read",
                maximum: buffer.len(),
            })?;
        for byte in bytes {
            if *byte == 0 {
                if !current.is_empty() {
                    observed_paths = observed_paths.checked_add(1).ok_or(
                        SourceStructureError::GitOutputBound {
                            operation,
                            stream: "path count",
                            maximum: limits.paths,
                        },
                    )?;
                    if observed_paths > limits.paths {
                        return Err(SourceStructureError::GitOutputBound {
                            operation,
                            stream: "path count",
                            maximum: limits.paths,
                        });
                    }
                    let path = String::from_utf8(std::mem::take(&mut current))
                        .map_err(|source| SourceStructureError::GitOutput { operation, source })?;
                    paths.insert(path);
                }
            } else {
                if current.len() >= limits.path_bytes {
                    return Err(SourceStructureError::GitOutputBound {
                        operation,
                        stream: "path bytes",
                        maximum: limits.path_bytes,
                    });
                }
                current.push(*byte);
            }
        }
    }
    if current.is_empty() {
        Ok(paths)
    } else {
        Err(SourceStructureError::GitOutputFraming { operation })
    }
}

struct BoundedBytes {
    bytes: Vec<u8>,
    exceeded: bool,
}

fn bounded_bytes(mut reader: impl Read, maximum: usize) -> Result<BoundedBytes, io::Error> {
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
            .ok_or_else(|| io::Error::other("diagnostic buffer admission overflow"))?;
        bytes.extend_from_slice(chunk);
        exceeded |= admitted < read;
    }
    Ok(BoundedBytes { bytes, exceeded })
}

#[cfg(test)]
mod tests;
