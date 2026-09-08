//! This module owns one validated semantic retention root generation.

use super::{
    RegisteredRetentionProfile, RetentionAnchor, RetentionClosureLimits, RetentionNamespace,
    RetentionPolicy, RetentionRootDigest, RetentionRootError, RootGeneration,
};

/// One canonical namespace root generation before durable byte encoding.
///
/// Construction canonicalizes the caller's `Vec` in place, rejects duplicate
/// or excessive anchors, and consumes it into an immutable boxed slice. The
/// boxed-slice conversion may discard excess capacity.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetentionRoot {
    namespace: RetentionNamespace,
    generation: RootGeneration,
    policy: RetentionPolicy,
    predecessor: Option<RetentionRootDigest>,
    anchors: Box<[RetentionAnchor]>,
    anchor_count: u32,
}

impl RetentionRoot {
    /// Maximum anchors admitted in one namespace generation.
    pub const MAXIMUM_ANCHOR_COUNT: u32 = 65_536;

    /// Admits one deterministic semantic root.
    ///
    /// Anchors are sorted into canonical order. Duplicate anchors refuse
    /// instead of being silently removed.
    ///
    /// # Errors
    ///
    /// Returns a typed predecessor, anchor-count, or duplicate refusal.
    pub fn new(
        namespace: RetentionNamespace,
        generation: RootGeneration,
        policy: RetentionPolicy,
        predecessor: Option<RetentionRootDigest>,
        mut anchors: Vec<RetentionAnchor>,
    ) -> Result<Self, RetentionRootError> {
        admit_predecessor(generation, predecessor)?;
        let observed = anchors.len();
        let anchor_count =
            u32::try_from(observed).map_err(|_| RetentionRootError::AnchorCountExceeded {
                maximum: Self::MAXIMUM_ANCHOR_COUNT,
                observed,
            })?;
        if anchor_count > Self::MAXIMUM_ANCHOR_COUNT {
            return Err(RetentionRootError::AnchorCountExceeded {
                maximum: Self::MAXIMUM_ANCHOR_COUNT,
                observed,
            });
        }
        anchors.sort_unstable();
        refuse_duplicate(&anchors)?;
        Ok(Self {
            namespace,
            generation,
            policy,
            predecessor,
            anchors: anchors.into_boxed_slice(),
            anchor_count,
        })
    }

    /// Returns the exact opaque namespace.
    pub const fn namespace(&self) -> &RetentionNamespace {
        &self.namespace
    }

    /// Returns the per-namespace root generation.
    pub const fn generation(&self) -> RootGeneration {
        self.generation
    }

    /// Returns the registered realization profile.
    pub const fn profile(&self) -> RegisteredRetentionProfile {
        self.policy.profile()
    }

    /// Returns the admitted closure limits.
    pub const fn limits(&self) -> RetentionClosureLimits {
        self.policy.limits()
    }

    /// Returns the exact predecessor, absent only for generation one.
    #[must_use]
    pub const fn predecessor(&self) -> Option<RetentionRootDigest> {
        self.predecessor
    }

    /// Returns the canonical, duplicate-free anchors.
    pub fn anchors(&self) -> &[RetentionAnchor] {
        &self.anchors
    }

    /// Returns the bounded anchor count.
    #[must_use]
    pub const fn anchor_count(&self) -> u32 {
        self.anchor_count
    }
}

const fn admit_predecessor(
    generation: RootGeneration,
    predecessor: Option<RetentionRootDigest>,
) -> Result<(), RetentionRootError> {
    match (generation.get(), predecessor) {
        (1, Some(observed)) => {
            Err(RetentionRootError::InitialGenerationHasPredecessor { observed })
        }
        (1, None) | (_, Some(_)) => Ok(()),
        (_, None) => Err(RetentionRootError::MissingPredecessor { generation }),
    }
}

fn refuse_duplicate(anchors: &[RetentionAnchor]) -> Result<(), RetentionRootError> {
    let mut previous = None;
    for anchor in anchors {
        if previous == Some(*anchor) {
            return Err(RetentionRootError::DuplicateAnchor { anchor: *anchor });
        }
        previous = Some(*anchor);
    }
    Ok(())
}
