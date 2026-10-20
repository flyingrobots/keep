//! This module owns migration filesystem protocol-state refusals.

use std::error::Error;
use std::fmt;

/// A semantic filesystem protocol refusal, preserved through I/O adapters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FilesystemMigrationRefusal {
    /// Version-one staging holds evidence that must be recovered before migration.
    VersionOneStageRequiresRecovery,
    /// A recovery phase was dispatched outside its protocol section.
    WrongPhaseSection {
        /// The phase that cannot be executed by that section.
        observed: super::StoreMigrationPhase,
    },
    /// Migration fixed stage was already active.
    StageAlreadyActive,
    /// Migration fixed record was already published.
    RecordAlreadyPublished,
    /// Migration fixed stage was not active.
    NoActiveStage,
    /// A different migration fixed stage was active.
    DifferentStageActive,
    /// Migration fixed record was not published.
    RecordNotPublished,
    /// Reader fence kind, length, or identity disagreed.
    ReaderFenceChanged,
    /// Migration residue bound overflowed.
    ResidueBoundOverflow,
    /// Reader fence has the wrong kind or length.
    ReaderFenceKindOrLength,
    /// Migration namespace entry has the wrong kind.
    NamespaceKind,
    /// Migration directory changed identity.
    DirectoryIdentityChanged,
    /// Migration namespace crossed the admitted filesystem or mount.
    NamespaceMountChanged,
    /// Required migration namespace directory was absent.
    DirectoryAbsent,
    /// Required migration directory has the wrong kind.
    RequiredDirectoryKind,
    /// Required migration file length exceeded u64.
    RequiredFileLengthOverflow,
    /// Required migration file has the wrong kind or length.
    RequiredFileKindOrLength,
    /// New migration namespace was not empty.
    NamespaceNotEmpty,
    /// Migration namespace membership disagreed.
    NamespaceMembership,
    /// Migration namespace count overflowed.
    NamespaceCountOverflow,
    /// Migration namespace contains an unknown entry.
    UnknownNamespaceEntry,
    /// Reader-fence predecessor namespace disagreed.
    ReaderFencePredecessorNamespace,
    /// Namespace prefix admission stopped short.
    NamespacePrefixIncomplete,
    /// Namespace prefix count exceeds the protocol.
    NamespacePrefixCount,
    /// Namespace prefix parent was not admitted.
    NamespaceParentAbsent,
    /// Gc appeared before the retention prefix.
    GcBeforeRetention,
    /// Recovery appeared before the gc prefix.
    RecoveryBeforeGc,
    /// Retention manifests appeared before roots.
    ManifestsBeforeRoots,
    /// Migration stage length exceeded u64.
    StageLengthOverflow,
    /// Only an incomplete regular pre-effect stage may be discarded.
    DiscardRequiresIncompleteStage,
    /// Migration residue holds more than one exact stage.
    MultipleExactStages,
    /// Migration stage prefix is not strict.
    StagePrefix,
    /// Migration stage handle changed identity.
    StageIdentityChanged,
    /// Migration stage record disagreed.
    StageRecord,
    /// Migration fixed-record length disagreed.
    RecordLength,
}

impl fmt::Display for FilesystemMigrationRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::VersionOneStageRequiresRecovery => {
                "version-one staging holds a retained stage; recover it before migration"
            }
            Self::WrongPhaseSection { .. } => "migration phase dispatched to the wrong section",
            Self::StageAlreadyActive => "migration fixed stage was already active",
            Self::RecordAlreadyPublished => "migration fixed record was already published",
            Self::NoActiveStage => "migration fixed stage was not active",
            Self::DifferentStageActive => "a different migration fixed stage was active",
            Self::RecordNotPublished => "migration fixed record was not published",
            Self::ReaderFenceChanged => "reader fence kind, length, or identity disagreed",
            Self::ResidueBoundOverflow => "migration residue bound overflowed",
            Self::ReaderFenceKindOrLength => "reader fence has the wrong kind or length",
            Self::NamespaceKind => "migration namespace entry has the wrong kind",
            Self::DirectoryIdentityChanged => "migration directory changed identity",
            Self::NamespaceMountChanged => {
                "migration namespace crossed the admitted filesystem or mount"
            }
            Self::DirectoryAbsent => "required migration namespace directory was absent",
            Self::RequiredDirectoryKind => "required migration directory has the wrong kind",
            Self::RequiredFileLengthOverflow => "required migration file length exceeded u64",
            Self::RequiredFileKindOrLength => {
                "required migration file has the wrong kind or length"
            }
            Self::NamespaceNotEmpty => "new migration namespace was not empty",
            Self::NamespaceMembership => "migration namespace membership disagreed",
            Self::NamespaceCountOverflow => "migration namespace count overflowed",
            Self::UnknownNamespaceEntry => "migration namespace contains an unknown entry",
            Self::ReaderFencePredecessorNamespace => "reader-fence predecessor namespace disagreed",
            Self::NamespacePrefixIncomplete => "namespace prefix admission stopped short",
            Self::NamespacePrefixCount => "namespace prefix count exceeds the protocol",
            Self::NamespaceParentAbsent => "namespace prefix parent was not admitted",
            Self::GcBeforeRetention => "gc appeared before the retention prefix",
            Self::RecoveryBeforeGc => "recovery appeared before the gc prefix",
            Self::ManifestsBeforeRoots => "retention manifests appeared before roots",
            Self::StageLengthOverflow => "migration stage length exceeded u64",
            Self::DiscardRequiresIncompleteStage => {
                "only an incomplete regular pre-effect stage may be discarded"
            }
            Self::MultipleExactStages => "migration residue holds more than one exact stage",
            Self::StagePrefix => "migration stage prefix is not strict",
            Self::StageIdentityChanged => "migration stage handle changed identity",
            Self::StageRecord => "migration stage record disagreed",
            Self::RecordLength => "migration fixed-record length disagreed",
        })
    }
}

impl Error for FilesystemMigrationRefusal {}
