//! Caller-supplied layouts preserve the same durable admission laws as reference reads.
//!
//! Size: medium. Oracle: source bytes and explicitly different same-length blob identities.
//! Delete when layout ingress is removed or stronger boundary evidence subsumes these laws.

use std::error::Error;

use super::durable_fixture::{build, identify, policy};
use keep::{
    AdmittedLayout, BlobId, ByteLength, ByteOffset, ByteRange, DurableReadError, DurableStore,
    LayoutDecodePolicy, LayoutEntryLimit, ReaderAttemptLimit, ReconstructionError,
    RegisteredStorageProfile,
};

#[test]
fn durable_layout_ingress_routes_return_identical_authenticated_bytes() -> Result<(), Box<dyn Error>>
{
    let bytes = b"one catalogued layout through every ingress";
    let sandbox = build("durable-layout-ingress", &[bytes])?;
    let identified = identify(bytes)?;
    let decode = LayoutDecodePolicy::new(LayoutEntryLimit::MAXIMUM);
    let layout = AdmittedLayout::decode_record(identified.record.bytes(), decode)?;
    let snapshot =
        DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT)?.snapshot()?;
    let mut admitted_output = Vec::new();
    let admitted = snapshot.reconstruct_admitted_layout(&layout, &mut admitted_output)?;
    let mut record_output = Vec::new();
    let record =
        snapshot.reconstruct_record(identified.record.bytes(), decode, &mut record_output)?;
    assert_eq!(admitted_output, bytes);
    assert_eq!(record_output, bytes);
    assert_eq!(admitted, record);
    let requested = ByteRange::new(ByteOffset::new(4), ByteLength::new(10))?;
    let mut admitted_range = Vec::new();
    let first = snapshot.read_admitted_layout_range(&layout, requested, &mut admitted_range)?;
    let mut record_range = Vec::new();
    let second = snapshot.read_record_range(
        identified.record.bytes(),
        decode,
        requested,
        &mut record_range,
    )?;
    assert_eq!(Some(admitted_range.as_slice()), bytes.get(4..14));
    assert_eq!(Some(record_range.as_slice()), bytes.get(4..14));
    assert_eq!(first, second);
    Ok(())
}

#[test]
fn a_wrong_whole_blob_claim_refuses_before_any_durable_output() -> Result<(), Box<dyn Error>> {
    let bytes = b"aaaa";
    let sandbox = build("durable-wrong-blob", &[bytes])?;
    let identified = identify(bytes)?;
    let wrong = BlobId::hash_bytes(b"bbbb")?;
    let layout = AdmittedLayout::from_spans(
        wrong,
        RegisteredStorageProfile::FAST_CDC_64K_V1,
        identified.spans,
        LayoutEntryLimit::MAXIMUM,
    )?;
    let layout_id = layout.encode_record()?.id();
    let snapshot =
        DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT)?.snapshot()?;
    let mut output = vec![0xAB];
    let failure = snapshot
        .reconstruct_admitted_layout(&layout, &mut output)
        .err()
        .ok_or("false blob claim succeeded")?;
    assert!(
        matches!(&failure, DurableReadError::Reconstruction(source)
        if matches!(source.as_ref(), ReconstructionError::BlobIdentityMismatch { layout, expected, observed }
            if *layout == layout_id && *expected == wrong && *observed == identified.target)),
        "exact whole-blob refusal: {failure:?}"
    );
    assert_eq!(output, [0xAB]);
    Ok(())
}

