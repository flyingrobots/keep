//! Deterministic mutation application and outcome assertions.

use keep::{BlobId, BlobIdBinaryParseError};

use super::harness_failure::HarnessFailure;
use super::scenario_corpus::MutationCase;

type TestResult = Result<(), HarnessFailure>;

pub(super) fn apply_mutation(target: &mut Vec<u8>, mutation: &MutationCase) -> TestResult {
    match mutation.operation {
        "xor-byte" => {
            let mask = mutation
                .value
                .first()
                .copied()
                .ok_or_else(|| HarnessFailure::corpus("xor mutation has no mask"))?;
            let byte = target
                .get_mut(mutation.offset)
                .ok_or_else(|| HarnessFailure::corpus("xor mutation offset escaped"))?;
            *byte ^= mask;
        }
        "truncate" => {
            if mutation.offset >= target.len() {
                return Err(HarnessFailure::corpus("truncate mutation offset escaped"));
            }
            target.truncate(mutation.offset);
        }
        "append" => {
            if mutation.offset != target.len() {
                return Err(HarnessFailure::corpus("append mutation offset moved"));
            }
            target.extend_from_slice(&mutation.value);
        }
        "set-u16-be" | "set-u8" => {
            let end = mutation
                .offset
                .checked_add(mutation.value.len())
                .ok_or_else(|| HarnessFailure::corpus("set mutation length overflow"))?;
            let destination = target
                .get_mut(mutation.offset..end)
                .ok_or_else(|| HarnessFailure::corpus("set mutation range escaped"))?;
            destination.copy_from_slice(&mutation.value);
        }
        _ => return Err(HarnessFailure::corpus("unknown mutation operation")),
    }
    Ok(())
}

#[test]
fn truncate_mutation_must_remove_at_least_one_byte() {
    for offset in [3, 4] {
        let mutation = MutationCase {
            target_kind: "content",
            target_case: "synthetic",
            operation: "truncate",
            offset,
            value: Vec::new(),
            expected_outcome: "keep.content.mismatch",
        };
        let mut target = vec![1, 2, 3];
        assert!(matches!(
            apply_mutation(&mut target, &mutation),
            Err(HarnessFailure::Corpus {
                fact: "truncate mutation offset escaped"
            })
        ));
        assert_eq!(target, [1, 2, 3]);
    }
}

pub(super) fn assert_binary_mutation(
    mutation: &MutationCase,
    original: BlobId,
    encoded: &[u8],
) -> TestResult {
    match mutation.expected_outcome {
        "keep.identity.invalid_magic" => assert!(matches!(
            BlobId::parse_binary(encoded),
            Err(BlobIdBinaryParseError::InvalidMagic { .. })
        )),
        "keep.identity.unsupported_version" => assert_eq!(
            BlobId::parse_binary(encoded),
            Err(BlobIdBinaryParseError::UnsupportedVersion {
                expected: 1,
                observed: 2,
            })
        ),
        "keep.identity.unsupported_algorithm" => assert_eq!(
            BlobId::parse_binary(encoded),
            Err(BlobIdBinaryParseError::UnsupportedAlgorithm {
                expected: 1,
                observed: 2,
            })
        ),
        "keep.identity.truncated" => assert_eq!(
            BlobId::parse_binary(encoded),
            Err(BlobIdBinaryParseError::Truncated {
                expected: BlobId::BINARY_LENGTH,
                observed: encoded.len(),
            })
        ),
        "keep.identity.trailing_data" => assert_eq!(
            BlobId::parse_binary(encoded),
            Err(BlobIdBinaryParseError::TrailingData {
                expected: BlobId::BINARY_LENGTH,
                observed: encoded.len(),
            })
        ),
        "keep.identity.different_supported_identity" => {
            assert_ne!(BlobId::parse_binary(encoded)?, original);
        }
        _ => return Err(HarnessFailure::corpus("unknown binary mutation outcome")),
    }
    Ok(())
}
