#![no_main]

use keep::BlobId;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|encoded: &[u8]| {
    if let Ok(identity) = BlobId::parse_binary(encoded) {
        assert_eq!(identity.encode_binary().as_slice(), encoded);
    }
});