#[test]
fn supplied_range_layouts_cannot_replace_the_catalogued_target_binding()
-> Result<(), Box<dyn Error>> {
    let bytes = b"aaaa";
    let sandbox = build("durable-range-target-binding", &[bytes])?;
    let identified = identify(bytes)?;
    let wrong = BlobId::hash_bytes(b"bbbb")?;
    let layout = AdmittedLayout::from_spans(
        wrong,
        RegisteredStorageProfile::FAST_CDC_64K_V1,
        identified.spans,
        LayoutEntryLimit::MAXIMUM,
    )?;
    let record = layout.encode_record()?;
    let snapshot =
        DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT)?.snapshot()?;
    let requested = ByteRange::new(ByteOffset::new(0), ByteLength::new(1))?;
    let mut output = vec![0xAB];
    let failure = snapshot
        .read_admitted_layout_range(&layout, requested, &mut output)
        .err()
        .ok_or("uncatalogued layout succeeded")?;
    assert!(
        matches!(failure, DurableReadError::LayoutMissing { requested } if requested == record.id())
    );
    let failure = snapshot
        .read_record_range(
            record.bytes(),
            LayoutDecodePolicy::new(LayoutEntryLimit::MAXIMUM),
            requested,
            &mut output,
        )
        .err()
        .ok_or("uncatalogued record succeeded")?;
    assert!(
        matches!(failure, DurableReadError::LayoutMissing { requested } if requested == record.id())
    );
    assert_eq!(output, [0xAB]);
    Ok(())
}

#[test]
fn corrupt_layout_ingress_refuses_exact_checksum_coordinates_before_output()
-> Result<(), Box<dyn Error>> {
    let bytes = b"a canonical layout whose checksum will be damaged";
    let sandbox = build("durable-layout-checksum", &[bytes])?;
    let identified = identify(bytes)?;
    let mut encoded = identified.record.bytes().to_vec();
    let offset = encoded.len().checked_sub(32).ok_or("checksum absent")?;
    let expected: [u8; 32] = encoded.get(offset..).ok_or("checksum absent")?.try_into()?;
    *encoded.last_mut().ok_or("record empty")? ^= 1;
    let observed: [u8; 32] = encoded.get(offset..).ok_or("checksum absent")?.try_into()?;
    let snapshot =
        DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT)?.snapshot()?;
    let decode = LayoutDecodePolicy::new(LayoutEntryLimit::MAXIMUM);
    let mut output = vec![0xAB];
    let failure = snapshot
        .reconstruct_record(&encoded, decode, &mut output)
        .err()
        .ok_or("corrupt record reconstructed")?;
    assert!(
        matches!(failure, DurableReadError::LayoutDecode(keep::LayoutDecodeError::ChecksumMismatch { expected: actual_expected, observed: actual_observed })
        if actual_expected == expected && actual_observed == observed)
    );
    let range = ByteRange::new(ByteOffset::new(0), ByteLength::new(1))?;
    let failure = snapshot
        .read_record_range(&encoded, decode, range, &mut output)
        .err()
        .ok_or("corrupt record range succeeded")?;
    assert!(
        matches!(failure, DurableReadError::LayoutDecode(keep::LayoutDecodeError::ChecksumMismatch { expected: actual_expected, observed: actual_observed })
        if actual_expected == expected && actual_observed == observed)
    );
    assert_eq!(output, [0xAB]);
    Ok(())
}

#[test]
fn a_content_correct_durable_layout_refuses_false_profile_boundaries() -> Result<(), Box<dyn Error>>
{
    let mutation = crate::layout_mutation_support::mutation_cases()?
        .into_iter()
        .find(|candidate| candidate.case() == "profile-boundary-mismatch")
        .ok_or("profile mismatch fixture absent")?;
    let encoded = mutation.mutated_record()?;
    let first = vec![0_u8; 262_143];
    let last = [0_u8; 2];
    let sandbox = build("durable-false-profile", &[&first, &last])?;
    let snapshot =
        DurableStore::open(sandbox.path(), policy()?, ReaderAttemptLimit::DEFAULT)?.snapshot()?;
    let mut output = vec![0xAB];
    let failure = snapshot
        .reconstruct_record(
            &encoded,
            LayoutDecodePolicy::new(LayoutEntryLimit::MAXIMUM),
            &mut output,
        )
        .err()
        .ok_or("false profile boundaries reconstructed")?;
    assert!(
        matches!(&failure, DurableReadError::Reconstruction(error)
        if matches!(error.as_ref(), ReconstructionError::ProfileBoundaryMismatch {
            index: 0, expected: Some(expected), observed: Some(observed), .. }
            if expected.offset().get() == 0 && expected.length().get() == 262_143
                && observed.offset().get() == 0 && observed.length().get() == 262_144)),
        "false profile must preserve exact boundary coordinates: {failure:?}"
    );
    assert_eq!(output, [0xAB]);
    Ok(())
}
