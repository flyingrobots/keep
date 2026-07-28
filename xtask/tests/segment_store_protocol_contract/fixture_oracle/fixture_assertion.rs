// Golden-file and manifest assertions over independently constructed bytes.

use std::fmt::Write as _;

fn encode_hex(bytes: &[u8]) -> Result<String, &'static str> {
    let capacity = bytes
        .len()
        .checked_mul(2)
        .ok_or("hex output length overflow")?;
    let mut encoded = String::with_capacity(capacity);
    for byte in bytes {
        write!(&mut encoded, "{byte:02x}").map_err(|_| "hex formatting failed")?;
    }
    Ok(encoded)
}

#[test]
fn golden_artifacts_match_the_independent_fixture_oracle() -> Result<(), String> {
    let expected = expected_artifacts().map_err(String::from)?;
    let fixtures = [
        EMPTY_SEGMENT,
        ONE_ZERO_SEGMENT,
        ONE_ZERO_CATALOG,
        ONE_ZERO_HEAD,
        ONE_ZERO_CATALOG_GENERATION_TWO,
        ONE_ZERO_HEAD_GENERATION_TWO,
        ONE_ZERO_BUNDLE_SEGMENT,
        ONE_ZERO_BUNDLE_CATALOG,
        ONE_ZERO_BUNDLE_HEAD,
    ];
    assert_eq!(expected.len(), fixtures.len());
    assert!(
        expected
            .iter()
            .any(|artifact| artifact.kind == "segment" && artifact.record_count == "2"),
        "golden corpus must cover chunk and layout records in one segment"
    );
    assert!(
        expected
            .iter()
            .any(|artifact| artifact.kind == "catalog" && artifact.entry_count == "2"),
        "golden corpus must cover cross-kind catalog ordering"
    );

    let mut manifest = String::from(
        "keep.segment-store.artifacts/v1\n\
         case\tkind\tbyte_length\trecord_count\tentry_count\tgeneration\t\
         bound_digest_hex\tfinal_checksum_hex\tfixture\n",
    );
    for (artifact, fixture) in expected.iter().zip(fixtures) {
        let encoded = encode_hex(&artifact.bytes).map_err(String::from)?;
        assert_eq!(
            fixture,
            format!("{encoded}\n"),
            "golden fixture drifted: {}",
            artifact.fixture
        );
        let bound_digest = encode_hex(&artifact.bound_digest).map_err(String::from)?;
        let final_checksum = encode_hex(&artifact.final_checksum).map_err(String::from)?;
        writeln!(
            manifest,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            artifact.case_name,
            artifact.kind,
            artifact.bytes.len(),
            artifact.record_count,
            artifact.entry_count,
            artifact.generation,
            bound_digest,
            final_checksum,
            artifact.fixture
        )
        .map_err(|_| "artifact manifest formatting failed")?;
    }
    assert_eq!(ARTIFACTS, manifest);
    Ok(())
}

#[test]
fn golden_chain_freezes_generation_two_and_its_predecessor() -> Result<(), String> {
    let expected = expected_artifacts().map_err(String::from)?;
    let generation_one = expected
        .iter()
        .find(|artifact| artifact.case_name == "one-zero-catalog")
        .ok_or("missing generation-one catalog")?;
    let generation_two = expected
        .iter()
        .find(|artifact| artifact.case_name == "one-zero-catalog-generation-two")
        .ok_or("missing generation-two catalog")?;
    let generation_two_head = expected
        .iter()
        .find(|artifact| artifact.case_name == "one-zero-head-generation-two")
        .ok_or("missing generation-two head")?;

    assert_eq!(
        generation_two.bytes.get(24..32),
        Some(2u64.to_be_bytes().as_slice())
    );
    assert_eq!(
        generation_two.bytes.get(32..64),
        Some(generation_one.bound_digest.as_slice())
    );
    assert_eq!(
        generation_two_head.bytes.get(24..32),
        Some(2u64.to_be_bytes().as_slice())
    );
    assert_eq!(generation_two_head.bound_digest, generation_two.bound_digest);
    Ok(())
}

#[test]
fn fixture_transport_refuses_uppercase_hexadecimal() {
    assert_eq!(
        decode_hex("AA\n"),
        Err("hex fixture contains a nonhexadecimal byte")
    );
}

#[test]
fn publication_head_uses_the_constructed_catalog_length() -> Result<(), &'static str> {
    let segment = build_one_zero_segment()?;
    let record_checksum = segment
        .record_checksum
        .ok_or("one-zero segment must have a record checksum")?;
    let catalog = build_catalog(segment.digest, record_checksum)?;
    let head = build_head_for_catalog(&catalog)?;
    let encoded_length = head
        .0
        .get(32..40)
        .ok_or("publication head lacks its catalog length")?;
    let expected_length = u64::try_from(catalog.bytes.len())
        .map_err(|_| "constructed catalog length overflow")?
        .to_be_bytes();
    assert_eq!(encoded_length, expected_length);
    Ok(())
}
