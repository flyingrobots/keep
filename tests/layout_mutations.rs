//! Field-complete flat-layout corruption and first-failure evidence.

#[path = "layout_mutations/support.rs"]
pub mod layout_mutation_support;
pub mod support;

use std::error::Error;

use keep::{
    AdmittedLayout, BlobIdBinaryParseError, LayoutDecodeError, LayoutDecodePolicy,
    LayoutEntryLimit, LayoutId, LayoutIdMismatch, LayoutValidationError,
};
use layout_mutation_support::{mutation_cases, recompute_record_checksum};
use support::{decode_hex, invalid_corpus};

#[test]
fn every_frozen_mutation_reaches_its_exact_first_failure_phase() -> Result<(), Box<dyn Error>> {
    for mutation in mutation_cases()? {
        let bytes = mutation.mutated_record()?;
        let result = AdmittedLayout::decode_record(
            &bytes,
            LayoutDecodePolicy::new(LayoutEntryLimit::MAXIMUM),
        );
        if mutation.decision_phase() == "verification" {
            let layout = result?;
            assert_eq!(
                layout.encode_record()?.bytes(),
                bytes,
                "{}",
                mutation.case()
            );
            continue;
        }
        let error = require_error(result, "mutation was unexpectedly admitted")?;
        assert_eq!(
            classify(&error),
            Some(mutation.expected_outcome()),
            "{}: {error:?}",
            mutation.case()
        );
    }
    Ok(())
}

#[test]
fn configured_entry_cap_refuses_before_materialization() -> Result<(), Box<dyn Error>> {
    let bytes = fixture("max-plus-one-zeros")?;
    let policy = LayoutDecodePolicy::new(LayoutEntryLimit::new(1)?);
    let error = require_error(
        AdmittedLayout::decode_record(&bytes, policy),
        "two entries were admitted under a one-entry cap",
    )?;

    assert!(matches!(
        error,
        LayoutDecodeError::ConfiguredEntryLimitExceeded {
            maximum: 1,
            observed: 2
        }
    ));
    Ok(())
}

#[test]
fn expected_layout_identity_is_checked_after_structural_admission() -> Result<(), Box<dyn Error>> {
    let empty_id = layout_id("empty")?;
    let one_zero_bytes = fixture("one-zero")?;
    let length_error = require_error(
        AdmittedLayout::decode_record(
            &one_zero_bytes,
            LayoutDecodePolicy::new(LayoutEntryLimit::MAXIMUM).with_expected_id(empty_id),
        ),
        "a different record length matched expected identity",
    )?;
    assert!(matches!(
        length_error,
        LayoutDecodeError::LayoutIdentity {
            source: LayoutIdMismatch::PlanLength { .. }
        }
    ));

    let mut altered_coordinate = layout_id_binary("one-zero")?;
    let last = altered_coordinate
        .last_mut()
        .ok_or_else(|| invalid_corpus("layout identity binary is empty"))?;
    *last ^= 1;
    let altered_id = LayoutId::parse_binary(&altered_coordinate)?;
    let digest_error = require_error(
        AdmittedLayout::decode_record(
            &one_zero_bytes,
            LayoutDecodePolicy::new(LayoutEntryLimit::MAXIMUM).with_expected_id(altered_id),
        ),
        "a different layout digest matched expected identity",
    )?;
    assert!(matches!(
        digest_error,
        LayoutDecodeError::LayoutIdentity {
            source: LayoutIdMismatch::Digest { .. }
        }
    ));
    Ok(())
}

