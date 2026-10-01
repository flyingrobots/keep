//! Canonical GC retirement receipt laws.

mod support;

use std::io;

use keep::{
    AdmittedGcRetirementIntent, AdmittedGcRetirementReceipt, CanonicalGcRetirementIntent,
    CanonicalGcRetirementReceipt, GcRetirementReceiptDecodeError as Refusal, PoolStateDigest,
    ReaderLockCoordinate,
};

use crate::support::{domain_hash, flip, patch};

const GC_INTENT: &str = include_str!("../conformance/segment-store/v2/one-candidate-gc-intent.hex");
const GC_RECEIPT: &str =
    include_str!("../conformance/segment-store/v2/one-candidate-gc-receipt.hex");
const POOL_STATE_OFFSET: usize = 96;
const CHECKSUM_OFFSET: usize = 288;

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
        field: "GC generation",
        reseal: true,
        mutate: |bytes| patch(bytes, 24, &2_u64.to_be_bytes()),
        refuses: |error| {
            matches!(
                error,
                Refusal::GenerationMismatch {
                    expected: 1,
                    observed: 2
                }
            )
        },
    },
    Mutation {
        field: "intent digest",
        reseal: true,
        mutate: |bytes| flip(bytes, 32),
        refuses: |error| matches!(error, Refusal::IntentDigestMismatch { .. }),
    },
    Mutation {
        field: "retired candidate-set digest",
        reseal: true,
        mutate: |bytes| flip(bytes, 64),
        refuses: |error| matches!(error, Refusal::RetiredSetDigestMismatch { .. }),
    },
    Mutation {
        field: "liveness generation",
        reseal: true,
        mutate: |bytes| patch(bytes, 128, &2_u64.to_be_bytes()),
        refuses: |error| {
            matches!(
                error,
                Refusal::LivenessGenerationMismatch {
                    expected: 1,
                    observed: 2
                }
            )
        },
    },
    Mutation {
        field: "retention-manifest digest",
        reseal: true,
        mutate: |bytes| flip(bytes, 136),
        refuses: |error| matches!(error, Refusal::ManifestDigestMismatch { .. }),
    },
    Mutation {
        field: "catalog generation",
        reseal: true,
        mutate: |bytes| patch(bytes, 168, &3_u64.to_be_bytes()),
        refuses: |error| {
            matches!(
                error,
                Refusal::CatalogGenerationMismatch {
                    expected: 2,
                    observed: 3
                }
            )
        },
    },
    Mutation {
        field: "catalog digest",
        reseal: true,
        mutate: |bytes| flip(bytes, 176),
        refuses: |error| matches!(error, Refusal::CatalogDigestMismatch { .. }),
    },
    Mutation {
        field: "reader-lock device",
        reseal: true,
        mutate: |bytes| patch(bytes, 208, &9_u64.to_be_bytes()),
        refuses: |error| {
            matches!(
                error,
                Refusal::ReaderLockMismatch {
                    coordinate: ReaderLockCoordinate::Device,
                    expected: 4,
                    observed: 9,
                }
            )
        },
    },
    Mutation {
        field: "reader-lock mount",
        reseal: true,
        mutate: |bytes| patch(bytes, 216, &9_u64.to_be_bytes()),
        refuses: |error| {
            matches!(
                error,
                Refusal::ReaderLockMismatch {
                    coordinate: ReaderLockCoordinate::Mount,
                    ..
                }
            )
        },
    },
    Mutation {
        field: "reader-lock file",
        reseal: true,
        mutate: |bytes| patch(bytes, 224, &9_u64.to_be_bytes()),
        refuses: |error| {
            matches!(
                error,
                Refusal::ReaderLockMismatch {
                    coordinate: ReaderLockCoordinate::File,
                    ..
                }
            )
        },
    },
    Mutation {
        field: "synchronization count",
        reseal: true,
        mutate: |bytes| patch(bytes, 232, &2_u64.to_be_bytes()),
        refuses: |error| {
            matches!(
                error,
                Refusal::SynchronizationCountMismatch {
                    expected: 1,
                    observed: 2
                }
            )
        },
    },
    Mutation {
        field: "reserved bytes",
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
];

