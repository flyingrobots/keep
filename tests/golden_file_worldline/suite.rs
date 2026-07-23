//! Public-path conformance tests for Keep's first identity milestone.

#[path = "harness_failure.rs"]
mod harness_failure;
#[path = "identity_assertions.rs"]
mod identity_assertions;
#[path = "identity_corpus.rs"]
mod identity_corpus;
#[path = "mutation_assertions.rs"]
mod mutation_assertions;
#[path = "partition_reader.rs"]
mod partition_reader;
#[path = "reference_model.rs"]
mod reference_model;
#[path = "scenario_corpus.rs"]
mod scenario_corpus;

use std::error::Error;
use std::io::ErrorKind;

use harness_failure::HarnessFailure;
use identity_assertions::{
    assert_named_bytes, generated_bytes, hash_partitioned, text_error_class,
};
use identity_corpus::{find_case, identity_cases};
use keep::{BlobId, BlobIdTextParseError, BlobReadError};
use mutation_assertions::{apply_mutation, assert_binary_mutation};
use partition_reader::{FailingReader, LyingReader, PartitionReader};
use reference_model::{MODEL_CAPACITY_BYTES, ReferenceModel, ReferenceModelError};
use scenario_corpus::{invalid_text_cases, mutation_cases, scenario_steps};

type TestResult = Result<(), HarnessFailure>;

#[test]
fn public_blob_id_matches_every_golden_vector() -> TestResult {
    for case in identity_cases()? {
        let bytes = case.bytes()?;
        let expected = case.expected_id()?;
        let observed = BlobId::hash_bytes(&bytes)?;
        assert_eq!(observed, expected, "identity moved for {}", case.name);
        assert_eq!(observed.to_string(), case.expected_text());
        assert_eq!(
            observed.logical_length().get(),
            u64::try_from(bytes.len())
                .map_err(|_source| HarnessFailure::corpus("fixture length does not fit u64"))?
        );
        let binary = case.expected_binary()?;
        assert_eq!(observed.encode_binary().as_slice(), binary.as_slice());
        assert_eq!(BlobId::parse_binary(&binary)?, observed);
    }
    Ok(())
}

#[test]
fn input_partitioning_does_not_move_blob_identity() -> TestResult {
    let irregular = [1, 7, 64, 4_093, 65_536];
    for case in identity_cases()? {
        let bytes = case.bytes()?;
        let expected = case.expected_id()?;
        assert_eq!(hash_partitioned(&bytes, &[4_096])?, expected);
        assert_eq!(hash_partitioned(&bytes, &irregular)?, expected);
        if bytes.len() <= 256 {
            assert_eq!(hash_partitioned(&bytes, &[1])?, expected);
        }
        let mut reader = PartitionReader::new(&bytes, &irregular);
        assert_eq!(BlobId::hash_reader(&mut reader)?, expected);
    }
    Ok(())
}

#[test]
fn generated_bytes_and_partitions_preserve_identity() -> TestResult {
    let lengths = [
        0, 1, 2, 3, 31, 32, 63, 64, 65, 255, 256, 257, 4_095, 4_096, 8_193,
    ];
    let seeds = [1_u64, 0x9e37_79b9_7f4a_7c15, u64::MAX];
    let plans: [&[usize]; 4] = [&[1], &[3, 5, 11], &[64, 4_096], &[7, 257, 8_191]];
    for seed in seeds {
        for length in lengths {
            let bytes = generated_bytes(seed, length);
            let expected = BlobId::hash_bytes(&bytes)?;
            for plan in plans {
                assert_eq!(hash_partitioned(&bytes, plan)?, expected);
                let mut reader = PartitionReader::new(&bytes, plan);
                assert_eq!(BlobId::hash_reader(&mut reader)?, expected);
            }
        }
    }
    Ok(())
}

