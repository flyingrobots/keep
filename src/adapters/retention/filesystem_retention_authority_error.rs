//! This boundary module owns filesystem retention authority refusals.

use std::error::Error;
use std::fmt;
use std::io;

/// Protocol directory a retention authority failed to pin.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionAuthorityDirectory {
    /// The pinned store root.
    Root,
    /// The `retention` protocol directory.
    Retention,
    /// The `retention/roots` immutable pool.
    Roots,
    /// The `retention/manifests` immutable pool.
    Manifests,
}

impl fmt::Display for RetentionAuthorityDirectory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Root => "store root",
            Self::Retention => "retention",
            Self::Roots => "retention/roots",
            Self::Manifests => "retention/manifests",
        })
    }
}

/// Exact refusal opening one writer-locked filesystem retention authority.
#[derive(Debug)]
#[non_exhaustive]
pub enum FilesystemRetentionAuthorityError {
    /// A required protocol directory could not be pinned without following links.
    Directory {
        /// The directory that refused.
        directory: RetentionAuthorityDirectory,
        /// The exact underlying failure.
        source: io::Error,
    },
}

impl fmt::Display for FilesystemRetentionAuthorityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Directory { directory, .. } => {
                write!(
                    formatter,
                    "could not pin {directory} for retention publication"
                )
            }
        }
    }
}

impl Error for FilesystemRetentionAuthorityError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Directory { source, .. } => Some(source),
        }
    }
}
