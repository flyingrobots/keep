//! This module owns retention filesystem protocol-state refusals.

use std::error::Error;
use std::fmt;

/// A semantic filesystem protocol refusal, preserved through I/O adapters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum FilesystemRetentionStageRefusal {
    /// A retained stage is not a regular file.
    NonRegularRecoveryStage,
    /// The checked bound cannot include a corruption witness.
    RecoveryStageBoundOverflow,
    /// No admitted retention publication attempt.
    NoPublicationAttempt,
    /// Retention root namespace was not admitted.
    NamespaceNotAdmitted,
    /// Retention root pool coordinate was not retained.
    RootPoolNotRetained,
    /// Retention manifest pool coordinate was not retained.
    ManifestPoolNotRetained,
    /// Retention root stage was not retained.
    RootStageNotRetained,
    /// Retention manifest stage was not retained.
    ManifestStageNotRetained,
    /// Retention head stage was not retained.
    HeadStageNotRetained,
    /// No disposition is in progress.
    NoDisposition,
    /// Retired pool entry is already absent.
    PoolEntryAbsent,
    /// Retired pool entry bytes disagree with the receipt.
    PoolEntryChanged,
    /// Disposition target root stage vanished.
    DispositionRootStageAbsent,
    /// Disposition target manifest stage vanished.
    DispositionManifestStageAbsent,
    /// Artifact is shorter than its checksum.
    ArtifactChecksumLength,
    /// Artifact checksum slot.
    ArtifactChecksumSlot,
    /// Retention pool entry is not a regular file.
    PoolEntryKind,
    /// Retention pool entry vanished.
    PoolEntryVanished,
    /// Disposition stage is not a regular file.
    DispositionStageKind,
    /// Retention stage handle changed identity.
    HandleIdentityChanged,
    /// Publication head is absent.
    HeadAbsent,
    /// Publication head checksum slot.
    HeadChecksumSlot,
    /// No retention recovery is in progress.
    NoRecovery,
    /// Recovery step expected a complete stage.
    RecoveryRequiresCompleteStage,
    /// Recovery step expected a retained stage.
    NoRetainedRecoveryStage,
    /// Recovery step expected a truncated stage.
    RecoveryRequiresTruncatedStage,
    /// Truncated retention stage changed before discard.
    TruncatedStageChanged,
    /// Root linking requires a complete root stage.
    RootLinkRequiresCompleteStage,
    /// Manifest linking requires a complete manifest stage.
    ManifestLinkRequiresCompleteStage,
    /// Root stage without a namespace.
    RootNamespaceAbsent,
}

impl fmt::Display for FilesystemRetentionStageRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NonRegularRecoveryStage => "retained retention stage is not a regular file",
            Self::RecoveryStageBoundOverflow => "stage bound overflowed",
            Self::NoPublicationAttempt => "no admitted retention publication attempt",
            Self::NamespaceNotAdmitted => "retention root namespace was not admitted",
            Self::RootPoolNotRetained => "retention root pool coordinate was not retained",
            Self::ManifestPoolNotRetained => "retention manifest pool coordinate was not retained",
            Self::RootStageNotRetained => "retention root stage was not retained",
            Self::ManifestStageNotRetained => "retention manifest stage was not retained",
            Self::HeadStageNotRetained => "retention head stage was not retained",
            Self::NoDisposition => "no disposition is in progress",
            Self::PoolEntryAbsent => "retired pool entry is already absent",
            Self::PoolEntryChanged => "retired pool entry bytes disagree with the receipt",
            Self::DispositionRootStageAbsent => "disposition target root stage vanished",
            Self::DispositionManifestStageAbsent => "disposition target manifest stage vanished",
            Self::ArtifactChecksumLength => "artifact is shorter than its checksum",
            Self::ArtifactChecksumSlot => "artifact checksum slot",
            Self::PoolEntryKind => "retention pool entry is not a regular file",
            Self::PoolEntryVanished => "retention pool entry vanished",
            Self::DispositionStageKind => "disposition stage is not a regular file",
            Self::HandleIdentityChanged => "retention stage handle changed identity",
            Self::HeadAbsent => "publication head is absent",
            Self::HeadChecksumSlot => "publication head checksum slot",
            Self::NoRecovery => "no retention recovery is in progress",
            Self::RecoveryRequiresCompleteStage => "recovery step expected a complete stage",
            Self::NoRetainedRecoveryStage => "recovery step expected a retained stage",
            Self::RecoveryRequiresTruncatedStage => "recovery step expected a truncated stage",
            Self::TruncatedStageChanged => "truncated retention stage changed before discard",
            Self::RootLinkRequiresCompleteStage => "link_root expected a complete root stage",
            Self::ManifestLinkRequiresCompleteStage => {
                "link_manifest expected a complete manifest stage"
            }
            Self::RootNamespaceAbsent => "root stage without a namespace",
        })
    }
}

impl Error for FilesystemRetentionStageRefusal {}
