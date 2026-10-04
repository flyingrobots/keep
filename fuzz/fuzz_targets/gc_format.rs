#![no_main]

//! This target owns canonical GC retirement record parser fuzzing.

use keep::{
    AdmittedGcRetirementIntent, AdmittedGcRetirementReceipt, AdmittedRecoveryDispositionReceipt,
    CanonicalGcRetirementIntent, CanonicalGcRetirementReceipt, CanonicalRecoveryDispositionReceipt,
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
        let canonical = CanonicalRecoveryDispositionReceipt::from_receipt(receipt.receipt());
        assert_eq!(
            (canonical.encoded(), canonical.receipt()),
            (input, receipt.receipt()),
            "admitted disposition must preserve its canonical bytes and semantics"
        );
    }
}

fn intent(input: &[u8]) {
    if let Ok(intent) = AdmittedGcRetirementIntent::decode(input) {
        let _ = canonical_intent(&intent);
    }
}

// Oracle: the canonical format relation, using the encoder rather than the
// decoder's retained input. Frozen independent vectors complement this relation.
fn canonical_intent(
    intent: &AdmittedGcRetirementIntent<'_>,
) -> Option<CanonicalGcRetirementIntent> {
    let canonical = CanonicalGcRetirementIntent::from_intent(intent.intent());
    assert_eq!(
        canonical.as_ref().ok().map(|value| (
            value.encoded(),
            value.intent(),
            value.digest(),
            value.candidate_set_digest(),
        )),
        Some((
            intent.encoded(),
            intent.intent(),
            intent.digest(),
            intent.candidate_set_digest(),
        )),
        "admitted intent must preserve its canonical bytes, semantics and digests: {canonical:?}"
    );
    canonical.ok()
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
        let Some(canonical_intent) = canonical_intent(&intent) else {
            return;
        };
        let canonical = CanonicalGcRetirementReceipt::from_intent(
            &canonical_intent,
            receipt.receipt().pool_state_digest(),
        );
        assert_eq!(
            (canonical.encoded(), canonical.receipt()),
            (receipt_bytes, receipt.receipt()),
            "admitted retirement receipt must preserve its canonical bytes and semantics"
        );
    }
}
