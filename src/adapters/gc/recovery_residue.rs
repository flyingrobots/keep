//! This boundary module owns what restart observes in `gc` and the segment
//! pool before planning GC recovery.

/// The exact bytes and presence restart read, nothing interpreted.
#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GcResidue {
    /// `gc/intent.next`, when present, up to one byte past the maximum
    /// intent length.
    pub intent_stage: Option<Box<[u8]>>,
    /// `gc/intent`, when present.
    pub intent: Option<Box<[u8]>>,
    /// `gc/receipt.next`, when present.
    pub receipt_stage: Option<Box<[u8]>>,
    /// `gc/receipt`, when present.
    pub receipt: Option<Box<[u8]>>,
    /// For each candidate the durable intent names, in canonical order,
    /// whether its segment-pool entry is present. Empty without an intent.
    pub candidates_present: Vec<bool>,
}

impl GcResidue {
    /// An empty `gc` directory.
    pub const IDLE: Self = Self {
        intent_stage: None,
        intent: None,
        receipt_stage: None,
        receipt: None,
        candidates_present: Vec::new(),
    };
}
