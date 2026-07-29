#![no_main]

//! This target owns bounded duplicate-refusing repository JSON fuzzing.

use libfuzzer_sys::fuzz_target;
use xtask::admit_repository_json;

fuzz_target!(|bytes: &[u8]| {
    let _ = admit_repository_json(bytes);
});
