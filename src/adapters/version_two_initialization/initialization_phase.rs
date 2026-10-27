//! This module owns the ordered phases of direct version-two initialization.

use std::fmt;

/// One protocol directory created by initialization, in creation order.
///
/// A nested directory is created only after its parent, and every directory
/// is created only after the one before it in [`Self::ALL`].
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum VersionTwoInitializationDirectory {
    /// `staging`.
    Staging,
    /// `segments`.
    Segments,
    /// `catalogs`.
    Catalogs,
    /// `retention`.
    Retention,
    /// `retention/roots`.
    RetentionRoots,
    /// `retention/manifests`.
    RetentionManifests,
    /// `gc`.
    Gc,
    /// `recovery`.
    Recovery,
    /// `recovery/dispositions`.
    RecoveryDispositions,
}

impl VersionTwoInitializationDirectory {
    /// Every directory in creation order.
    pub const ALL: [Self; 9] = [
        Self::Staging,
        Self::Segments,
        Self::Catalogs,
        Self::Retention,
        Self::RetentionRoots,
        Self::RetentionManifests,
        Self::Gc,
        Self::Recovery,
        Self::RecoveryDispositions,
    ];

    /// Returns the directory's path relative to the store root.
    #[must_use]
    pub const fn path(self) -> &'static str {
        match self {
            Self::Staging => "staging",
            Self::Segments => "segments",
            Self::Catalogs => "catalogs",
            Self::Retention => "retention",
            Self::RetentionRoots => "retention/roots",
            Self::RetentionManifests => "retention/manifests",
            Self::Gc => "gc",
            Self::Recovery => "recovery",
            Self::RecoveryDispositions => "recovery/dispositions",
        }
    }

    /// Returns the directory's own name inside its parent.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::RetentionRoots => "roots",
            Self::RetentionManifests => "manifests",
            Self::RecoveryDispositions => "dispositions",
            other => other.path(),
        }
    }

    /// Returns the parent directory, or `None` for a root entry.
    #[must_use]
    pub const fn parent(self) -> Option<Self> {
        match self {
            Self::RetentionRoots | Self::RetentionManifests => Some(Self::Retention),
            Self::RecoveryDispositions => Some(Self::Recovery),
            _ => None,
        }
    }

    /// Returns the canonical children created inside this directory.
    pub(super) const fn children(self) -> &'static [Self] {
        match self {
            Self::Retention => &[Self::RetentionRoots, Self::RetentionManifests],
            Self::Recovery => &[Self::RecoveryDispositions],
            _ => &[],
        }
    }
}

/// One exact operation of direct version-two initialization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VersionTwoInitializationPhase {
    /// Open the root without following links and admit the platform profile.
    AdmitPlatform,
    /// Classify the root without mutation.
    ObservePrefix,
    /// Create or open `writer.lock`, lock it, and synchronize it and the root.
    OpenAndLockWriterFile,
    /// Remove an incomplete `FORMAT.next` left by process death and
    /// synchronize the root.
    DiscardIncompleteMarkerStage,
    /// Exclusively create `FORMAT.next` and write the canonical marker.
    WriteMarkerStage,
    /// Synchronize `FORMAT.next` and reverify its bytes and identity.
    SynchronizeMarkerStage,
    /// Synchronize the root so the stage name is durable.
    SynchronizeRootAfterMarkerStage,
    /// Create or admit zero-length `reader.lock` and synchronize the root.
    AdmitReaderFence,
    /// Create or admit one protocol directory and synchronize its parent.
    AdmitDirectory(VersionTwoInitializationDirectory),
    /// Hard-link `FORMAT.next` to `FORMAT` without replacement.
    LinkMarker,
    /// Synchronize the root after the link: the commit point.
    SynchronizeRootAfterMarker,
    /// Remove `FORMAT.next` after the commit is durable.
    RemoveMarkerStage,
    /// Synchronize the root after stage removal.
    SynchronizeRootAfterCleanup,
}

impl fmt::Display for VersionTwoInitializationPhase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AdmitDirectory(directory) => {
                write!(formatter, "directory admission of {}", directory.path())
            }
            other => formatter.write_str(other.label()),
        }
    }
}

impl VersionTwoInitializationPhase {
    const fn label(self) -> &'static str {
        match self {
            Self::AdmitPlatform => "platform admission",
            Self::ObservePrefix => "initialization prefix observation",
            Self::OpenAndLockWriterFile => "writer-lock admission",
            Self::DiscardIncompleteMarkerStage => "incomplete marker-stage discard",
            Self::WriteMarkerStage => "marker-stage write",
            Self::SynchronizeMarkerStage => "marker-stage synchronization",
            Self::SynchronizeRootAfterMarkerStage => "root synchronization after the marker stage",
            Self::AdmitReaderFence => "reader-fence admission",
            Self::AdmitDirectory(_) => "directory admission",
            Self::LinkMarker => "marker link",
            Self::SynchronizeRootAfterMarker => "commit synchronization",
            Self::RemoveMarkerStage => "marker-stage removal",
            Self::SynchronizeRootAfterCleanup => "root synchronization after cleanup",
        }
    }
}
