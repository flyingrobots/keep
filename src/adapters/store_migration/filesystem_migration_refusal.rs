//! This module owns typed refusals of forward filesystem migration: fixed-stage
//! state, phase dispatch, and migration namespace admission.
//!
//! Each refusal travels as the payload of the `io::Error` a migration storage
//! port returns. Identity and mount contradictions use
//! [`FilesystemOperationRefusal`][operation]; restart-recovery refusals use
//! [`FilesystemMigrationRecoveryRefusal`][recovery].
//!
//! [operation]: crate::FilesystemOperationRefusal
//! [recovery]: super::FilesystemMigrationRecoveryRefusal

use std::error::Error;
use std::fmt;
use std::num::TryFromIntError;

use super::{StoreMigrationFixedStage, StoreMigrationPhase};
use crate::adapters::FilesystemEntryKind;

/// Why the filesystem migration adapter refused a phase or namespace state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FilesystemMigrationRefusal {
    /// A phase was dispatched to a section that does not execute it.
    PhaseSection {
        /// The misdispatched phase.
        observed: StoreMigrationPhase,
    },
    /// A fixed stage was written while another stage is still active.
    StageAlreadyActive {
        /// The stage that is active.
        active: StoreMigrationFixedStage,
    },
    /// A fixed stage was removed after its record was already published.
    RecordAlreadyPublished {
        /// The published record.
        stage: StoreMigrationFixedStage,
    },
    /// A phase needs an active fixed stage and none is active.
    StageInactive {
        /// The stage the phase requires.
        expected: StoreMigrationFixedStage,
    },
    /// A phase needs one fixed stage while a different stage is active.
    DifferentStageActive {
        /// The stage the phase requires.
        expected: StoreMigrationFixedStage,
        /// The stage that is active.
        observed: StoreMigrationFixedStage,
    },
    /// A phase needs a published fixed record that was never published.
    RecordNotPublished {
        /// The record the phase requires.
        stage: StoreMigrationFixedStage,
    },
    /// A link request names a record other than the retained stage.
    ///
    /// Equal stages mean the requested bytes differ from the retained bytes.
    StageRecordDisagreed {
        /// The stage this handle retains.
        retained: StoreMigrationFixedStage,
        /// The record the caller asked to link.
        requested: StoreMigrationFixedStage,
    },
    /// The root matches neither the before- nor after-fence membership.
    ReaderFencePredecessorNamespace,
    /// Namespace prefix admission stopped before every directory was admitted.
    NamespacePrefixIncomplete,
    /// A partial namespace prefix request exceeds the protocol's directories.
    NamespacePrefixCount {
        /// The protocol's directory count.
        maximum: usize,
        /// The requested count.
        observed: usize,
    },
    /// A nested prefix directory was requested before its parent.
    NamespaceParentAbsent,
    /// `gc` exists before the retention prefix is complete.
    GcBeforeRetention,
    /// `recovery` exists before `gc`.
    RecoveryBeforeGc,
    /// `retention/manifests` exists before `retention/roots`.
    ManifestsBeforeRoots,
    /// A migration namespace entry has the wrong kind.
    EntryKind {
        /// Kind the protocol requires.
        expected: FilesystemEntryKind,
        /// Kind observed without following a symbolic link.
        observed: FilesystemEntryKind,
    },
    /// A required migration namespace directory is absent.
    DirectoryAbsent {
        /// The protocol directory name.
        name: &'static str,
    },
    /// A required file length does not fit the filesystem's `u64` length.
    FileLengthOverflow {
        /// The required length.
        length: usize,
        /// The original checked conversion failure.
        source: TryFromIntError,
    },
    /// A required regular file has the wrong length.
    FileLength {
        /// The required length.
        expected: u64,
        /// The observed length.
        observed: u64,
    },
    /// A newly admitted migration directory was not empty.
    NamespaceNotEmpty,
    /// A directory's membership differs from the exact protocol set.
    MembershipDisagreed,
    /// Counting migration namespace entries overflowed.
    EntryCountOverflow,
    /// A migration namespace contains an entry outside the protocol set.
    UnknownEntry,
}

impl fmt::Display for FilesystemMigrationRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::PhaseSection { .. } => "migration phase dispatched to the wrong section",
            Self::StageAlreadyActive { .. } => "migration fixed stage was already active",
            Self::RecordAlreadyPublished { .. } => "migration fixed record was already published",
            Self::StageInactive { .. } => "migration fixed stage was not active",
            Self::DifferentStageActive { .. } => "a different migration fixed stage was active",
            Self::RecordNotPublished { .. } => "migration fixed record was not published",
            Self::StageRecordDisagreed { .. } => "migration stage record disagreed",
            Self::ReaderFencePredecessorNamespace => "reader-fence predecessor namespace disagreed",
            Self::NamespacePrefixIncomplete => "namespace prefix admission stopped short",
            Self::NamespacePrefixCount { .. } => "namespace prefix count exceeds the protocol",
            Self::NamespaceParentAbsent => "namespace prefix parent was not admitted",
            Self::GcBeforeRetention => "gc appeared before the retention prefix",
            Self::RecoveryBeforeGc => "recovery appeared before the gc prefix",
            Self::ManifestsBeforeRoots => "retention manifests appeared before roots",
            Self::EntryKind { .. } => "migration namespace entry has the wrong kind",
            Self::DirectoryAbsent { .. } => "required migration namespace directory was absent",
            Self::FileLengthOverflow { .. } => "required migration file length exceeded u64",
            Self::FileLength { .. } => "required migration file has the wrong length",
            Self::NamespaceNotEmpty => "new migration namespace was not empty",
            Self::MembershipDisagreed => "migration namespace membership disagreed",
            Self::EntryCountOverflow => "migration namespace count overflowed",
            Self::UnknownEntry => "migration namespace contains an unknown entry",
        })
    }
}

impl Error for FilesystemMigrationRefusal {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::FileLengthOverflow { source, .. } => Some(source),
            _ => None,
        }
    }
}
