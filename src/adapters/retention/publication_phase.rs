//! This boundary module owns exact retention publication durability phases.

use std::fmt;

/// Storage transition attempted by retention namespace publication.
///
/// [`Self::ALL`] corresponds in order to `KEEP-CRASH-036` through
/// `KEEP-CRASH-052`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionPublicationPhase {
    /// Write the complete canonical `root.next`.
    WriteRootStage,
    /// Synchronize `root.next`.
    SynchronizeRootStage,
    /// Create or exactly admit the digest-named root namespace.
    AdmitRootNamespace,
    /// Synchronize `retention/roots` after namespace admission.
    SynchronizeRootsAfterNamespace,
    /// Link the synchronized root stage into its immutable namespace.
    LinkRoot,
    /// Synchronize the digest-named root namespace after linking.
    SynchronizeRootNamespace,
    /// Write the complete canonical `manifest.next`.
    WriteManifestStage,
    /// Synchronize `manifest.next`.
    SynchronizeManifestStage,
    /// Link the synchronized manifest into its immutable pool.
    LinkManifest,
    /// Synchronize the immutable manifest pool.
    SynchronizeManifestPool,
    /// Write the complete canonical retention `head.next`.
    WriteHeadStage,
    /// Synchronize the retention `head.next`.
    SynchronizeHeadStage,
    /// Atomically replace the retention `HEAD`.
    ReplaceHead,
    /// Synchronize `retention` after head replacement.
    SynchronizeRetentionNamespace,
    /// Remove the retained `root.next`.
    RemoveRootStage,
    /// Remove the retained `manifest.next`.
    RemoveManifestStage,
    /// Synchronize `retention` after stage cleanup.
    SynchronizeCleanup,
}

impl RetentionPublicationPhase {
    /// Every publication phase in normative crash-boundary order.
    pub const ALL: [Self; 17] = [
        Self::WriteRootStage,
        Self::SynchronizeRootStage,
        Self::AdmitRootNamespace,
        Self::SynchronizeRootsAfterNamespace,
        Self::LinkRoot,
        Self::SynchronizeRootNamespace,
        Self::WriteManifestStage,
        Self::SynchronizeManifestStage,
        Self::LinkManifest,
        Self::SynchronizeManifestPool,
        Self::WriteHeadStage,
        Self::SynchronizeHeadStage,
        Self::ReplaceHead,
        Self::SynchronizeRetentionNamespace,
        Self::RemoveRootStage,
        Self::RemoveManifestStage,
        Self::SynchronizeCleanup,
    ];
}

impl fmt::Display for RetentionPublicationPhase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::WriteRootStage => "root-stage write",
            Self::SynchronizeRootStage => "root-stage synchronization",
            Self::AdmitRootNamespace => "root-namespace admission",
            Self::SynchronizeRootsAfterNamespace => "post-namespace roots synchronization",
            Self::LinkRoot => "immutable root link",
            Self::SynchronizeRootNamespace => "root-namespace synchronization",
            Self::WriteManifestStage => "manifest-stage write",
            Self::SynchronizeManifestStage => "manifest-stage synchronization",
            Self::LinkManifest => "immutable manifest link",
            Self::SynchronizeManifestPool => "manifest-pool synchronization",
            Self::WriteHeadStage => "retention-head-stage write",
            Self::SynchronizeHeadStage => "retention-head-stage synchronization",
            Self::ReplaceHead => "retention-head replacement",
            Self::SynchronizeRetentionNamespace => "retention-namespace synchronization",
            Self::RemoveRootStage => "retained root-stage removal",
            Self::RemoveManifestStage => "retained manifest-stage removal",
            Self::SynchronizeCleanup => "retention cleanup synchronization",
        })
    }
}