#[test]
fn frozen_receipt_completes_the_frozen_intent_and_reencodes_canonically()
-> Result<(), Box<dyn std::error::Error>> {
    let intent_bytes = fixture_bytes(GC_INTENT)?;
    let receipt_bytes = fixture_bytes(GC_RECEIPT)?;
    let intent = AdmittedGcRetirementIntent::decode(&intent_bytes)?;
    let admitted = AdmittedGcRetirementReceipt::decode(&receipt_bytes, &intent)?;
    assert_eq!(admitted.encoded(), receipt_bytes);
    let receipt = admitted.receipt();
    assert_eq!(
        receipt.generation(),
        intent.intent().coordinates().generation
    );
    assert_eq!(receipt.intent_digest(), intent.digest());
    assert_eq!(
        receipt.retired_candidate_set_digest(),
        intent.candidate_set_digest()
    );
    assert_eq!(receipt.synchronization_count(), 1);
    assert_eq!(receipt.reader_lock().file(), 6);

    let canonical_intent = CanonicalGcRetirementIntent::from_intent(intent.intent())?;
    let canonical =
        CanonicalGcRetirementReceipt::from_intent(&canonical_intent, receipt.pool_state_digest());
    assert_eq!(canonical.encoded(), receipt_bytes);
    assert_eq!(canonical.receipt(), receipt);
    Ok(())
}

#[test]
fn every_receipt_field_has_one_exact_first_refusal() -> Result<(), Box<dyn std::error::Error>> {
    let intent_bytes = fixture_bytes(GC_INTENT)?;
    let intent = AdmittedGcRetirementIntent::decode(&intent_bytes)?;
    for mutation in MATRIX {
        let mut bytes = fixture_bytes(GC_RECEIPT)?;
        (mutation.mutate)(&mut bytes)?;
        if mutation.reseal {
            reseal(&mut bytes)?;
        }
        let Err(error) = AdmittedGcRetirementReceipt::decode(&bytes, &intent) else {
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
fn receipt_framing_refuses_any_length_but_the_fixed_width() -> Result<(), Box<dyn std::error::Error>>
{
    let intent_bytes = fixture_bytes(GC_INTENT)?;
    let intent = AdmittedGcRetirementIntent::decode(&intent_bytes)?;
    let bytes = fixture_bytes(GC_RECEIPT)?;
    let mut truncated = bytes.clone();
    assert!(truncated.pop().is_some());
    assert!(matches!(
        AdmittedGcRetirementReceipt::decode(&truncated, &intent),
        Err(Refusal::WrongLength {
            expected: 320,
            observed: 319,
        })
    ));
    let mut trailing = bytes;
    trailing.push(0);
    assert!(matches!(
        AdmittedGcRetirementReceipt::decode(&trailing, &intent),
        Err(Refusal::WrongLength {
            expected: 320,
            observed: 321,
        })
    ));
    Ok(())
}

#[test]
fn pool_state_digest_is_carried_not_bound_to_the_intent() -> Result<(), Box<dyn std::error::Error>>
{
    let intent_bytes = fixture_bytes(GC_INTENT)?;
    let intent = AdmittedGcRetirementIntent::decode(&intent_bytes)?;
    let mut bytes = fixture_bytes(GC_RECEIPT)?;
    flip(&mut bytes, POOL_STATE_OFFSET)?;
    reseal(&mut bytes)?;
    let admitted = AdmittedGcRetirementReceipt::decode(&bytes, &intent)?;
    let expected: [u8; 32] = bytes
        .get(POOL_STATE_OFFSET..POOL_STATE_OFFSET + 32)
        .ok_or_else(|| io::Error::other("receipt lacks its pool-state digest"))?
        .try_into()?;
    assert_eq!(
        admitted.receipt().pool_state_digest(),
        PoolStateDigest::new(expected)
    );
    Ok(())
}

fn fixture_bytes(fixture: &str) -> Result<Vec<u8>, io::Error> {
    let encoded = fixture
        .strip_suffix('\n')
        .ok_or_else(|| io::Error::other("GC fixture lacks final newline"))?;
    support::decode_hex(encoded)
}

fn reseal(bytes: &mut [u8]) -> io::Result<()> {
    let preimage = bytes
        .get(..CHECKSUM_OFFSET)
        .ok_or_else(|| io::Error::other("GC receipt lacks its checksum preimage"))?;
    let checksum = domain_hash(b"keep.gc-retirement-receipt-checksum/v2\0", preimage);
    patch(bytes, CHECKSUM_OFFSET, &checksum)
}
