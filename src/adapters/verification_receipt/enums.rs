//! This boundary module owns the registered enumerations a verification
//! receipt encodes and the depth codes it shares with the verification
//! vocabulary.

use crate::VerificationDepth;

macro_rules! registered {
    ($(#[$doc:meta])* $name:ident { $($(#[$variant_doc:meta])* $variant:ident = $code:literal,)* }) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $name {
            $($(#[$variant_doc])* $variant,)*
        }

        impl $name {
            /// Every registered value in code order.
            pub const ALL: &'static [Self] = &[$(Self::$variant,)*];

            /// The registered wire code.
            #[must_use]
            pub const fn code(self) -> u16 {
                match self {
                    $(Self::$variant => $code,)*
                }
            }

            /// The value registered under `code`, if any.
            #[must_use]
            pub const fn from_code(code: u16) -> Option<Self> {
                match code {
                    $($code => Some(Self::$variant),)*
                    _ => None,
                }
            }
        }
    };
}

registered! {
    /// Whether the receipt records an established depth or a refusal.
    ReceiptOutcomeKind {
        /// A `VerificationReport`: the depth was established.
        Report = 1,
        /// A `VerificationRefusal`: the stage could not be established.
        Refusal = 2,
    }
}

registered! {
    /// What the subject identity slot holds.
    ReceiptSubjectKind {
        /// A 59-byte canonical `BlobId` binary, zero padded to 60.
        Blob = 1,
        /// A 60-byte canonical `LayoutId` binary.
        Layout = 2,
    }
}

registered! {
    /// Which view the verification ran against.
    ReceiptViewKind {
        /// The non-durable reference store: no catalog or retention.
        Reference = 1,
        /// One admitted durable snapshot with catalog and retention coordinates.
        Durable = 2,
    }
}

registered! {
    /// The refusal classification; `None` for a report.
    ReceiptRefusalClass {
        /// Not a refusal.
        None = 0,
        /// Required evidence is absent from a complete view.
        Missing = 1,
        /// Present evidence contradicts the identity it must reproduce.
        Corrupt = 2,
        /// Admitted evidence conflicts.
        Ambiguous = 3,
        /// The view cannot establish the requested depth at all.
        Unsupported = 4,
    }
}

registered! {
    /// Which evidence the refusal names; `None` unless missing or corrupt.
    ReceiptEvidenceKind {
        /// No evidence coordinate.
        None = 0,
        /// Missing: no committed layout names the blob. Corrupt: a chunk does
        /// not hash to the identity the layout names.
        First = 1,
        /// Missing: the exact layout is not committed. Corrupt: the layout does
        /// not produce the identity it is keyed by.
        Second = 2,
        /// Missing: the layout names a chunk the view lacks. Corrupt: the
        /// authenticated chunks do not reproduce the target blob.
        Third = 3,
        /// Corrupt only: profile replay diverged at a boundary.
        Fourth = 4,
    }
}

/// The registered code of one verification depth: its one-based position
/// in `VerificationDepth::ALL`.
#[must_use]
pub(super) fn depth_code(depth: VerificationDepth) -> u16 {
    VerificationDepth::ALL
        .iter()
        .position(|candidate| *candidate == depth)
        .and_then(|index| u16::try_from(index).ok())
        .map_or(0, |index| index.saturating_add(1))
}

/// The depth registered under `code`, if any.
#[must_use]
pub(super) fn depth_from_code(code: u16) -> Option<VerificationDepth> {
    let index = usize::from(code.checked_sub(1)?);
    VerificationDepth::ALL.get(index).copied()
}
