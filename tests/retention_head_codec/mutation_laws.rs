//! Field-by-field corruption matrix for the version-2 retention head.
//!
//! Selected malformed head fields report their named refusal (KEEP-RETENTION-003). Generation and manifest-length cases assert complete nested diagnostics; the opaque manifest digest has a separate admission law.

use std::io;

use keep::{
    ChecksummedRetentionHead, LivenessGenerationError, RetentionHeadDecodeError as Refusal,
    RetentionHeadError, RetentionManifestLengthError,
};

use super::{CHECKSUM_OFFSET, ONE_ROOT_HEAD, fixture_bytes};
use crate::support::{domain_hash, flip, patch};

enum Seal {
    Checksum,
    Nothing,
}

struct Mutation {
    field: &'static str,
    seal: Seal,
    mutate: fn(&mut Vec<u8>) -> io::Result<()>,
    refuses: fn(&Refusal) -> bool,
}

const MATRIX: &[Mutation] = &[
    Mutation {
        field: "magic",
        seal: Seal::Checksum,
        mutate: |bytes| flip(bytes, 15),
        refuses: |error| matches!(error, Refusal::InvalidMagic { .. }),
    },
    Mutation {
        field: "version",
        seal: Seal::Checksum,
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
        seal: Seal::Checksum,
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
        seal: Seal::Checksum,
        mutate: |bytes| patch(bytes, 20, &1_u32.to_be_bytes()),
        refuses: |error| matches!(error, Refusal::UnsupportedFlags { observed: 1 }),
    },
    Mutation {
        field: "liveness generation zero",
        seal: Seal::Checksum,
        mutate: |bytes| patch(bytes, 24, &0_u64.to_be_bytes()),
        refuses: |error| {
            matches!(
                error,
                Refusal::LivenessGeneration {
                    source: LivenessGenerationError::Zero
                }
            )
        },
    },
    Mutation {
        field: "liveness generation two without predecessor",
        seal: Seal::Checksum,
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
        seal: Seal::Checksum,
        mutate: |bytes| patch(bytes, 32, &223_u64.to_be_bytes()),
        refuses: |error| {
            matches!(
                error,
                Refusal::ManifestLength {
                    source: RetentionManifestLengthError::OutOfBounds {
                        minimum: 224,
                        maximum: 295_136,
                        observed: 223
                    }
                }
            )
        },
    },
    Mutation {
        field: "manifest length not congruent",
        seal: Seal::Checksum,
        mutate: |bytes| patch(bytes, 32, &225_u64.to_be_bytes()),
        refuses: |error| {
            matches!(
                error,
                Refusal::ManifestLength {
                    source: RetentionManifestLengthError::NotCongruent { observed: 225 }
                }
            )
        },
    },
    Mutation {
        field: "predecessor digest at generation one",
        seal: Seal::Checksum,
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
        seal: Seal::Checksum,
        mutate: |bytes| flip(bytes, 111),
        refuses: |error| matches!(error, Refusal::NonZeroReserved { .. }),
    },
    Mutation {
        field: "checksum",
        seal: Seal::Nothing,
        mutate: |bytes| flip(bytes, 143),
        refuses: |error| matches!(error, Refusal::ChecksumMismatch { .. }),
    },
];

#[test]
fn malformed_head_fields_report_the_named_refusal() -> Result<(), Box<dyn std::error::Error>> {
    for mutation in MATRIX {
        let mut bytes = fixture_bytes(ONE_ROOT_HEAD)?;
        (mutation.mutate)(&mut bytes)?;
        match mutation.seal {
            Seal::Checksum => reseal(&mut bytes)?,
            Seal::Nothing => {}
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
