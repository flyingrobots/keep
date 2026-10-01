#![no_main]

//! This target owns canonical GC retirement record parser fuzzing.

use keep::{
    AdmittedGcRetirementIntent, AdmittedGcRetirementReceipt, AdmittedRecoveryDispositionReceipt,
};
use libfuzzer_sys::fuzz_target;

// Receipt inputs carry their exact intent dependency first, framed by a
// big-endian `u32` intent length, so mutations exercise cross-record
// binding and not only framing.
fuzz_target!(|bytes: &[u8]| {
    let Some((&selector, input)) = bytes.split_first() else {
        return;
    };
    match selector {
        0 => intent(input),
        1 => receipt(input),
        _ => disposition(input),
    }
});

fn disposition(input: &[u8]) {
    if let Ok(receipt) = AdmittedRecoveryDispositionReceipt::decode(input) {
        assert_eq!(receipt.encoded(), input);
    }
}

fn intent(input: &[u8]) {
    if let Ok(intent) = AdmittedGcRetirementIntent::decode(input) {
        assert_eq!(intent.encoded(), input);
    }
}

fn receipt(input: &[u8]) {
    let Some((length, remainder)) = input.split_at_checked(4) else {
        return;
    };
    let Ok(length) = <[u8; 4]>::try_from(length).map(u32::from_be_bytes) else {
        return;
    };
    let Ok(length) = usize::try_from(length) else {
        return;
    };
    let Some((intent_bytes, receipt_bytes)) = remainder.split_at_checked(length) else {
        return;
    };
    let Ok(intent) = AdmittedGcRetirementIntent::decode(intent_bytes) else {
        return;
    };
    if let Ok(receipt) = AdmittedGcRetirementReceipt::decode(receipt_bytes, &intent) {
        assert_eq!(receipt.encoded(), receipt_bytes);
    }
}
