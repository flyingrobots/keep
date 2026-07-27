use std::path::Path;

use super::super::{GoldenError, MAX_TABLE_BYTES, protocol_lines_from_bytes, table_rows};

const SCHEMA: &str = "# keep.cases/v1";
const COLUMNS: [&str; 2] = ["case", "value"];

#[test]
fn malformed_protocol_framing_has_exact_refusals() {
    let cases = [
        FramingRefusal {
            name: "empty",
            bytes: b"",
            expected: "cases.tsv: protocol must use final-LF-only framing",
        },
        FramingRefusal {
            name: "missing-final-lf",
            bytes: b"# keep.cases/v1",
            expected: "cases.tsv: protocol must use final-LF-only framing",
        },
        FramingRefusal {
            name: "carriage-return",
            bytes: b"# keep.cases/v1\r\n",
            expected: "cases.tsv: protocol must use final-LF-only framing",
        },
        FramingRefusal {
            name: "blank-line",
            bytes: b"# keep.cases/v1\n\n",
            expected: "cases.tsv: protocol contains a blank line",
        },
    ];
    for case in cases {
        let result = protocol_lines_from_bytes(Path::new("cases.tsv"), case.bytes);
        assert!(
            matches!(
                result,
                Err(GoldenError::Violation(ref message)) if message == case.expected
            ),
            "framing case moved: {}",
            case.name
        );
    }
}

#[test]
fn malformed_protocol_utf8_has_a_typed_refusal() {
    let result = protocol_lines_from_bytes(Path::new("cases.tsv"), &[u8::MAX, b'\n']);
    assert!(matches!(
        result,
        Err(GoldenError::Utf8 { ref path, .. }) if path == Path::new("cases.tsv")
    ));
}

#[test]
fn oversized_protocol_is_refused_before_decoding() {
    let input = vec![b'a'; MAX_TABLE_BYTES.saturating_add(1)];
    let result = protocol_lines_from_bytes(Path::new("cases.tsv"), &input);
    assert!(matches!(
        result,
        Err(GoldenError::Violation(ref message))
            if message == "cases.tsv: file exceeds 1048576 bytes"
    ));
}

#[test]
fn malformed_table_structure_has_exact_refusals() {
    let cases = [
        TableRefusal {
            name: "wrong-schema",
            lines: &["# keep.cases/v2", "case\tvalue", "example\t00"],
            expected: "cases.tsv: unsupported schema or empty table",
        },
        TableRefusal {
            name: "schema-only",
            lines: &[SCHEMA],
            expected: "cases.tsv: unsupported schema or empty table",
        },
        TableRefusal {
            name: "wrong-columns",
            lines: &[SCHEMA, "value\tcase", "00\texample"],
            expected: "cases.tsv: unexpected columns",
        },
        TableRefusal {
            name: "no-data",
            lines: &[SCHEMA, "case\tvalue"],
            expected: "cases.tsv: table has no data rows",
        },
        TableRefusal {
            name: "short-row",
            lines: &[SCHEMA, "case\tvalue", "example"],
            expected: "cases.tsv:3: malformed field count",
        },
        TableRefusal {
            name: "long-row",
            lines: &[SCHEMA, "case\tvalue", "example\t00\textra"],
            expected: "cases.tsv:3: malformed field count",
        },
    ];
    for case in cases {
        let result = table_rows(
            "cases.tsv",
            SCHEMA,
            &COLUMNS,
            case.lines.iter().map(|line| (*line).to_owned()).collect(),
        );
        assert!(
            matches!(
                result,
                Err(GoldenError::Violation(ref message)) if message == case.expected
            ),
            "table case moved: {}",
            case.name
        );
    }
}

struct FramingRefusal {
    name: &'static str,
    bytes: &'static [u8],
    expected: &'static str,
}

struct TableRefusal {
    name: &'static str,
    lines: &'static [&'static str],
    expected: &'static str,
}
