//! This module owns process-death positions around one durability transition.

/// The process-death position relative to one durability transition.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DurabilityCrashPosition {
    /// Process death occurs before the transition begins.
    Before,
    /// Process death occurs while the transition is incomplete.
    During,
    /// Process death occurs after the transition completes.
    After,
}

impl DurabilityCrashPosition {
    /// Every position in canonical matrix order.
    pub const ALL: [Self; 3] = [Self::Before, Self::During, Self::After];

    /// Returns the stable identifier used by the crash-matrix protocol.
    #[must_use]
    pub const fn identifier(self) -> &'static str {
        match self {
            Self::Before => "before",
            Self::During => "during",
            Self::After => "after",
        }
    }

    /// Parses one exact crash-position identifier.
    #[must_use]
    pub fn from_identifier(identifier: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|position| position.identifier() == identifier)
    }
}
