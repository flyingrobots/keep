//! This boundary module owns the observed residue of one interrupted
//! migration.

/// Names of the version-2 directory prefix in creation order, after
/// `reader.lock`.
pub const MIGRATION_NAMESPACE_PREFIX: [&str; 6] = [
    "retention",
    "retention/roots",
    "retention/manifests",
    "gc",
    "recovery",
    "recovery/dispositions",
];

/// Everything a migration may have left behind, as one observation.
///
/// An observer reads each fixed name without following links and refuses a
/// wrong file kind, a link, or an unknown entry before producing this value,
/// so the planner sees only bytes and presence. Byte fields carry exactly the
/// bytes observed, bounded by the observer to one byte more than the record's
/// canonical length so overlong records stay distinguishable.
#[must_use]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StoreMigrationResidue {
    /// Bytes of `migration.intent.next`, when present.
    pub intent_stage: Option<Vec<u8>>,
    /// Bytes of `migration.intent`, when present.
    pub intent: Option<Vec<u8>>,
    /// Whether the persistent `reader.lock` exists.
    pub reader_fence: bool,
    /// Presence of each directory in [`MIGRATION_NAMESPACE_PREFIX`] order.
    pub namespace_prefix: [bool; 6],
    /// Bytes of `FORMAT.next`, when present.
    pub marker_stage: Option<Vec<u8>>,
    /// Bytes of `FORMAT`, when present.
    pub marker: Option<Vec<u8>>,
    /// Bytes of `migration.receipt.next`, when present.
    pub receipt_stage: Option<Vec<u8>>,
    /// Bytes of `migration.receipt`, when present.
    pub receipt: Option<Vec<u8>>,
}

impl StoreMigrationResidue {
    /// The observation of a store with no migration artifact at all.
    pub const VERSION_ONE: Self = Self {
        intent_stage: None,
        intent: None,
        reader_fence: false,
        namespace_prefix: [false; 6],
        marker_stage: None,
        marker: None,
        receipt_stage: None,
        receipt: None,
    };

    /// Whether `reader.lock` or any prefix directory exists.
    #[must_use]
    pub fn has_namespace_effect(&self) -> bool {
        self.reader_fence || self.namespace_prefix.iter().any(|present| *present)
    }

    /// Whether any marker or receipt artifact or stage exists.
    #[must_use]
    pub const fn has_marker_or_receipt_effect(&self) -> bool {
        self.marker_stage.is_some()
            || self.marker.is_some()
            || self.receipt_stage.is_some()
            || self.receipt.is_some()
    }

    /// Whether any receipt artifact or stage exists.
    #[must_use]
    pub const fn has_receipt_effect(&self) -> bool {
        self.receipt_stage.is_some() || self.receipt.is_some()
    }

    /// Length of the contiguous present prefix of `reader.lock` followed by
    /// the six directories.
    ///
    /// # Errors
    ///
    /// Returns the index of the first absent slot and the index of a later
    /// present slot when the prefix is not contiguous.
    pub fn namespace_extent(&self) -> Result<usize, (usize, usize)> {
        let slots = std::iter::once(self.reader_fence).chain(self.namespace_prefix);
        let mut first_absent = None;
        for (index, present) in slots.enumerate() {
            match (first_absent, present) {
                (None, false) => first_absent = Some(index),
                (Some(absent), true) => return Err((absent, index)),
                _ => {}
            }
        }
        Ok(first_absent.unwrap_or(7))
    }
}
