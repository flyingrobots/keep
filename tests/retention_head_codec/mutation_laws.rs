//! Field-by-field corruption matrix for the version-2 retention head.
//!
//! Every field of the fixed 144-byte head has one mutation and one exact
//! first refusal (`KEEP-RETENTION-003`).

use std::io;

use keep::{ChecksummedRetentionHead, RetentionHeadDecodeError as Refusal, RetentionHeadError};

use super::{CHECKSUM_OFFSET, ONE_ROOT_HEAD, fixture_bytes};
use crate::support::{domain_hash, flip, patch};

struct Mutation {
    field: &'static str,
    reseal: bool,
    mutate: fn(&mut Vec<u8>) -> io::Result<()>,
    refuses: fn(&Refusal) -> bool,
}

const MATRIX: &[Mutation] = &[
    Mutation {
        field: "magic",
        reseal: true,
        mutate: |bytes| flip(bytes, 15),
        refuses: |error| matches!(error, Refusal::InvalidMagic { .. }),
    },
    Mutation {
        field: "version",
        reseal: true,
        mutate: |bytes| patch(bytes, 16, &3_u16.to_be_bytes()),
        refuses: |error| {
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
        field: "record length",
        reseal: true,
        mutate: |bytes| patch(bytes, 18, &143_u16.to_be_bytes()),
        refuses: |error| {
            matches!(
                error,
                Refusal::InvalidRecordLength {
                    expected: 144,
                    observed: 143
                }
            )
        },
    },
    Mutation {
        field: "flags",
        reseal: true,
        mutate: |bytes| patch(bytes, 20, &1_u32.to_be_bytes()),
        refuses: |error| matches!(error, Refusal::UnsupportedFlags { observed: 1 }),
    },
    Mutation {
        field: "liveness generation zero",
        reseal: true,
        mutate: |bytes| patch(bytes, 24, &0_u64.to_be_bytes()),
        refuses: |error| matches!(error, Refusal::LivenessGeneration { .. }),
    },
    Mutation {
        field: "liveness generation two without predecessor",
        reseal: true,
        mutate: |bytes| patch(bytes, 24, &2_u64.to_be_bytes()),
        refuses: |error| {
            matches!(
                error,
                Refusal::Semantic {
                    source: RetentionHeadError::MissingPredecessor { .. }
                }
            )
        },
    },
    Mutation {
        field: "manifest length below bound",
        reseal: true,
        mutate: |bytes| patch(bytes, 32, &223_u64.to_be_bytes()),
        refuses: |error| matches!(error, Refusal::ManifestLength { .. }),
    },
    Mutation {
        field: "manifest length not congruent",
        reseal: true,
        mutate: |bytes| patch(bytes, 32, &225_u64.to_be_bytes()),
        refuses: |error| matches!(error, Refusal::ManifestLength { .. }),
    },
    Mutation {
        field: "predecessor digest at generation one",
        reseal: true,
        mutate: |bytes| flip(bytes, 72),
        refuses: |error| {
            matches!(
                error,
                Refusal::Semantic {
                    source: RetentionHeadError::InitialGenerationHasPredecessor { .. },
                }
            )
        },
    },
    Mutation {
        field: "reserved bytes",
        reseal: true,
        mutate: |bytes| flip(bytes, 111),
        refuses: |error| matches!(error, Refusal::NonZeroReserved { .. }),
    },
    Mutation {
        field: "checksum",
        reseal: false,
        mutate: |bytes| flip(bytes, 143),
        refuses: |error| matches!(error, Refusal::ChecksumMismatch { .. }),
    },
];

#[test]
fn every_head_field_has_one_exact_first_refusal() -> Result<(), Box<dyn std::error::Error>> {
    for mutation in MATRIX {
        let mut bytes = fixture_bytes(ONE_ROOT_HEAD)?;
        (mutation.mutate)(&mut bytes)?;
        if mutation.reseal {
            reseal(&mut bytes)?;
        }
        let Err(error) = ChecksummedRetentionHead::decode(&bytes) else {
            return Err(format!("mutated {} was admitted", mutation.field).into());
        };
        assert!(
            (mutation.refuses)(&error),
            "{} refused with {error:?}",
            mutation.field
        );
    }
    Ok(())
}

#[test]
fn manifest_digest_is_carried_not_verified_by_the_head() -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = fixture_bytes(ONE_ROOT_HEAD)?;
    flip(&mut bytes, 40)?;
    reseal(&mut bytes)?;
    let head = ChecksummedRetentionHead::decode(&bytes)?;
    let mut expected = fixture_bytes(ONE_ROOT_HEAD)?;
    flip(&mut expected, 40)?;
    assert_eq!(
        head.head().manifest_digest().as_bytes(),
        expected
            .get(40..72)
            .ok_or_else(|| io::Error::other("frozen retention head lacks a digest"))?
    );
    Ok(())
}

fn reseal(bytes: &mut [u8]) -> io::Result<()> {
    let preimage = bytes
        .get(..CHECKSUM_OFFSET)
        .ok_or_else(|| io::Error::other("retention head lacks its checksum preimage"))?;
    let checksum = domain_hash(b"keep.retention-head-checksum/v2\0", preimage);
    patch(bytes, CHECKSUM_OFFSET, &checksum)
}
