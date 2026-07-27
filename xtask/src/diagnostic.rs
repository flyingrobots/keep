//! This module owns stable single-line rendering of untrusted diagnostic fields.

#![allow(
    clippy::redundant_pub_crate,
    reason = "multiple private sibling modules share the renderer"
)]

use std::fmt::{self, Write as _};
use std::path::Path;

pub(crate) fn escaped_controls(
    formatter: &mut fmt::Formatter<'_>,
    diagnostic: &str,
) -> fmt::Result {
    for character in diagnostic.chars() {
        if character.is_control() {
            for escaped in character.escape_default() {
                formatter.write_char(escaped)?;
            }
        } else {
            formatter.write_char(character)?;
        }
    }
    Ok(())
}

pub(crate) fn escaped_path(formatter: &mut fmt::Formatter<'_>, path: &Path) -> fmt::Result {
    escaped_controls(formatter, &path.to_string_lossy())
}
