//! This module owns nonempty strict migration interruption-prefix laws.

use std::error::Error;

use keep::{
    AdmittedStoreFormatMarker, AdmittedStoreMigrationIntent, AdmittedStoreMigrationReceipt,
};
use xtask::protocol_admission::{EmptyHex, decode_lower_hex};

use super::{INTENT_INTERRUPTION, MARKER_INTERRUPTION, RECEIPT_INTERRUPTION};

const INTENT: &str = include_str!("../../../../conformance/segment-store/v2/migration-intent.hex");
const MARKER: &str = include_str!("../../../../conformance/segment-store/v2/format-marker.hex");
const RECEIPT: &str =
    include_str!("../../../../conformance/segment-store/v2/migration-receipt.hex");

#[test]
fn stage_interruptions_are_nonempty_strict_prefixes_of_admitted_records()
-> Result<(), Box<dyn Error>> {
    let intent_bytes = decode(INTENT)?;
    let marker_bytes = decode(MARKER)?;
    let receipt_bytes = decode(RECEIPT)?;
    let intent = AdmittedStoreMigrationIntent::decode(&intent_bytes)?;
    let marker = AdmittedStoreFormatMarker::decode(&marker_bytes)?;
    let receipt = AdmittedStoreMigrationReceipt::decode(&receipt_bytes, &intent, &marker)?;
    for (bytes, prefix) in [
        (intent.encoded(), INTENT_INTERRUPTION),
        (marker.encoded(), MARKER_INTERRUPTION),
        (receipt.encoded(), RECEIPT_INTERRUPTION),
    ] {
        assert!(prefix > 0, "an interruption must leave a nonempty prefix");
        assert!(
            prefix < bytes.len(),
            "an interruption must not complete the record"
        );
    }
    Ok(())
}

fn decode(text: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let hex = text
        .strip_suffix('\n')
        .ok_or("fixture has no final newline")?;
    Ok(decode_lower_hex(hex, 256, EmptyHex::Refuse)?)
}
