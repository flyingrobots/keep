#![no_main]

use keep::BlobId;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    if let Ok(encoded) = std::str::from_utf8(bytes)
        && let Ok(identity) = encoded.parse::<BlobId>()
    {
        assert_eq!(identity.to_string(), encoded);
    }
});
