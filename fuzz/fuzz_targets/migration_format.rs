#![no_main]

//! This target owns canonical store-migration record parser fuzzing.

use keep::{
    AdmittedStoreFormatMarker, AdmittedStoreMigrationIntent, AdmittedStoreMigrationReceipt,
};
use libfuzzer_sys::fuzz_target;

const MARKER_BYTES: usize = 96;
const INTENT_BYTES: usize = 256;

fuzz_target!(|bytes: &[u8]| {
    let Some((&selector, input)) = bytes.split_first() else {
        return;
    };
    match selector {
        0 => marker(input),
        1 => intent(input),
        _ => receipt(input),
    }
});

fn marker(input: &[u8]) {
    if let Ok(marker) = AdmittedStoreFormatMarker::decode(input) {
        assert_eq!(marker.encoded(), input);
    }
}

fn intent(input: &[u8]) {
    if let Ok(intent) = AdmittedStoreMigrationIntent::decode(input) {
        assert_eq!(intent.encoded(), input);
    }
}

fn receipt(input: &[u8]) {
    let Some((marker_bytes, remainder)) = input.split_at_checked(MARKER_BYTES) else {
        return;
    };
    let Some((intent_bytes, receipt_bytes)) = remainder.split_at_checked(INTENT_BYTES) else {
        return;
    };
    let (Ok(marker), Ok(intent)) = (
        AdmittedStoreFormatMarker::decode(marker_bytes),
        AdmittedStoreMigrationIntent::decode(intent_bytes),
    ) else {
        return;
    };
    if let Ok(receipt) = AdmittedStoreMigrationReceipt::decode(receipt_bytes, &intent, &marker) {
        assert_eq!(receipt.encoded(), receipt_bytes);
    }
}
