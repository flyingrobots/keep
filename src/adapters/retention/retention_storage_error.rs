//! This module owns typed failures at the retention storage boundary.

use std::error::Error;
use std::fmt;
use std::io;

use super::{
    RetentionEffectDurability, RetentionKnownEffect, RetentionNamespaceEffect,
    RetentionRecordRefusal, RetentionStorageBoundary, RetentionStorageProgress,
};

/// An operational failure or exact-record refusal during retention storage.
#[derive(Debug)]
#[non_exhaustive]
pub enum RetentionStorageError {
    /// An operation stopped at a known boundary, possibly after namespace effects.
    Operation {
        /// The original typed storage cause, never stringified.
        source: Box<Self>,
        /// Known effects and uncertainty within this failing capability.
        progress: RetentionStorageProgress,
    },
    /// A filesystem operation failed.
    Io {
        /// The original operational error, including its OS code and source.
        source: io::Error,
    },
    /// The record did not match its admitted evidence.
    Refused {
        /// The precise record refusal.
        source: RetentionRecordRefusal,
    },
}

impl fmt::Display for RetentionStorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Operation { progress, .. } => write!(
                formatter,
                "retention storage stopped at {:?}; known effects {:?}, uncertain effect {:?}",
                progress.boundary, progress.known, progress.uncertain
            ),
            Self::Io { .. } => formatter.write_str("retention storage operation failed"),
            Self::Refused { source } => fmt::Display::fmt(source, formatter),
        }
    }
}

impl Error for RetentionStorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Operation { source, .. } => Some(source.as_ref()),
            Self::Io { source } => Some(source),
            Self::Refused { source } => Some(source),
        }
    }
}

impl From<io::Error> for RetentionStorageError {
    fn from(source: io::Error) -> Self {
        Self::Io { source }
    }
}

impl From<RetentionStorageError> for io::Error {
    fn from(error: RetentionStorageError) -> Self {
        match error {
            RetentionStorageError::Io { source } => source,
            refused @ (RetentionStorageError::Refused { .. }
            | RetentionStorageError::Operation { .. }) => Self::new(refused.io_kind(), refused),
        }
    }
}

impl RetentionStorageError {
    fn io_kind(&self) -> io::ErrorKind {
        match self {
            Self::Io { source } => source.kind(),
            Self::Refused { .. } => io::ErrorKind::InvalidData,
            Self::Operation { source, .. } => source.io_kind(),
        }
    }

    pub(in crate::adapters) fn at(self, boundary: RetentionStorageBoundary) -> Self {
        match self {
            Self::Operation { .. } => self,
            source => Self::Operation {
                source: Box::new(source),
                progress: RetentionStorageProgress {
                    boundary,
                    known: Vec::new(),
                    uncertain: None,
                },
            },
        }
    }

    pub(in crate::adapters) fn after(
        mut self,
        effect: RetentionNamespaceEffect,
        durability: RetentionEffectDurability,
    ) -> Self {
        if let Self::Operation { progress, .. } = &mut self {
            progress
                .known
                .insert(0, RetentionKnownEffect { effect, durability });
        }
        self
    }

    pub(in crate::adapters) const fn uncertain(mut self, effect: RetentionNamespaceEffect) -> Self {
        if let Self::Operation { progress, .. } = &mut self {
            progress.uncertain = Some(effect);
        }
        self
    }
    /// Progress within the failing capability, if its adapter reports it.
    ///
    /// `None` means unreported effects, not absence of effects. Callers must reobserve.
    #[must_use]
    pub const fn progress(&self) -> Option<&RetentionStorageProgress> {
        match self {
            Self::Operation { progress, .. } => Some(progress),
            _ => None,
        }
    }
}
