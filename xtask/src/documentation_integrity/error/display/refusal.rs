//! This module owns human-readable documentation-refusal diagnostics.

use std::fmt;

use super::super::DocumentationError;

pub(super) fn refusal_fixture(formatter: &mut fmt::Formatter<'_>, action: &str) -> fmt::Result {
    write!(
        formatter,
        "cannot {action} for documentation refusal evidence"
    )
}

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
