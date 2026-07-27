//! Repository verification and pure protocol-admission surfaces.

#![deny(warnings)]
#![forbid(unsafe_code)]

#[cfg(feature = "golden-protocol-fuzz")]
extern crate self as xtask;

#[cfg(feature = "golden-protocol-fuzz")]
mod diagnostic;

#[cfg(feature = "golden-protocol-fuzz")]
#[allow(
    clippy::redundant_pub_crate,
    reason = "the library facade deliberately hides the parser implementation"
)]
mod golden_protocol_fuzz;

pub mod protocol_admission;

/// Whether one bounded Golden File Worldline production parser admitted input.
#[cfg(feature = "golden-protocol-fuzz")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoldenProtocolAdmission {
    /// The selected production parser admitted the input.
    Admitted,
    /// The selected production parser refused the input.
    Refused,
}

/// Exercise one bounded production parser from the Golden File Worldline corpus.
///
/// Every selector value maps deterministically to a table, scalar,
/// invalid-identity, or mutation parser. This entry point performs no I/O and
/// exists so fuzzing exercises the exact parser implementations used by the
/// repository checker.
#[cfg(feature = "golden-protocol-fuzz")]
#[must_use]
pub fn admit_golden_protocol(selector: u8, input: &[u8]) -> GoldenProtocolAdmission {
    if golden_protocol_fuzz::admit(selector, input).is_ok() {
        GoldenProtocolAdmission::Admitted
    } else {
        GoldenProtocolAdmission::Refused
    }
}
