//! This module owns exact all-target fuzz execution failures.

use std::error::Error;
use std::fmt;

use crate::fuzz_campaign::process::ProcessError;

pub(crate) struct ExecutionError {
    pub(super) operation: &'static str,
    pub(super) failures: Vec<TargetFailure>,
}

pub(super) struct TargetFailure {
    pub(super) target: String,
    pub(super) reason: TargetFailureReason,
}

pub(super) enum TargetFailureReason {
    Exit,
    Process(ProcessError),
    RefusedOutput,
}

impl fmt::Debug for ExecutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "fuzz {} failed for: ", self.operation)?;
        for (index, failure) in self.failures.iter().enumerate() {
            if index != 0 {
                formatter.write_str(", ")?;
            }
            write!(
                formatter,
                "{} ({})",
                failure.target,
                FailureReason(&failure.reason)
            )?;
        }
        Ok(())
    }
}

impl Error for ExecutionError {}

struct FailureReason<'a>(&'a TargetFailureReason);

impl fmt::Display for FailureReason<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            TargetFailureReason::Exit => formatter.write_str("nonzero exit"),
            TargetFailureReason::Process(error) => write!(formatter, "{error}"),
            TargetFailureReason::RefusedOutput => {
                formatter.write_str("cargo-fuzz reported a swallowed failure")
            }
        }
    }
}
