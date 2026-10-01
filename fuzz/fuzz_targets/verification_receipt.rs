#![no_main]

//! This target owns canonical verification receipt parser fuzzing: an
//! admitted receipt re-encodes to exactly its input.

use keep::CanonicalVerificationReceipt;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    if let Ok(receipt) = CanonicalVerificationReceipt::decode(bytes) {
        assert_eq!(receipt.encoded().as_slice(), bytes);
        let again = CanonicalVerificationReceipt::encode(receipt.receipt());
        assert_eq!(again, receipt);
    }
});
