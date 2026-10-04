//! Field-by-field corruption matrix for GC retirement intents.
//!
//! Selected malformed fields refuse at their named boundary (KEEP-GC-001); opaque coordinates have separate admission laws. Sealing reconstructs integrity fields not targeted by each case.

use std::io;

use keep::{
    AdmittedGcRetirementIntent, CatalogGenerationError, GcGenerationError,
    GcRetirementIntentDecodeError as Refusal, GcRetirementIntentError, LivenessGenerationError,
    RetentionProfileAdmissionError,
};

use super::{
    CANDIDATE_SET_DIGEST_OFFSET, CANDIDATE_WIDTH, CHECKSUM_OFFSET, HEADER_LENGTH,
    INTENT_DIGEST_OFFSET, fixture_bytes,
};
use crate::support::{counted_domain_hash, domain_hash, flip, patch, read_u32};

const CANDIDATE_COUNT_OFFSET: usize = 44;
const TRAILER_LENGTH: usize = 64;
const MAXIMUM_CANDIDATE_COUNT: u32 = 65_536;

#[derive(Clone, Copy)]
enum Seal {
    Nothing,
    Checksum,
    Digests,
    Everything,
}

struct Mutation {
    field: &'static str,
    seal: Seal,
    mutate: fn(&mut Vec<u8>) -> io::Result<()>,
    refuses: fn(&Refusal, &[u8], &[u8]) -> bool,
}

const MATRIX: &[Mutation] = &[
    Mutation {
        field: "magic",
        seal: Seal::Everything,
        mutate: |bytes| flip(bytes, 15),
        refuses: |error, _, mutated| matches!(error, Refusal::InvalidMagic { observed } if Some(observed.as_slice()) == mutated.get(..16)),
    },
    Mutation {
        field: "version",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, 16, &3_u16.to_be_bytes()),
        refuses: |error, _, _| {
            matches!(
                error,
                Refusal::UnsupportedVersion {
                    expected: 2,
                    observed: 3
                }
            )
        },
    },
    Mutation {
        field: "header length",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, 18, &319_u16.to_be_bytes()),
        refuses: |error, _, _| {
            matches!(
                error,
                Refusal::InvalidHeaderLength {
                    expected: 320,
                    observed: 319
                }
            )
        },
    },
    Mutation {
        field: "flags",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, 20, &1_u32.to_be_bytes()),
        refuses: |error, _, _| matches!(error, Refusal::UnsupportedFlags { observed: 1 }),
    },
    Mutation {
        field: "total record length",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, 24, &455_u64.to_be_bytes()),
        refuses: |error, _, _| {
            matches!(
                error,
                Refusal::DeclaredLengthMismatch {
                    expected: 456,
                    observed: 455
                }
            )
        },
    },
    Mutation {
        field: "GC generation zero",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, 32, &0_u64.to_be_bytes()),
        refuses: |error, _, _| {
            matches!(
                error,
                Refusal::Generation {
                    source: GcGenerationError::Zero
                }
            )
        },
    },
    Mutation {
        field: "candidate width",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, 40, &71_u16.to_be_bytes()),
        refuses: |error, _, _| {
            matches!(
                error,
                Refusal::InvalidCandidateWidth {
                    expected: 72,
                    observed: 71
                }
            )
        },
    },
    Mutation {
        field: "reserved candidate bytes",
        seal: Seal::Everything,
        mutate: |bytes| flip(bytes, 42),
        refuses: |error, _, _| matches!(error, Refusal::NonZeroReserved { field: "candidate" }),
    },
    Mutation {
        field: "candidate count participates in the declared length",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, CANDIDATE_COUNT_OFFSET, &2_u32.to_be_bytes()),
        refuses: |error, _, _| {
            matches!(
                error,
                Refusal::DeclaredLengthMismatch {
                    expected: 528,
                    observed: 456
                }
            )
        },
    },
    Mutation {
        field: "liveness generation zero",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, 48, &0_u64.to_be_bytes()),
        refuses: |error, _, _| {
            matches!(
                error,
                Refusal::LivenessGeneration {
                    source: LivenessGenerationError::Zero
                }
            )
        },
    },
    Mutation {
        field: "catalog generation zero",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, 88, &0_u64.to_be_bytes()),
        refuses: |error, _, _| {
            matches!(
                error,
                Refusal::CatalogGeneration {
                    source: CatalogGenerationError::Zero
                }
            )
        },
    },
    Mutation {
        field: "profile identity",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, 128, &2_u32.to_be_bytes()),
        refuses: |error, _, _| {
            matches!(
                error,
                Refusal::Profile {
                    source: RetentionProfileAdmissionError::UnsupportedCoordinate {
                        expected_identity: 1,
                        expected_version: 1,
                        observed_identity: 2,
                        observed_version: 1
                    }
                }
            )
        },
    },
    Mutation {
        field: "profile version",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, 132, &2_u32.to_be_bytes()),
        refuses: |error, _, _| {
            matches!(
                error,
                Refusal::Profile {
                    source: RetentionProfileAdmissionError::UnsupportedCoordinate {
                        expected_identity: 1,
                        expected_version: 1,
                        observed_identity: 1,
                        observed_version: 2
                    }
                }
            )
        },
    },
    Mutation {
        field: "profile-definition digest",
        seal: Seal::Everything,
        mutate: |bytes| flip(bytes, 136),
        refuses: |error, original, mutated| matches!(error, Refusal::Profile { source: RetentionProfileAdmissionError::DefinitionDigestMismatch { expected, observed } } if Some(expected.as_slice()) == original.get(136..168) && Some(observed.as_slice()) == mutated.get(136..168)),
    },
    Mutation {
        field: "candidate-set digest",
        seal: Seal::Digests,
        mutate: |bytes| flip(bytes, CANDIDATE_SET_DIGEST_OFFSET),
        refuses: |error, _, _| matches!(error, Refusal::CandidateSetDigestMismatch { .. }),
    },
    Mutation {
        field: "intent digest",
        seal: Seal::Checksum,
        mutate: |bytes| flip(bytes, INTENT_DIGEST_OFFSET),
        refuses: |error, _, _| matches!(error, Refusal::IntentDigestMismatch { .. }),
    },
    Mutation {
        field: "checksum",
        seal: Seal::Nothing,
        mutate: |bytes| flip(bytes, CHECKSUM_OFFSET),
        refuses: |error, _, _| matches!(error, Refusal::ChecksumMismatch { .. }),
    },
];

