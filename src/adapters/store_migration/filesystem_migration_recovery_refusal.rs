//! This module owns typed filesystem migration recovery refusals.

use std::error::Error;
use std::fmt;
use std::num::TryFromIntError;

use super::StoreMigrationFixedStage;

/// Kind observed at a fixed migration protocol name without following links.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FilesystemMigrationResidueKind {
    /// A regular file.
    RegularFile,
    /// A directory.
    Directory,
    /// A link, device, pipe, or another unsupported kind.
    Other,
}

/// Semantic failure retained as an I/O payload during migration recovery.
#[derive(Debug)]
#[non_exhaustive]
pub enum FilesystemMigrationRecoveryRefusal {
    /// Nested migration directories contain invalid or out-of-order residue.
    NamespacePreflight {
        /// The original filesystem or namespace failure.
        source: std::io::Error,
    },
    /// The record's bounded observation length cannot be represented.
    ResidueBoundOverflow {
        /// The canonical record bound.
        length: usize,
    },
    /// A persistent reader fence is not an empty regular file.
    ReaderFence {
        /// The observed entry kind; an empty regular file is required.
        observed_kind: FilesystemMigrationResidueKind,
        /// The observed length; zero is required.
        observed_length: u64,
    },
    /// A directory-prefix name is not a directory.
    NamespaceKind {
        /// The observed kind.
        observed_kind: FilesystemMigrationResidueKind,
    },
    /// The canonical stage bound does not fit the filesystem length.
    StageLengthOverflow {
        /// The bound that failed conversion.
        length: usize,
        /// The original conversion failure.
        source: TryFromIntError,
    },
    /// Only an incomplete regular pre-effect stage can be discarded.
    StageNotIncomplete {
        /// The stage being considered.
        stage: StoreMigrationFixedStage,
        /// The first complete length, excluded from discard.
        complete_length: u64,
        /// The observed entry length.
        observed_length: u64,
        /// The observed entry kind.
        observed_kind: FilesystemMigrationResidueKind,
    },
    /// More than one complete fixed stage would be adopted simultaneously.
    MultipleExactStages,
    /// A crash hook requested a non-strict byte prefix.
    StagePrefix {
        /// The complete length, excluded from strict prefixes.
        complete_length: usize,
        /// The requested prefix length.
        observed: usize,
    },
    /// The supplied fixed record does not have its canonical length.
    RecordLength {
        /// The canonical length.
        expected: usize,
        /// The supplied length.
        observed: usize,
    },
}

impl fmt::Display for FilesystemMigrationRecoveryRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "migration recovery refused: {self:?}")
    }
}

impl Error for FilesystemMigrationRecoveryRefusal {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NamespacePreflight { source } => Some(source),
            Self::StageLengthOverflow { source, .. } => Some(source),
            _ => None,
        }
    }
}

pub(super) fn kind(metadata: &cap_std::fs::Metadata) -> FilesystemMigrationResidueKind {
    if metadata.is_file() {
        FilesystemMigrationResidueKind::RegularFile
    } else if metadata.is_dir() {
        FilesystemMigrationResidueKind::Directory
    } else {
        FilesystemMigrationResidueKind::Other
    }
}

pub(super) fn invalid(refusal: FilesystemMigrationRecoveryRefusal) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, refusal)
}
