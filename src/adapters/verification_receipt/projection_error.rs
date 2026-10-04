//! This module owns precise failures projecting live evidence into receipt v1.

use crate::{VerificationDepth, VerificationSubject};
use std::{error::Error, fmt};

/// Receipt v1 cannot truthfully represent the supplied operation evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum VerificationReceiptProjectionError {
    /// The subject has no registered v1 representation.
    Subject(VerificationSubject),
    /// The depth has no registered v1 code.
    Depth(VerificationDepth),
    /// The supported set is empty or has holes in v1's wire interval.
    SupportedSetNotRepresentable,
    /// The supplied view disagrees with the operation's provenance.
    ViewMismatch,
    /// No admitted layout/target/work details accompany the report.
    LayoutEvidenceRequired,
    /// No admitted request/stage evidence accompanies the refusal.
    RefusalContextRequired,
    /// Operational failure is not a verification refusal.
    OperationalFailure,
    /// An evidence index cannot fit v1's exact index slot.
    IndexOverflow,
}

impl fmt::Display for VerificationReceiptProjectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "verification receipt v1 projection refused: {self:?}")
    }
}
impl Error for VerificationReceiptProjectionError {}
