//! This module owns observed device and mount contradictions during admission.

use std::{error::Error, fmt};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FilesystemMountRefusal {
    Device { expected: u64, observed: u64 },
    Mount { expected: u64, observed: u64 },
}

impl fmt::Display for FilesystemMountRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Device { expected, observed } => write!(
                formatter,
                "protocol directory device is {observed}, expected {expected}"
            ),
            Self::Mount { expected, observed } => write!(
                formatter,
                "protocol directory mount is {observed}, expected {expected}"
            ),
        }
    }
}

impl Error for FilesystemMountRefusal {}
