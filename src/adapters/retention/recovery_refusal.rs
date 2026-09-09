//! This module owns the typed refusals of retention recovery planning.

use std::error::Error;
use std::fmt;

use super::{RetentionHeadDecodeError, RetentionManifestDecodeError, RetentionRootDecodeError};

/// One of the three fixed retention stage names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionFixedStage {
    /// `retention/root.next`.
    Root,
    /// `retention/manifest.next`.
    Manifest,
    /// `retention/head.next`.
    Head,
}

/// One of the two immutable retention pools.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetentionPool {
    /// `retention/roots/<namespace>`.
    Roots,
    /// `retention/manifests`.
    Manifests,
}

/// Why the observed stages are unrecoverable ambiguity rather than a plan.
///
/// Every variant leaves the store untouched: recovery refuses before any
/// effect, and the evidence stays for explicit disposition.
#[derive(Debug)]
#[non_exhaustive]
pub enum RetentionRecoveryRefusal {
    /// A stage is complete enough to judge and fails a canonical law.
    StageCorrupt {
        /// The stage that failed.
        stage: RetentionFixedStage,
        /// The exact decode refusal.
        source: Box<dyn Error + Send + Sync + 'static>,
    },
    /// A truncated stage has a later-ordered effect, so it is not pre-effect.
    TruncatedStageWithLaterEffect {
        /// The truncated stage.
        stage: RetentionFixedStage,
    },
    /// A complete stage names a pool entry that exists with other bytes.
    PoolEntryDiffers {
        /// The pool holding the conflicting entry.
        pool: RetentionPool,
    },
    /// A complete head stage exists without a complete manifest stage.
    HeadStageWithoutManifestStage,
    /// The head stage names a manifest other than the staged one.
    HeadStageNamesOtherManifest,
    /// The head stage's predecessor is not the published manifest.
    HeadPredecessorMismatch,
    /// The head stage exists but the staged manifest was never linked.
    ManifestNotLinkedBeforeHead,
    /// The head stage exists but the staged root was never linked.
    RootNotLinkedBeforeHead,
    /// A complete manifest stage exists without the root stage it introduces
    /// and is not the published manifest.
    ManifestStageWithoutRootStage,
    /// The manifest stage does not select the staged root.
    ManifestStageNamesOtherRoot,
    /// The manifest stage is not the exact successor of the published manifest.
    ManifestNotSuccessor,
    /// The root stage is not the exact successor of the namespace's current root.
    RootNotSuccessor,
}

impl RetentionRecoveryRefusal {
    pub(super) fn corrupt_root(source: RetentionRootDecodeError) -> Self {
        Self::StageCorrupt {
            stage: RetentionFixedStage::Root,
            source: Box::new(source),
        }
    }

    pub(super) fn corrupt_manifest(source: RetentionManifestDecodeError) -> Self {
        Self::StageCorrupt {
            stage: RetentionFixedStage::Manifest,
            source: Box::new(source),
        }
    }

    pub(super) fn corrupt_head(source: RetentionHeadDecodeError) -> Self {
        Self::StageCorrupt {
            stage: RetentionFixedStage::Head,
            source: Box::new(source),
        }
    }
}

impl fmt::Display for RetentionFixedStage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Root => "root.next",
            Self::Manifest => "manifest.next",
            Self::Head => "head.next",
        })
    }
}

impl fmt::Display for RetentionPool {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Roots => "retention/roots",
            Self::Manifests => "retention/manifests",
        })
    }
}

impl fmt::Display for RetentionRecoveryRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StageCorrupt { stage, .. } => {
                write!(formatter, "retention stage {stage} is corrupt")
            }
            Self::TruncatedStageWithLaterEffect { stage } => write!(
                formatter,
                "truncated retention stage {stage} has a later-ordered effect"
            ),
            Self::PoolEntryDiffers { pool } => {
                write!(
                    formatter,
                    "{pool} holds a different entry under the staged name"
                )
            }
            other => formatter.write_str(other.message()),
        }
    }
}

impl RetentionRecoveryRefusal {
    const fn message(&self) -> &'static str {
        match self {
            Self::HeadStageWithoutManifestStage => {
                "head.next exists without a complete manifest.next"
            }
            Self::HeadStageNamesOtherManifest => {
                "head.next names a manifest other than manifest.next"
            }
            Self::HeadPredecessorMismatch => "head.next does not succeed the published manifest",
            Self::ManifestNotLinkedBeforeHead => {
                "head.next exists but manifest.next was never linked"
            }
            Self::RootNotLinkedBeforeHead => "head.next exists but root.next was never linked",
            Self::ManifestStageWithoutRootStage => {
                "manifest.next exists without root.next and is not the published manifest"
            }
            Self::ManifestStageNamesOtherRoot => "manifest.next does not select root.next",
            Self::ManifestNotSuccessor => {
                "manifest.next is not the successor of the published manifest"
            }
            Self::RootNotSuccessor => {
                "root.next is not the successor of its namespace's current root"
            }
            Self::StageCorrupt { .. }
            | Self::TruncatedStageWithLaterEffect { .. }
            | Self::PoolEntryDiffers { .. } => "retention recovery refused",
        }
    }
}

impl Error for RetentionRecoveryRefusal {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::StageCorrupt { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}
