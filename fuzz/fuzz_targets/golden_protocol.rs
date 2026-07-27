#![no_main]

//! This target owns bounded repository-protocol admission fuzzing.

use libfuzzer_sys::fuzz_target;
use xtask::protocol_admission::{
    EmptyHex, decode_lower_hex, framed_lines, posix_relative_path, tab_fields,
};

const MAX_FIELD_BYTES: usize = 4_096;
const MAX_PROTOCOL_BYTES: usize = 1_048_576;

fuzz_target!(|bytes: &[u8]| {
    if let Ok(lines) = framed_lines(bytes, MAX_PROTOCOL_BYTES) {
        let reconstructed = format!("{}\n", lines.join("\n"));
        assert_eq!(reconstructed.as_bytes(), bytes);
        for line in lines {
            let _ = tab_fields(&line, 7);
        }
    }
    if bytes.len() <= MAX_FIELD_BYTES
        && let Ok(value) = std::str::from_utf8(bytes)
    {
        if let Ok(decoded) = decode_lower_hex(value, 64, EmptyHex::Allow) {
            assert_eq!(decoded.len().checked_mul(2), Some(value.len()));
        }
        let _ = decode_lower_hex(value, 64, EmptyHex::Refuse);
        let _ = posix_relative_path(value);
    }
});
