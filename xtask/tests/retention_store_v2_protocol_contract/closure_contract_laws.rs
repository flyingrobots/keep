//! Closure verification and corruption-boundary contract laws.

use super::{FORMAT_ROOT, normalized, read};

#[test]
fn closure_accounting_has_exact_units_and_canonical_evidence()
-> Result<(), Box<dyn std::error::Error>> {
    let closure = format!(
        "{} {}",
        normalized(&read(&format!("{FORMAT_ROOT}/closure.md"))?),
        normalized(&read(&format!("{FORMAT_ROOT}/closure-corruption.md"))?)
    );

    for required in [
        "one pinned, completely verified catalog generation",
        "first scheduled",
        "anchor is not a closure node",
        "unique `SegmentRecordIdentity`",
        "depth `1`",
        "depth `2`",
        "canonical layout payload length",
        "complete segment-record length",
        "checked addition before",
        "repeated logical occurrence",
        "replay the exact registered storage profile",
        "authenticate the complete `BlobId`",
        "keep.retention-closure/v2\\0",
        "96-byte closure-member entries",
        "canonical typed-identity order",
        "Missing members still consume",
        "`CatalogSnapshot`",
        "does not accept untrusted bytes",
        "segment-record admission laws",
        "`segment_format` fuzz target",
    ] {
        assert!(
            closure.contains(required),
            "segment-store v2 closure contract omits `{required}`"
        );
    }
    Ok(())
}
