//! This module owns documentation-run closure regression evidence.

use std::io;

use crate::documentation_integrity::DocumentationError;

use super::finish_run;

#[test]
fn tool_and_snapshot_cleanup_failures_are_both_reported() {
    let result = finish_run(
        Err(DocumentationError::ToolFailed {
            program: "markdownlint-cli2",
            code: Some(1),
            stdout: String::from("tool failure"),
            stderr: String::new(),
        }),
        Err(DocumentationError::Snapshot {
            action: "remove documentation snapshot",
            source: io::Error::other("cleanup failure"),
        }),
    );

    assert!(matches!(
        result,
        Err(DocumentationError::CheckFailures { first, second })
            if matches!(
                *first,
                DocumentationError::ToolFailed {
                    program: "markdownlint-cli2",
                    code: Some(1),
                    ref stdout,
                    ref stderr,
                } if stdout == "tool failure" && stderr.is_empty()
            ) && matches!(
                *second,
                DocumentationError::Snapshot {
                    action: "remove documentation snapshot",
                    ..
                }
            )
    ));
}