#[test]
fn earlier_layout_laws_precede_zero_lengths_in_later_entry_decoding() -> Result<(), Box<dyn Error>>
{
    let empty = fixture("empty")?;
    let mut cardinality = fixture("one-zero")?;
    overwrite(&mut cardinality, 44, 103, bytes_between(&empty, 44, 103)?)?;
    overwrite(&mut cardinality, 152, 156, &[0_u8; 4])?;
    recompute_record_checksum(&mut cardinality)?;
    let cardinality_error = require_error(
        AdmittedLayout::decode_record(
            &cardinality,
            LayoutDecodePolicy::new(LayoutEntryLimit::MAXIMUM),
        ),
        "empty cardinality with a zero-length entry was admitted",
    )?;
    assert!(matches!(
        cardinality_error,
        LayoutDecodeError::Validation {
            source: LayoutValidationError::EmptyBlobHasEntries { .. }
        }
    ));

    let mut ordering = mutation_cases()?
        .into_iter()
        .find(|mutation| mutation.case() == "entry-order-swap")
        .ok_or_else(|| invalid_corpus("entry-order mutation is missing"))?
        .mutated_record()?;
    overwrite(&mut ordering, 284, 288, &[0_u8; 4])?;
    recompute_record_checksum(&mut ordering)?;
    let ordering_error = require_error(
        AdmittedLayout::decode_record(
            &ordering,
            LayoutDecodePolicy::new(LayoutEntryLimit::MAXIMUM),
        ),
        "an earlier gap was hidden by a later zero-length entry",
    )?;
    assert!(matches!(
        ordering_error,
        LayoutDecodeError::Validation {
            source: LayoutValidationError::Gap { index: 1, .. }
        }
    ));
    Ok(())
}

const fn classify(error: &LayoutDecodeError) -> Option<&'static str> {
    match error {
        LayoutDecodeError::InvalidMagic { .. } => Some("layout.wrong-magic"),
        LayoutDecodeError::UnsupportedFormatVersion { .. } => {
            Some("layout.unsupported-format-version")
        }
        LayoutDecodeError::UnsupportedCodec { .. } => Some("layout.unsupported-codec"),
        LayoutDecodeError::UnknownFlags { .. } => Some("layout.unknown-flags"),
        LayoutDecodeError::WrongHeaderLength { .. } => Some("layout.wrong-header-length"),
        LayoutDecodeError::WrongEntryLength { .. } => Some("layout.wrong-entry-length"),
        LayoutDecodeError::RecordLengthMismatch { .. } => Some("layout.record-length-mismatch"),
        LayoutDecodeError::EntryCountLengthMismatch { .. } => {
            Some("layout.entry-count-length-mismatch")
        }
        LayoutDecodeError::UnsupportedChecksumAlgorithm { .. } => {
            Some("layout.unsupported-checksum-algorithm")
        }
        LayoutDecodeError::UnsupportedChunkHashAlgorithm { .. } => {
            Some("layout.unsupported-chunk-hash-algorithm")
        }
        LayoutDecodeError::UnsupportedChunkIdentityVersion { .. } => {
            Some("layout.unsupported-chunk-identity-version")
        }
        LayoutDecodeError::BlobId { source } => classify_blob_id(source),
        LayoutDecodeError::UnsupportedStorageProfileVersion { .. } => {
            Some("layout.unsupported-storage-profile-version")
        }
        LayoutDecodeError::UnsupportedStorageProfileAlgorithm { .. } => {
            Some("layout.unsupported-storage-profile-algorithm")
        }
        LayoutDecodeError::StorageProfile { .. } => Some("layout.unsupported-storage-profile"),
        LayoutDecodeError::NonzeroReserved { .. } => Some("layout.nonzero-reserved"),
        LayoutDecodeError::ZeroChunkLength { .. } => Some("layout.zero-chunk-length"),
        LayoutDecodeError::ChecksumMismatch { .. } => Some("layout.checksum-mismatch"),
        LayoutDecodeError::TruncatedRecord { .. } => Some("layout.truncated-record"),
        LayoutDecodeError::TrailingData { .. } => Some("layout.trailing-data"),
        LayoutDecodeError::EntryCountLimitExceeded { .. } => {
            Some("layout.entry-count-limit-exceeded")
        }
        LayoutDecodeError::RecordLengthLimitExceeded { .. } => {
            Some("layout.record-length-limit-exceeded")
        }
        LayoutDecodeError::Validation { source } => classify_validation(source),
        _ => None,
    }
}

