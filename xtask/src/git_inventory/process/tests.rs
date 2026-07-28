//! This module owns adversarial Git diagnostic-bound tests.

use std::io::Cursor;

use super::{GitInventoryError, git_failure};
use crate::process_output::bounded_bytes;

#[test]
fn git_diagnostic_encoding_failure_retains_exit_status() {
    let error = git_failure("test diagnostics", Some(9), vec![u8::MAX]);
    assert!(matches!(
        error,
        GitInventoryError::DiagnosticEncoding {
            operation: "test diagnostics",
            code: Some(9),
            ..
        }
    ));
}

#[test]
fn git_diagnostics_are_drained_but_only_the_bound_is_retained() {
    let result = bounded_bytes(Cursor::new(b"abcdef"), 3);
    assert!(matches!(
        result,
        Ok(ref diagnostic) if diagnostic.bytes == b"abc" && diagnostic.exceeded
    ));
}
