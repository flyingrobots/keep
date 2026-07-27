#![no_main]

//! This target owns bounded Golden File Worldline production-parser fuzzing.

use libfuzzer_sys::fuzz_target;
use xtask::admit_golden_protocol;

fuzz_target!(|bytes: &[u8]| {
    if let Some((selector, input)) = bytes.split_first() {
        let _ = admit_golden_protocol(*selector, input);
    }
});