#[test]
fn noncanonical_text_is_refused_by_exact_error_class() -> TestResult {
    for case in invalid_text_cases()? {
        let input = std::str::from_utf8(&case.input)
            .map_err(|_source| HarnessFailure::corpus("invalid-text fixture is not UTF-8"))?;
        let error = match input.parse::<BlobId>() {
            Ok(_identity) => {
                return Err(HarnessFailure::corpus("invalid identity text was accepted"));
            }
            Err(error) => error,
        };
        assert_eq!(text_error_class(error), case.expected);
    }
    Ok(())
}

#[test]
fn maximum_text_bound_has_exact_acceptance_and_refusal_edges() -> TestResult {
    let maximum = concat!(
        "keep:blob:v1:blake3-256:18446744073709551615:",
        "0000000000000000000000000000000000000000000000000000000000000000"
    );
    let identity: BlobId = maximum.parse()?;
    assert_eq!(identity.logical_length().get(), u64::MAX);
    assert_eq!(identity.to_string(), maximum);
    assert_eq!(BlobId::parse_binary(&identity.encode_binary())?, identity);

    let too_long = "a".repeat(110);
    assert_eq!(
        too_long.parse::<BlobId>(),
        Err(BlobIdTextParseError::InputTooLong {
            maximum: 109,
            observed: 110,
        })
    );
    Ok(())
}

#[test]
fn value_ordering_matches_canonical_binary_ordering() -> TestResult {
    let mut identities = Vec::new();
    for case in identity_cases()? {
        identities.push(case.expected_id()?);
    }
    for left in &identities {
        for right in &identities {
            assert_eq!(
                left.cmp(right),
                left.encode_binary().cmp(&right.encode_binary())
            );
        }
    }
    Ok(())
}

#[test]
fn binary_identity_mutations_never_alias_the_original() -> TestResult {
    for mutation in mutation_cases()?
        .into_iter()
        .filter(|case| case.target_kind == "identity-binary")
    {
        let fixture = find_case(mutation.target_case)?;
        let original = fixture.expected_id()?;
        let mut encoded = fixture.expected_binary()?;
        apply_mutation(&mut encoded, &mutation)?;
        assert_binary_mutation(&mutation, original, &encoded)?;
    }
    Ok(())
}

#[test]
fn every_single_bit_content_change_moves_the_fixture_identity() -> TestResult {
    let case = find_case("state-a")?;
    let original_bytes = case.bytes()?;
    let original_id = case.expected_id()?;
    let masks = [1_u8, 2, 4, 8, 16, 32, 64, 128];
    for offset in 0..original_bytes.len() {
        for mask in masks {
            let mut mutated = original_bytes.clone();
            let byte = mutated
                .get_mut(offset)
                .ok_or_else(|| HarnessFailure::corpus("content mutation offset escaped"))?;
            *byte ^= mask;
            assert_ne!(BlobId::hash_bytes(&mutated)?, original_id);
        }
    }
    Ok(())
}

#[test]
fn reference_model_executes_the_golden_worldline() -> TestResult {
    let mut model = ReferenceModel::new();
    for step in scenario_steps()? {
        let identity = find_case(step.identity_case)?.expected_id()?;
        match step.operation {
            "identify" => {
                assert_eq!(step.expected_outcome, "keep.identity.identified");
                assert_eq!(
                    BlobId::hash_bytes(&find_case(step.input_case)?.bytes()?)?,
                    identity
                );
            }
            "admit-exact" => {
                assert_eq!(step.expected_outcome, "keep.content.admitted");
                assert_eq!(
                    model.admit_exact_materialized(&find_case(step.input_case)?.bytes()?)?,
                    identity
                );
            }
            "read-exact" => {
                assert_eq!(step.expected_outcome, "keep.content.exact");
                let expected = find_case(step.input_case)?.bytes()?;
                let observed = model.read_exact_materialized(identity)?;
                assert_named_bytes(identity, observed)?;
                assert_eq!(observed, expected.as_slice());
            }
            "verify-claimed-content" => {
                assert_eq!(step.expected_outcome, "keep.content.mismatch");
                let result = model
                    .admit_claimed_materialized(identity, &find_case(step.input_case)?.bytes()?);
                assert!(matches!(
                    result,
                    Err(ReferenceModelError::ContentMismatch { .. })
                ));
            }
            "read-absent" => {
                assert_eq!(step.expected_outcome, "keep.content.absent");
                assert_eq!(
                    model.read_exact_materialized(identity),
                    Err(ReferenceModelError::Absent {
                        requested: identity
                    })
                );
            }
            _ => return Err(HarnessFailure::corpus("unknown worldline operation")),
        }
    }
    Ok(())
}

