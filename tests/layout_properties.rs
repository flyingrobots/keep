//! Generated canonicality properties for admitted flat layouts.

use std::error::Error;

use keep::{
    AdmittedLayout, BlobId, ChunkSpan, FastCdc, LayoutDecodePolicy, LayoutEntryLimit,
    RegisteredStorageProfile,
};

const LENGTHS: [usize; 12] = [
    0, 1, 2, 31, 16_383, 16_384, 65_535, 65_536, 65_537, 262_143, 262_144, 262_145,
];

#[test]
fn generated_layouts_have_one_canonical_record_and_identity() -> Result<(), Box<dyn Error>> {
    for (case, length) in LENGTHS.into_iter().enumerate() {
        let source = generated_bytes(length, case)?;
        assert_canonical_round_trip(&source)?;
    }
    for case in 0_usize..24 {
        let length = case
            .checked_mul(12_289)
            .and_then(|value| value.checked_add(997))
            .ok_or("generated layout length overflow")?;
        let source = generated_bytes(length, case)?;
        assert_canonical_round_trip(&source)?;
    }
    Ok(())
}

fn assert_canonical_round_trip(source: &[u8]) -> Result<(), Box<dyn Error>> {
    let target = BlobId::hash_bytes(source)?;
    let layout = AdmittedLayout::from_spans(
        target,
        RegisteredStorageProfile::FAST_CDC_64K_V1,
        detect_spans(source)?,
        LayoutEntryLimit::MAXIMUM,
    )?;
    let canonical = layout.encode_record()?;
    let policy =
        LayoutDecodePolicy::new(LayoutEntryLimit::MAXIMUM).with_expected_id(canonical.id());
    let decoded = AdmittedLayout::decode_record(canonical.bytes(), policy)?;
    let reencoded = decoded.encode_record()?;

    assert_eq!(decoded, layout);
    assert_eq!(reencoded.bytes(), canonical.bytes());
    assert_eq!(reencoded.id(), canonical.id());
    Ok(())
}

fn detect_spans(bytes: &[u8]) -> Result<Vec<ChunkSpan>, keep::ChunkingError> {
    let mut detector = FastCdc::new();
    let mut spans = Vec::new();
    detector.feed(bytes, |span| spans.push(span))?;
    if let Some(span) = detector.finish()? {
        spans.push(span);
    }
    Ok(spans)
}

fn generated_bytes(length: usize, case: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    let case_u64 = u64::try_from(case)?;
    let mut state = 0x9e37_79b9_7f4a_7c15_u64
        .checked_add(case_u64)
        .ok_or("generated seed overflow")?;
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(length)?;
    for _ in 0..length {
        state ^= state.wrapping_shl(13);
        state ^= state.wrapping_shr(7);
        state ^= state.wrapping_shl(17);
        bytes.push(u8::try_from(state & u64::from(u8::MAX))?);
    }
    Ok(bytes)
}
