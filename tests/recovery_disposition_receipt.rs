//! Canonical recovery-disposition receipt laws.

mod support;

use std::io;

use keep::{
    AdmittedRecoveryDispositionReceipt, CanonicalRecoveryDispositionReceipt, GcRetentionState,
    RecoveryArtifactKind, RecoveryClassification, RecoveryDispositionDecision,
    RecoveryDispositionDecodeError as Refusal, RecoveryDispositionField as Field,
    RecoveryDispositionReceipt,
};

use crate::support::{domain_hash, flip, patch};

const DISPOSITION: &str =
    include_str!("../conformance/segment-store/v2/one-orphan-retire-disposition.hex");
const SEGMENT: &str = include_str!("../conformance/segment-store/v1/one-zero-segment.hex");
const DEFINITION: &str = include_str!("../conformance/segment-store/v2/definition.tsv");
const CHECKSUM_DOMAIN: &[u8] = b"keep.recovery-disposition-receipt-checksum/v2\0";
const CHECKSUM_OFFSET: usize = 288;
const CONTENT_DIGEST_OFFSET: usize = 72;
const SEGMENT_DIGEST_OFFSET: usize = 273;

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
        mutate: |bytes| patch(bytes, 18, &319_u16.to_be_bytes()),
        refuses: |error| {
            matches!(
                error,
                Refusal::InvalidRecordLength {
                    expected: 320,
                    observed: 319
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
        field: "artifact kind",
        reseal: true,
        mutate: |bytes| patch(bytes, 24, &6_u16.to_be_bytes()),
        refuses: |error| {
            matches!(
                error,
                Refusal::UnregisteredCode {
                    field: Field::ArtifactKind,
                    observed: 6
                }
            )
        },
    },
    Mutation {
        field: "decision",
        reseal: true,
        mutate: |bytes| patch(bytes, 26, &0_u16.to_be_bytes()),
        refuses: |error| {
            matches!(
                error,
                Refusal::UnregisteredCode {
                    field: Field::Decision,
                    observed: 0
                }
            )
        },
    },
    Mutation {
        field: "classification",
        reseal: true,
        mutate: |bytes| patch(bytes, 28, &4_u16.to_be_bytes()),
        refuses: |error| {
            matches!(
                error,
                Refusal::UnregisteredCode {
                    field: Field::Classification,
                    observed: 4
                }
            )
        },
    },
    Mutation {
        field: "header reserved",
        reseal: true,
        mutate: |bytes| flip(bytes, 31),
        refuses: |error| matches!(error, Refusal::NonZeroReserved),
    },
    Mutation {
        field: "publication-head generation",
        reseal: true,
        mutate: |bytes| patch(bytes, 104, &0_u64.to_be_bytes()),
        refuses: |error| matches!(error, Refusal::ZeroGeneration { offset: 104 }),
    },
    Mutation {
        field: "catalog generation",
        reseal: true,
        mutate: |bytes| patch(bytes, 144, &0_u64.to_be_bytes()),
        refuses: |error| matches!(error, Refusal::ZeroGeneration { offset: 144 }),
    },
    Mutation {
        field: "liveness generation zero beside a published manifest digest",
        reseal: true,
        mutate: |bytes| patch(bytes, 184, &0_u64.to_be_bytes()),
        refuses: |error| matches!(error, Refusal::EmptyRetentionDigestMismatch { .. }),
    },
    Mutation {
        field: "trailer reserved",
        reseal: true,
        mutate: |bytes| flip(bytes, 287),
        refuses: |error| matches!(error, Refusal::NonZeroReserved),
    },
    Mutation {
        field: "checksum",
        reseal: false,
        mutate: |bytes| flip(bytes, 319),
        refuses: |error| matches!(error, Refusal::ChecksumMismatch { .. }),
    },
    Mutation {
        field: "artifact length under a stale checksum",
        reseal: false,
        mutate: |bytes| flip(bytes, 39),
        refuses: |error| matches!(error, Refusal::ChecksumMismatch { .. }),
    },
];

#[test]
fn frozen_disposition_decodes_and_reencodes_canonically() -> Result<(), Box<dyn std::error::Error>>
{
    let bytes = fixture_bytes(DISPOSITION)?;
    let admitted = AdmittedRecoveryDispositionReceipt::decode(&bytes)?;
    assert_eq!(admitted.encoded(), bytes);

    let receipt = *admitted.receipt();
    let artifact = receipt.artifact();
    assert_eq!(artifact.kind, RecoveryArtifactKind::Segment);
    assert_eq!(receipt.decision(), RecoveryDispositionDecision::Retire);
    assert_eq!(
        artifact.classification,
        RecoveryClassification::CompleteOrphan
    );
    assert_eq!(artifact.length, 337);
    let segment = fixture_bytes(SEGMENT)?;
    assert_eq!(
        artifact.identity_digest.as_bytes().as_slice(),
        segment
            .get(SEGMENT_DIGEST_OFFSET..SEGMENT_DIGEST_OFFSET + 32)
            .ok_or("segment digest")?
    );
    assert_eq!(
        artifact.content_digest,
        CanonicalRecoveryDispositionReceipt::artifact_content_digest(&segment)
    );
    assert_eq!(
        bytes
            .get(CONTENT_DIGEST_OFFSET..CONTENT_DIGEST_OFFSET + 32)
            .ok_or("content digest")?,
        artifact.content_digest.as_bytes().as_slice()
    );
    let coordinates = receipt.coordinates();
    assert_eq!(coordinates.publication_generation.get(), 2);
    assert_eq!(coordinates.catalog_generation.get(), 2);
    assert!(matches!(
        coordinates.retention,
        GcRetentionState::Published { generation, .. } if generation.get() == 1
    ));
    assert_eq!(
        (
            coordinates.reader_lock.device().get(),
            coordinates.reader_lock.mount().get(),
            coordinates.reader_lock.file().get()
        ),
        (4, 5, 6)
    );

    let canonical = CanonicalRecoveryDispositionReceipt::from_receipt(&receipt);
    assert_eq!(canonical.encoded(), bytes);
    assert_eq!(canonical.receipt(), &receipt);
    Ok(())
}

#[test]
fn every_registered_code_round_trips_and_matches_the_definition()
-> Result<(), Box<dyn std::error::Error>> {
    for kind in RecoveryArtifactKind::ALL {
        assert_eq!(RecoveryArtifactKind::from_code(kind.code()), Some(*kind));
    }
    for decision in RecoveryDispositionDecision::ALL {
        assert_eq!(
            RecoveryDispositionDecision::from_code(decision.code()),
            Some(*decision)
        );
    }
    for classification in RecoveryClassification::ALL {
        assert_eq!(
            RecoveryClassification::from_code(classification.code()),
            Some(*classification)
        );
    }
    assert_eq!(RecoveryArtifactKind::from_code(0), None);
    assert_eq!(RecoveryDispositionDecision::from_code(3), None);
    assert_eq!(RecoveryClassification::from_code(u16::MAX), None);

    let registered = |key: &str| -> Result<String, Box<dyn std::error::Error>> {
        DEFINITION
            .lines()
            .find_map(|row| {
                row.strip_prefix(key)
                    .and_then(|rest| rest.strip_prefix('\t'))
            })
            .map(str::to_owned)
            .ok_or_else(|| format!("definition lacks {key}").into())
    };
    let render = |pairs: Vec<(&str, u16)>| {
        pairs
            .into_iter()
            .map(|(name, code)| format!("{name}:{code}"))
            .collect::<Vec<_>>()
            .join(",")
    };
    assert_eq!(
        registered("recovery.disposition.artifact-kinds")?,
        render(
            RecoveryArtifactKind::ALL
                .iter()
                .map(|kind| (kind.identifier(), kind.code()))
                .collect()
        )
    );
    assert_eq!(
        registered("recovery.disposition.decisions")?,
        render(
            RecoveryDispositionDecision::ALL
                .iter()
                .map(|decision| (decision.identifier(), decision.code()))
                .collect()
        )
    );
    assert_eq!(
        registered("recovery.disposition.classifications")?,
        render(
            RecoveryClassification::ALL
                .iter()
                .map(|class| (class.identifier(), class.code()))
                .collect()
        )
    );
    Ok(())
}

#[test]
fn every_structural_field_has_one_exact_first_refusal() -> Result<(), Box<dyn std::error::Error>> {
    let canonical = fixture_bytes(DISPOSITION)?;
    for mutation in MATRIX {
        let mut bytes = canonical.clone();
        (mutation.mutate)(&mut bytes)?;
        if mutation.reseal {
            let checksum = domain_hash(
                CHECKSUM_DOMAIN,
                bytes.get(..CHECKSUM_OFFSET).ok_or("checksum preimage")?,
            );
            patch(&mut bytes, CHECKSUM_OFFSET, &checksum)?;
        }
        let error = AdmittedRecoveryDispositionReceipt::decode(&bytes)
            .err()
            .ok_or_else(|| format!("mutated {} was admitted", mutation.field))?;
        assert!(
            (mutation.refuses)(&error),
            "{} reached the wrong refusal: {error:?}",
            mutation.field
        );
    }
    Ok(())
}

#[test]
fn liveness_zero_beside_the_empty_retention_digest_round_trips_as_empty_retention()
-> Result<(), Box<dyn std::error::Error>> {
    let bytes = fixture_bytes(DISPOSITION)?;
    let receipt = *AdmittedRecoveryDispositionReceipt::decode(&bytes)?.receipt();
    let mut coordinates = receipt.coordinates();
    coordinates.retention = GcRetentionState::Empty;
    let empty = RecoveryDispositionReceipt::new(
        receipt.artifact(),
        receipt.decision(),
        coordinates,
        receipt.evidence_digest(),
    );

    let canonical = CanonicalRecoveryDispositionReceipt::from_receipt(&empty);

    assert_eq!(
        canonical.encoded().get(184..192),
        Some(0_u64.to_be_bytes().as_slice())
    );
    let admitted = AdmittedRecoveryDispositionReceipt::decode(canonical.encoded())?;
    assert_eq!(
        admitted.receipt().coordinates().retention,
        GcRetentionState::Empty
    );
    Ok(())
}

#[test]
fn framing_refuses_truncation_and_trailing_bytes() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fixture_bytes(DISPOSITION)?;
    let mut truncated = bytes.clone();
    assert!(truncated.pop().is_some());
    assert!(matches!(
        AdmittedRecoveryDispositionReceipt::decode(&truncated),
        Err(Refusal::WrongLength {
            expected: 320,
            observed: 319
        })
    ));
    let mut trailing = bytes;
    trailing.push(0);
    assert!(matches!(
        AdmittedRecoveryDispositionReceipt::decode(&trailing),
        Err(Refusal::WrongLength {
            expected: 320,
            observed: 321
        })
    ));
    Ok(())
}

fn fixture_bytes(hex: &str) -> Result<Vec<u8>, io::Error> {
    support::decode_hex(hex.trim_end())
}