#[test]
fn harness_detects_a_deliberately_substituted_read() -> TestResult {
    let expected = find_case("state-a")?.expected_id()?;
    let substituted = find_case("state-b")?.bytes()?;
    assert!(matches!(
        assert_named_bytes(expected, &substituted),
        Err(HarnessFailure::NamedBytesMismatch {
            expected: observed_expected,
            ..
        }) if observed_expected == expected
    ));
    Ok(())
}

#[test]
fn reference_model_capacity_is_bounded_and_refusal_is_atomic() -> TestResult {
    let mut model = ReferenceModel::new();
    let capacity_filler = vec![0_u8; MODEL_CAPACITY_BYTES];
    let retained = model.admit_exact_materialized(&capacity_filler)?;
    assert_eq!(model.admit_exact_materialized(&capacity_filler)?, retained);

    let refused_bytes = [1_u8];
    let refused = BlobId::hash_bytes(&refused_bytes)?;
    let attempted = MODEL_CAPACITY_BYTES
        .checked_add(refused_bytes.len())
        .ok_or_else(|| HarnessFailure::corpus("model capacity test overflowed"))?;
    assert_eq!(
        model.admit_exact_materialized(&refused_bytes),
        Err(ReferenceModelError::CapacityExceeded {
            capacity: MODEL_CAPACITY_BYTES,
            attempted,
        })
    );
    assert_eq!(
        model.read_exact_materialized(retained)?,
        capacity_filler.as_slice()
    );
    assert_eq!(
        model.read_exact_materialized(refused),
        Err(ReferenceModelError::Absent { requested: refused })
    );
    Ok(())
}

#[test]
fn claimed_identity_refuses_every_content_mutation() -> TestResult {
    let mut model = ReferenceModel::new();
    for mutation in mutation_cases()?
        .into_iter()
        .filter(|case| case.target_kind == "content")
    {
        let fixture = find_case(mutation.target_case)?;
        let identity = fixture.expected_id()?;
        let mut bytes = fixture.bytes()?;
        apply_mutation(&mut bytes, &mutation)?;
        assert_eq!(mutation.expected_outcome, "keep.content.mismatch");
        assert!(matches!(
            model.admit_claimed_materialized(identity, &bytes),
            Err(ReferenceModelError::ContentMismatch { .. })
        ));
    }
    Ok(())
}

#[test]
fn reader_failures_preserve_precise_boundaries_and_sources() -> TestResult {
    let mut failing = FailingReader;
    let error = BlobId::hash_reader(&mut failing).err().ok_or_else(|| {
        HarnessFailure::corpus("failing reader unexpectedly produced an identity")
    })?;
    match &error {
        BlobReadError::Read { source } => assert_eq!(source.kind(), ErrorKind::PermissionDenied),
        _ => {
            return Err(HarnessFailure::corpus(
                "reader failure changed error boundary",
            ));
        }
    }
    assert!(Error::source(&error).is_some());

    let mut lying = LyingReader;
    assert!(matches!(
        BlobId::hash_reader(&mut lying),
        Err(BlobReadError::InvalidReadCount { .. })
    ));
    Ok(())
}
