//! This module owns one deadline spanning process and output collection.

use std::time::{Duration, Instant};

use super::ProcessError;

pub(super) enum ProcessDeadline {
    Bounded {
        duration: Duration,
        expires: Instant,
    },
    Unbounded,
}

impl ProcessDeadline {
    pub(super) fn new(
        program: &'static str,
        duration: Option<Duration>,
    ) -> Result<Self, ProcessError> {
        let Some(duration) = duration else {
            return Ok(Self::Unbounded);
        };
        let expires = Instant::now()
            .checked_add(duration)
            .ok_or(ProcessError::Timeout { program, duration })?;
        Ok(Self::Bounded { duration, expires })
    }

    pub(super) fn remaining(
        &self,
        program: &'static str,
    ) -> Result<Option<(Duration, Duration)>, ProcessError> {
        match self {
            Self::Unbounded => Ok(None),
            Self::Bounded { duration, expires } => expires
                .checked_duration_since(Instant::now())
                .map(|remaining| Some((remaining, *duration)))
                .ok_or(ProcessError::Timeout {
                    program,
                    duration: *duration,
                }),
        }
    }
}
