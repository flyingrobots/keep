//! This boundary module owns the ordered durable phases of one GC
//! retirement execution, `KEEP-CRASH-074` through `KEEP-CRASH-087`.

use std::fmt;

/// One durable transition of GC execution, in protocol order.
///
/// The intent travels the fixed-stage protocol into `gc/intent`; every
/// candidate is then unlinked in canonical digest order with a segment-pool
/// synchronization after each unlink; the receipt is staged, synchronized,
/// and renamed onto `gc/receipt` (replacing the prior retirement's receipt
/// atomically, as `retention/HEAD` is replaced); finally the completed
/// intent is removed and `gc` synchronized.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GcExecutionPhase {
    /// Write the complete canonical intent to `gc/intent.next`.
    WriteIntentStage,
    /// Synchronize `gc/intent.next`.
    SynchronizeIntentStage,
    /// Link the stage to `gc/intent` without replacement.
    LinkIntent,
    /// Synchronize `gc` after the intent link.
    SynchronizeGcAfterIntent,
    /// Remove the retained `gc/intent.next`.
    RemoveIntentStage,
    /// Synchronize `gc` after intent-stage cleanup.
    SynchronizeGcAfterIntentCleanup,
    /// Reopen, verify, and unlink one candidate (one occurrence per candidate).
    UnlinkCandidate,
    /// Synchronize `segments` after one unlink (one occurrence per candidate).
    SynchronizeSegmentPool,
    /// Write the complete canonical receipt to `gc/receipt.next`.
    WriteReceiptStage,
    /// Synchronize `gc/receipt.next`.
    SynchronizeReceiptStage,
    /// Rename the stage onto `gc/receipt`, replacing any prior receipt.
    ReplaceReceipt,
    /// Synchronize `gc` after the receipt replacement.
    SynchronizeGcAfterReceipt,
    /// Remove the completed `gc/intent`.
    RemoveIntent,
    /// Synchronize `gc` after the intent removal.
    SynchronizeGcAfterIntentRemoval,
}

impl GcExecutionPhase {
    /// Every phase in execution order.
    pub const ALL: [Self; 14] = [
        Self::WriteIntentStage,
        Self::SynchronizeIntentStage,
        Self::LinkIntent,
        Self::SynchronizeGcAfterIntent,
        Self::RemoveIntentStage,
        Self::SynchronizeGcAfterIntentCleanup,
        Self::UnlinkCandidate,
        Self::SynchronizeSegmentPool,
        Self::WriteReceiptStage,
        Self::SynchronizeReceiptStage,
        Self::ReplaceReceipt,
        Self::SynchronizeGcAfterReceipt,
        Self::RemoveIntent,
        Self::SynchronizeGcAfterIntentRemoval,
    ];

    /// Whether the phase occurs once per candidate.
    #[must_use]
    pub const fn per_candidate(self) -> bool {
        matches!(self, Self::UnlinkCandidate | Self::SynchronizeSegmentPool)
    }

    /// The stable `transitions.tsv` operation name.
    #[must_use]
    pub const fn operation(self) -> &'static str {
        match self {
            Self::WriteIntentStage => "write-intent-stage",
            Self::SynchronizeIntentStage => "sync-intent-stage",
            Self::LinkIntent => "link-intent",
            Self::SynchronizeGcAfterIntent => "sync-gc-after-intent",
            Self::RemoveIntentStage => "remove-intent-stage",
            Self::SynchronizeGcAfterIntentCleanup => "sync-gc-after-intent-cleanup",
            Self::UnlinkCandidate => "unlink-candidate",
            Self::SynchronizeSegmentPool => "sync-segment-pool",
            Self::WriteReceiptStage => "write-receipt-stage",
            Self::SynchronizeReceiptStage => "sync-receipt-stage",
            Self::ReplaceReceipt => "replace-receipt",
            Self::SynchronizeGcAfterReceipt => "sync-gc-after-receipt",
            Self::RemoveIntent => "remove-intent",
            Self::SynchronizeGcAfterIntentRemoval => "sync-gc-after-intent-removal",
        }
    }
}

impl fmt::Display for GcExecutionPhase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.operation())
    }
}

/// One phase at one candidate index: the unit execution resumes from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GcExecutionPoint {
    /// The phase.
    pub phase: GcExecutionPhase,
    /// The candidate index for a per-candidate phase; zero otherwise.
    pub candidate: usize,
}

impl GcExecutionPoint {
    /// The first point of a fresh execution.
    pub const START: Self = Self::at(GcExecutionPhase::WriteIntentStage);

    /// The point at a phase that is not per candidate.
    #[must_use]
    pub const fn at(phase: GcExecutionPhase) -> Self {
        Self {
            phase,
            candidate: 0,
        }
    }
}

impl fmt::Display for GcExecutionPoint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.phase.per_candidate() {
            write!(formatter, "{}[{}]", self.phase, self.candidate)
        } else {
            fmt::Display::fmt(&self.phase, formatter)
        }
    }
}
