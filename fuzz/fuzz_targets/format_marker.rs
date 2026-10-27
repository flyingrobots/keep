#![no_main]

//! This target owns `FORMAT` marker parser fuzzing.

use keep::{AdmittedStoreFormatMarker, CanonicalStoreFormatMarker};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    if let Ok(marker) = AdmittedStoreFormatMarker::decode(bytes) {
        assert_eq!(marker.encoded(), bytes);
        assert_eq!(
            bytes,
            CanonicalStoreFormatMarker::version_two().encoded(),
            "exactly one marker is registered"
        );
    }
});