const fn classify_blob_id(error: &BlobIdBinaryParseError) -> Option<&'static str> {
    match error {
        BlobIdBinaryParseError::InvalidMagic { .. } => Some("layout.blob-id-wrong-magic"),
        BlobIdBinaryParseError::UnsupportedVersion { .. } => {
            Some("layout.blob-id-unsupported-version")
        }
        BlobIdBinaryParseError::UnsupportedAlgorithm { .. } => {
            Some("layout.blob-id-unsupported-algorithm")
        }
        BlobIdBinaryParseError::Truncated { .. } | BlobIdBinaryParseError::TrailingData { .. } => {
            None
        }
    }
}

const fn classify_validation(error: &LayoutValidationError) -> Option<&'static str> {
    match error {
        LayoutValidationError::EmptyBlobHasEntries { .. } => Some("layout.empty-blob-has-entries"),
        LayoutValidationError::NonemptyBlobHasNoEntries => {
            Some("layout.nonempty-blob-has-no-entries")
        }
        LayoutValidationError::FirstOffsetNotZero { .. } => Some("layout.first-offset-not-zero"),
        LayoutValidationError::Gap { .. } => Some("layout.gap"),
        LayoutValidationError::Overlap { .. } => Some("layout.overlap"),
        LayoutValidationError::ProfileLengthOutOfBounds { .. } => {
            Some("layout.profile-length-out-of-bounds")
        }
        LayoutValidationError::AggregateLengthMismatch { .. } => {
            Some("layout.aggregate-length-mismatch")
        }
        _ => None,
    }
}

fn fixture(case: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let encoded = match case {
        "empty" => include_str!("../conformance/layout/v1/empty.layout.hex"),
        "one-zero" => include_str!("../conformance/layout/v1/one-zero.layout.hex"),
        "max-plus-one-zeros" => {
            include_str!("../conformance/layout/v1/max-plus-one-zeros.layout.hex")
        }
        _ => return Err(Box::new(invalid_corpus("unknown layout fixture"))),
    };
    Ok(decode_hex(encoded.trim_end())?)
}

fn layout_id(case: &str) -> Result<LayoutId, Box<dyn Error>> {
    Ok(layout_field(case, 10)?.parse()?)
}

fn layout_id_binary(case: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    Ok(decode_hex(layout_field(case, 11)?)?)
}

fn layout_field(case: &str, index: usize) -> Result<&'static str, Box<dyn Error>> {
    let row = include_str!("../conformance/layout/v1/layouts.tsv")
        .lines()
        .skip(2)
        .find(|row| row.split('\t').next() == Some(case))
        .ok_or_else(|| invalid_corpus("layout case is missing"))?;
    row.split('\t')
        .nth(index)
        .ok_or_else(|| Box::<dyn Error>::from(invalid_corpus("layout field is missing")))
}

fn require_error<T, E>(result: Result<T, E>, message: &'static str) -> Result<E, std::io::Error> {
    match result {
        Ok(_) => Err(invalid_corpus(message)),
        Err(error) => Ok(error),
    }
}

fn bytes_between(bytes: &[u8], start: usize, end: usize) -> Result<&[u8], std::io::Error> {
    bytes
        .get(start..end)
        .ok_or_else(|| invalid_corpus("test fixture span is out of bounds"))
}

fn overwrite(
    bytes: &mut [u8],
    start: usize,
    end: usize,
    replacement: &[u8],
) -> Result<(), std::io::Error> {
    let target = bytes
        .get_mut(start..end)
        .ok_or_else(|| invalid_corpus("test mutation span is out of bounds"))?;
    if target.len() != replacement.len() {
        return Err(invalid_corpus("test mutation replacement width mismatch"));
    }
    target.copy_from_slice(replacement);
    Ok(())
}
