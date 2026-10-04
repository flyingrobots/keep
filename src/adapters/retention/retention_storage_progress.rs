//! This module owns reported namespace effects of a failing retention storage operation.

/// The fallible boundary at which a retention storage capability stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RetentionStorageBoundary {
    /// Exclusive creation of the stage pathname.
    StageCreation,
    /// The newly created stage handle identity was recorded.
    StageIdentity,
    /// Stage contents were written.
    StageWrite,
    /// Staged writes were flushed without establishing directory durability.
    StageFlush,
    /// A receipt stage was renamed onto the published receipt.
    ReceiptRename,
    /// The replaced receipt identity and exact bytes were checked.
    ReceiptVerification,
    /// No active recovery context or required stage was available.
    RecoveryContext,
    /// The retained source handle and pathname were checked.
    SourceVerification,
    /// Staged file contents were synchronized.
    StageSynchronization,
    /// A root namespace directory was created.
    NamespaceCreation,
    /// The root namespace directory was opened without following links.
    NamespaceOpen,
    /// The roots directory was synchronized after namespace creation.
    RootsSynchronization,
    /// A pool hard link was attempted without replacement.
    PoolLink,
    /// The pool target's kind, identity and exact bytes were checked.
    PoolVerification,
    /// A pool directory was synchronized.
    PoolSynchronization,
    /// The source stage pathname was unlinked.
    StageUnlink,
    /// The source stage pathname was checked for absence.
    StageAbsence,
    /// The head stage was renamed onto the published head.
    HeadRename,
    /// The replaced head's identity and exact bytes were checked.
    HeadVerification,
    /// The retention directory was synchronized after a namespace effect.
    RetentionSynchronization,
}

/// A namespace effect whose occurrence or attempted occurrence is reported.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionNamespaceEffect {
    /// A stage pathname was created; this does not imply complete contents.
    StageCreated,
    /// A published receipt pathname was replaced.
    ReceiptReplaced,
    /// A new root namespace directory was created.
    NamespaceCreated,
    /// A new immutable pool link was created.
    PoolLinkCreated,
    /// The published head pathname was replaced.
    HeadReplaced,
    /// A retained stage pathname was removed.
    StageRemoved,
}

/// Whether the directory synchronization covering an observed effect completed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionEffectDurability {
    /// The covering directory synchronization has not completed successfully.
    Unconfirmed,
    /// The covering directory synchronization completed successfully.
    Synchronized,
}

/// One effect positively observed by the failing capability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetentionKnownEffect {
    pub(super) effect: RetentionNamespaceEffect,
    pub(super) durability: RetentionEffectDurability,
}

impl RetentionKnownEffect {
    /// The namespace effect known to have occurred.
    #[must_use]
    pub const fn effect(self) -> RetentionNamespaceEffect {
        self.effect
    }
    /// The observed synchronization status; failure never implies rollback.
    #[must_use]
    pub const fn durability(self) -> RetentionEffectDurability {
        self.durability
    }
}

/// What one failing capability established before returning its error.
///
/// Only namespace effects are listed; this is not a complete syscall trace.
/// An empty known list does not exclude the separately reported uncertain effect.
#[derive(Debug)]
pub struct RetentionStorageProgress {
    pub(super) boundary: RetentionStorageBoundary,
    pub(super) known: Vec<RetentionKnownEffect>,
    pub(super) uncertain: Option<RetentionNamespaceEffect>,
}

impl RetentionStorageProgress {
    /// The exact failing boundary within the capability.
    #[must_use]
    pub const fn boundary(&self) -> RetentionStorageBoundary {
        self.boundary
    }
    /// Positively observed namespace effects, with their synchronization status.
    #[must_use]
    pub fn known_effects(&self) -> &[RetentionKnownEffect] {
        &self.known
    }
    /// An attempted effect whose occurrence could not be established from the failed operation.
    #[must_use]
    pub const fn uncertain_effect(&self) -> Option<RetentionNamespaceEffect> {
        self.uncertain
    }
}