#[test]
fn malformed_intent_fields_report_the_named_refusal() -> Result<(), Box<dyn std::error::Error>> {
    for mutation in MATRIX {
        let mut bytes = fixture_bytes()?;
        (mutation.mutate)(&mut bytes)?;
        seal(&mut bytes, mutation.seal)?;
        let Err(error) = AdmittedGcRetirementIntent::decode(&bytes) else {
            return Err(format!("mutated {} was admitted", mutation.field).into());
        };
        assert!(
            (mutation.refuses)(&error, &fixture_bytes()?, &bytes),
            "{} refused with {error:?}",
            mutation.field
        );
    }
    Ok(())
}

#[test]
fn intent_framing_refuses_truncation_and_trailing_data() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fixture_bytes()?;
    let mut truncated = bytes.clone();
    assert!(truncated.pop().is_some());
    assert!(matches!(
        AdmittedGcRetirementIntent::decode(&truncated),
        Err(Refusal::Truncated {
            expected: 456,
            observed: 455,
        })
    ));
    let mut trailing = bytes;
    trailing.push(0);
    assert!(matches!(
        AdmittedGcRetirementIntent::decode(&trailing),
        Err(Refusal::TrailingData {
            expected: 456,
            observed: 457,
        })
    ));
    Ok(())
}

