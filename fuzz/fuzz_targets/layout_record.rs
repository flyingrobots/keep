#![no_main]

//! This target owns bounded flat-layout decoding and canonicality fuzzing.

use keep::{AdmittedLayout, LayoutDecodePolicy, LayoutEntryLimit};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    let policy = LayoutDecodePolicy::new(LayoutEntryLimit::MAXIMUM);
    if let Ok(layout) = AdmittedLayout::decode_record(bytes, policy) {
        let encoded = layout.encode_record();
        assert!(
            encoded.is_ok(),
            "an admitted layout failed canonical encoding: {:?}",
            encoded.as_ref().err()
        );
        let Ok(canonical) = encoded else {
            std::process::abort();
        };
        assert_eq!(canonical.bytes(), bytes);
        let expected_policy = policy.with_expected_id(canonical.id());
        assert!(AdmittedLayout::decode_record(bytes, expected_policy).is_ok());
    }
});
