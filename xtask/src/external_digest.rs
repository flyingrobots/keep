//! This module owns the bounded external digest witness.

use std::env;
use std::error::Error;
use std::fmt;
use std::process::Command;
use std::string::FromUtf8Error;
use std::time::Duration;

use crate::bounded_process::{
    CaptureLimits, ProcessError, ProcessOutput, capture_with_input_limits,
};
use crate::diagnostic::escaped_controls;

const B3SUM: &str = "b3sum";
const DIAGNOSTIC_LIMIT_BYTES: usize = 65_536;
const DIGEST_BYTES: usize = 32;
const PROCESS_DEADLINE: Duration = Duration::from_secs(10);
const PROCESS_LIMITS: CaptureLimits = CaptureLimits::new(DIGEST_BYTES, DIAGNOSTIC_LIMIT_BYTES);

/// A typed refusal from the external digest witness.
pub(crate) enum ExternalDigestError {
    /// The executable search path required for the hermetic child is absent.
    Environment {
        /// The missing admitted environment variable.
        variable: &'static str,
    },
    /// A bounded process operation failed.
    Process {
        /// The precise process-layer refusal.
        source: ProcessError,
    },
    /// A failed child emitted diagnostics that were not UTF-8.
    DiagnosticEncoding {
        /// The platform exit code, or `None` after signal termination.
        code: Option<i32>,
        /// The original diagnostic decoding failure.
        source: FromUtf8Error,
    },
    /// The child returned a non-success status with admitted diagnostics.
    Failed {
        /// The platform exit code, or `None` after signal termination.
        code: Option<i32>,
        /// The bounded UTF-8 diagnostic.
        stderr: String,
    },
    /// A successful child emitted an unexpected diagnostic.
    UnexpectedDiagnostic,
    /// A successful child returned a digest with the wrong width.
    Width {
        /// The number of bytes returned by the child.
        observed: usize,
    },
}

/// Computes one raw BLAKE3 digest through the independent `b3sum` witness.
///
/// Input parts are streamed without concatenation. The child receives only the
/// admitted executable search path and `C` locale, and the complete operation
/// is bounded by one ten-second deadline and independent output limits.
pub(crate) fn b3sum(parts: &[&[u8]]) -> Result<[u8; DIGEST_BYTES], ExternalDigestError> {
    let mut command = b3sum_command()?;
    execute(&mut command, parts, PROCESS_DEADLINE)
}

fn b3sum_command() -> Result<Command, ExternalDigestError> {
    let path = env::var_os("PATH").ok_or(ExternalDigestError::Environment { variable: "PATH" })?;
    let mut command = Command::new(B3SUM);
    command
        .args(["--raw", "--no-mmap", "--num-threads", "1"])
        .env_clear()
        .env("PATH", path)
        .env("LC_ALL", "C");
    Ok(command)
}

fn execute(
    command: &mut Command,
    parts: &[&[u8]],
    deadline: Duration,
) -> Result<[u8; DIGEST_BYTES], ExternalDigestError> {
    let output = capture_with_input_limits(B3SUM, command, parts, Some(deadline), PROCESS_LIMITS)
        .map_err(|source| ExternalDigestError::Process { source })?;
    validate(output)
}

fn validate(output: ProcessOutput) -> Result<[u8; DIGEST_BYTES], ExternalDigestError> {
    if !output.succeeded {
        return match String::from_utf8(output.stderr) {
            Ok(stderr) => Err(ExternalDigestError::Failed {
                code: output.code,
                stderr,
            }),
            Err(source) => Err(ExternalDigestError::DiagnosticEncoding {
                code: output.code,
                source,
            }),
        };
    }
    if !output.stderr.is_empty() {
        return Err(ExternalDigestError::UnexpectedDiagnostic);
    }
    let observed = output.stdout.len();
    output
        .stdout
        .try_into()
        .map_err(|_output| ExternalDigestError::Width { observed })
}

impl fmt::Debug for ExternalDigestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for ExternalDigestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Environment { variable } => {
                write!(
                    formatter,
                    "external digest requires the {variable} environment variable"
                )
            }
            Self::Process { source } => fmt::Display::fmt(source, formatter),
            Self::DiagnosticEncoding { code, .. } => write!(
                formatter,
                "{B3SUM} failed with status {code:?} and non-UTF-8 diagnostics"
            ),
            Self::Failed { code, stderr } => {
                write!(formatter, "{B3SUM} failed with status {code:?}: ")?;
                escaped_controls(formatter, stderr)
            }
            Self::UnexpectedDiagnostic => {
                write!(
                    formatter,
                    "{B3SUM} wrote diagnostics despite successful exit"
                )
            }
            Self::Width { observed } => write!(
                formatter,
                "{B3SUM} returned {observed} digest bytes instead of {DIGEST_BYTES}"
            ),
        }
    }
}

impl Error for ExternalDigestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Process { source } => Some(source),
            Self::DiagnosticEncoding { source, .. } => Some(source),
            Self::Environment { .. }
            | Self::Failed { .. }
            | Self::UnexpectedDiagnostic
            | Self::Width { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests;
