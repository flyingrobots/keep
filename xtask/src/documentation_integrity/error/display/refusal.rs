//! This module owns human-readable documentation-refusal diagnostics.

use std::fmt;

use super::super::DocumentationError;

/// Formats one failed filesystem action at the refusal-fixture boundary.
///
/// `action` names the attempted operation without terminal punctuation. The
/// formatter receives no underlying I/O details; those remain available through
/// the error source chain.
pub(super) fn refusal_fixture(formatter: &mut fmt::Formatter<'_>, action: &str) -> fmt::Result {
    write!(
        formatter,
        "cannot {action} for documentation refusal evidence"
    )
}

/// Formats a refusal scenario that did not return its exact reviewed error.
///
/// The stable prefix identifies `scenario`. When `observed` contains a different
/// typed failure, its escaped display diagnostic is appended as secondary
/// evidence; normal absence records that the malformed input was accepted.
pub(super) fn refusal_mismatch(
    formatter: &mut fmt::Formatter<'_>,
    scenario: &str,
    observed: Option<&DocumentationError>,
) -> fmt::Result {
    write!(
        formatter,
        "documentation refusal scenario `{scenario}` did not produce its reviewed error"
    )?;
    if let Some(error) = observed {
        write!(formatter, ": {error}")?;
    }
    Ok(())
}