#[test]
fn candidate_order_and_count_bounds_refuse_after_complete_integrity()
-> Result<(), Box<dyn std::error::Error>> {
    let candidate = fixture_candidate()?;
    let empty = reframe(&[], 0)?;
    assert!(matches!(
        AdmittedGcRetirementIntent::decode(&empty),
        Err(Refusal::Semantic {
            source: GcRetirementIntentError::NoCandidates,
        })
    ));

    let mut repeated = candidate.clone();
    repeated.extend_from_slice(&candidate);
    let repeated = reframe(&repeated, 2)?;
    assert!(matches!(
        AdmittedGcRetirementIntent::decode(&repeated),
        Err(Refusal::Semantic {
            source: GcRetirementIntentError::DuplicateCandidate { index: 1 },
        })
    ));

    let mut smaller = candidate.clone();
    patch(&mut smaller, 0, &[0x00])?;
    let mut descending = candidate.clone();
    descending.extend_from_slice(&smaller);
    let descending = reframe(&descending, 2)?;
    assert!(matches!(
        AdmittedGcRetirementIntent::decode(&descending),
        Err(Refusal::Semantic {
            source: GcRetirementIntentError::NonCanonicalCandidateOrder { index: 1 },
        })
    ));
    let mut ascending = smaller;
    ascending.extend_from_slice(&candidate);
    let ascending = reframe(&ascending, 2)?;
    let two = AdmittedGcRetirementIntent::decode(&ascending)?;
    assert_eq!(two.intent().candidate_count(), 2);

    let count = MAXIMUM_CANDIDATE_COUNT
        .checked_add(1)
        .ok_or_else(|| io::Error::other("candidate ceiling overflows"))?;
    let body_length = usize::try_from(count)?
        .checked_mul(CANDIDATE_WIDTH)
        .ok_or_else(|| io::Error::other("candidate body overflows"))?;
    let oversized = reframe(&vec![0_u8; body_length], count)?;
    assert!(matches!(
        AdmittedGcRetirementIntent::decode(&oversized),
        Err(Refusal::CandidateCountExceeded {
            maximum: MAXIMUM_CANDIDATE_COUNT,
            observed,
        }) if observed == count
    ));
    Ok(())
}

fn fixture_candidate() -> io::Result<Vec<u8>> {
    let bytes = fixture_bytes()?;
    bytes
        .get(HEADER_LENGTH..INTENT_DIGEST_OFFSET)
        .map(<[u8]>::to_vec)
        .ok_or_else(|| io::Error::other("frozen intent lacks its candidate body"))
}

/// Builds an intent from the frozen header with a replaced candidate body.
fn reframe(candidates: &[u8], candidate_count: u32) -> io::Result<Vec<u8>> {
    let fixture = fixture_bytes()?;
    let mut bytes = fixture
        .get(..HEADER_LENGTH)
        .map(<[u8]>::to_vec)
        .ok_or_else(|| io::Error::other("frozen intent lacks its header"))?;
    bytes.extend_from_slice(candidates);
    bytes.extend_from_slice(&[0_u8; TRAILER_LENGTH]);
    let total_length = u64::try_from(bytes.len())
        .map_err(|_| io::Error::other("record exceeds the u64 length field"))?;
    patch(&mut bytes, 24, &total_length.to_be_bytes())?;
    patch(
        &mut bytes,
        CANDIDATE_COUNT_OFFSET,
        &candidate_count.to_be_bytes(),
    )?;
    seal(&mut bytes, Seal::Everything)?;
    Ok(bytes)
}

fn seal(bytes: &mut [u8], seal: Seal) -> io::Result<()> {
    let checksum_offset = bytes
        .len()
        .checked_sub(32)
        .ok_or_else(|| io::Error::other("GC intent lacks a checksum"))?;
    let digest_offset = checksum_offset
        .checked_sub(32)
        .ok_or_else(|| io::Error::other("GC intent lacks a digest"))?;
    if matches!(seal, Seal::Everything) {
        let count = read_u32(bytes, CANDIDATE_COUNT_OFFSET)?;
        let body = bytes
            .get(HEADER_LENGTH..digest_offset)
            .ok_or_else(|| io::Error::other("GC intent lacks its candidate body"))?;
        let digest = counted_domain_hash(b"keep.gc-candidate-set/v2\0", count, body);
        patch(bytes, CANDIDATE_SET_DIGEST_OFFSET, &digest)?;
    }
    if matches!(seal, Seal::Everything | Seal::Digests) {
        let preimage = bytes
            .get(..digest_offset)
            .ok_or_else(|| io::Error::other("GC intent lacks its digest preimage"))?;
        let digest = domain_hash(b"keep.gc-retirement-intent/v2\0", preimage);
        patch(bytes, digest_offset, &digest)?;
    }
    if !matches!(seal, Seal::Nothing) {
        let preimage = bytes
            .get(..checksum_offset)
            .ok_or_else(|| io::Error::other("GC intent lacks its checksum preimage"))?;
        let checksum = domain_hash(b"keep.gc-retirement-intent-checksum/v2\0", preimage);
        patch(bytes, checksum_offset, &checksum)?;
    }
    Ok(())
}
