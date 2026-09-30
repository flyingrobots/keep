//! This module owns the typed retention recovery plan and its outcome.

/// One ordered recovery effect. Each maps to exactly one storage capability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionRecoveryStep {
    /// Remove a truncated `head.next` whose replacement never happened.
    DiscardHeadStage,
    /// Remove a truncated `manifest.next` that was never linked.
    DiscardManifestStage,
    /// Remove a truncated `root.next` that was never linked.
    DiscardRootStage,
    /// Admit the namespace directory and link the complete root stage into it.
    LinkRoot,
    /// Link the complete manifest stage into the manifest pool.
    LinkManifest,
    /// Replace `retention/HEAD` with the complete head stage and synchronize.
    FinalizeHead,
    /// Remove the retained root stage after its pool link is proven.
    RemoveRootStage,
    /// Remove the retained manifest stage after its pool link is proven.
    RemoveManifestStage,
}

/// The state recovery leaves once every step has executed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionRecoveryOutcome {
    /// No stage remains; forward publication may proceed.
    Clean,
    /// The staged generation is (or was already) the published head and its
    /// stages are removed; forward publication may proceed.
    Committed,
    /// Complete stages remain linked and retained as valid orphans. They are
    /// recovery-protected until explicit disposition; forward publication
    /// refuses meanwhile.
    Protected {
        /// `root.next` remains retained.
        root_stage: bool,
        /// `manifest.next` remains retained.
        manifest_stage: bool,
    },
}

/// The ordered effects recovery must execute and the state they produce.
#[derive(Clone, Debug, Eq, PartialEq)]
#[must_use]
pub struct RetentionRecoveryPlan {
    steps: Vec<RetentionRecoveryStep>,
    outcome: RetentionRecoveryOutcome,
}

impl RetentionRecoveryPlan {
    pub(super) const fn new(
        steps: Vec<RetentionRecoveryStep>,
        outcome: RetentionRecoveryOutcome,
    ) -> Self {
        Self { steps, outcome }
    }

    /// The effects in execution order; empty when nothing must change.
    #[must_use]
    pub fn steps(&self) -> &[RetentionRecoveryStep] {
        &self.steps
    }

    /// The state the store is in after every step executes.
    #[must_use]
    pub const fn outcome(&self) -> RetentionRecoveryOutcome {
        self.outcome
    }
}
