#![no_main]

//! This target owns bounded canonical benchmark-report admission fuzzing.

use libfuzzer_sys::fuzz_target;
use xtask::admit_benchmark_report;

fuzz_target!(|bytes: &[u8]| {
    let _ = admit_benchmark_report(bytes);
});
