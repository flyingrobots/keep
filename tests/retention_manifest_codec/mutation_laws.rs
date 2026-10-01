//! Field-by-field corruption matrix for version-2 retention manifests.
//!
//! Every structural field of the manifest header, entry body, and trailer
//! has one mutation and one exact first refusal (`KEEP-RETENTION-003`).

use std::io;

use keep::{
    AdmittedRetentionManifest, RetentionManifestDecodeError as Refusal, RetentionManifestError,
};

use super::{
    CHECKSUM_OFFSET, ENTRY_BODY_OFFSET, ENTRY_SET_DIGEST_OFFSET, MANIFEST_DIGEST_OFFSET,
    ONE_ROOT_MANIFEST, fixture_bytes,
};
use crate::support::{counted_domain_hash, domain_hash, flip, patch, read_u32};

const HEADER_LENGTH: usize = 160;
const ENTRY_WIDTH: usize = 72;
const TRAILER_LENGTH: usize = 64;
const ENTRY_COUNT_OFFSET: usize = 44;
const MAXIMUM_ENTRY_COUNT: u32 = 4_096;

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
    refuses: fn(&Refusal) -> bool,
}

const MATRIX: &[Mutation] = &[
    Mutation {
        field: "magic",
        seal: Seal::Everything,
        mutate: |bytes| flip(bytes, 15),
        refuses: |error| matches!(error, Refusal::InvalidMagic { .. }),
    },
    Mutation {
        field: "version",
        seal: Seal::Everything,
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
        field: "header length",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, 18, &159_u16.to_be_bytes()),
        refuses: |error| {
            matches!(
                error,
                Refusal::InvalidHeaderLength {
                    expected: 160,
                    observed: 159
                }
            )
        },
    },
    Mutation {
        field: "flags",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, 20, &1_u32.to_be_bytes()),
        refuses: |error| matches!(error, Refusal::UnsupportedFlags { observed: 1 }),
    },
    Mutation {
        field: "total record length",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, 24, &295_u64.to_be_bytes()),
        refuses: |error| {
            matches!(
                error,
                Refusal::DeclaredLengthMismatch {
                    expected: 296,
                    observed: 295
                }
            )
        },
    },
    Mutation {
        field: "liveness generation zero",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, 32, &0_u64.to_be_bytes()),
        refuses: |error| matches!(error, Refusal::LivenessGeneration { .. }),
    },
    Mutation {
        field: "liveness generation two without predecessor",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, 32, &2_u64.to_be_bytes()),
        refuses: |error| {
            matches!(
                error,
                Refusal::Semantic {
                    source: RetentionManifestError::MissingPredecessor { .. }
                }
            )
        },
    },
    Mutation {
        field: "entry width",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, 40, &71_u16.to_be_bytes()),
        refuses: |error| {
            matches!(
                error,
                Refusal::InvalidEntryWidth {
                    expected: 72,
                    observed: 71
                }
            )
        },
    },
    Mutation {
        field: "reserved entry bytes",
        seal: Seal::Everything,
        mutate: |bytes| flip(bytes, 42),
        refuses: |error| matches!(error, Refusal::NonZeroReserved { field: "entry" }),
    },
    Mutation {
        field: "entry count participates in the declared length",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, ENTRY_COUNT_OFFSET, &2_u32.to_be_bytes()),
        refuses: |error| {
            matches!(
                error,
                Refusal::DeclaredLengthMismatch {
                    expected: 368,
                    observed: 296
                }
            )
        },
    },
    Mutation {
        field: "predecessor digest at generation one",
        seal: Seal::Everything,
        mutate: |bytes| flip(bytes, 48),
        refuses: |error| {
            matches!(
                error,
                Refusal::Semantic {
                    source: RetentionManifestError::InitialGenerationHasPredecessor { .. },
                }
            )
        },
    },
    Mutation {
        field: "entry-set digest",
        seal: Seal::Digests,
        mutate: |bytes| flip(bytes, ENTRY_SET_DIGEST_OFFSET),
        refuses: |error| matches!(error, Refusal::EntrySetDigestMismatch { .. }),
    },
    Mutation {
        field: "reserved trailing header bytes",
        seal: Seal::Everything,
        mutate: |bytes| flip(bytes, 159),
        refuses: |error| {
            matches!(
                error,
                Refusal::NonZeroReserved {
                    field: "trailing header"
                }
            )
        },
    },
    Mutation {
        field: "entry root generation zero",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, ENTRY_BODY_OFFSET + 32, &0_u64.to_be_bytes()),
        refuses: |error| matches!(error, Refusal::RootGeneration { index: 0, .. }),
    },
    Mutation {
        field: "manifest digest",
        seal: Seal::Checksum,
        mutate: |bytes| flip(bytes, MANIFEST_DIGEST_OFFSET),
        refuses: |error| matches!(error, Refusal::ManifestDigestMismatch { .. }),
    },
    Mutation {
        field: "checksum",
        seal: Seal::Nothing,
        mutate: |bytes| flip(bytes, CHECKSUM_OFFSET),
        refuses: |error| matches!(error, Refusal::ChecksumMismatch { .. }),
    },
];

#[test]
fn every_manifest_field_has_one_exact_first_refusal() -> Result<(), Box<dyn std::error::Error>> {
    for mutation in MATRIX {
        let mut bytes = fixture_bytes(ONE_ROOT_MANIFEST)?;
        (mutation.mutate)(&mut bytes)?;
        seal(&mut bytes, mutation.seal)?;
        let Err(error) = AdmittedRetentionManifest::decode(&bytes) else {
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
fn entry_order_and_count_ceilings_refuse_after_complete_integrity()
-> Result<(), Box<dyn std::error::Error>> {
    let entry = fixture_entry_bytes()?;
    let mut duplicated = entry.clone();
    duplicated.extend_from_slice(&entry);
    let repeated = reframe(&duplicated, 2)?;
    assert!(matches!(
        AdmittedRetentionManifest::decode(&repeated),
        Err(Refusal::NonCanonicalEntryOrder { index: 1 })
    ));

    let count = MAXIMUM_ENTRY_COUNT
        .checked_add(1)
        .ok_or_else(|| io::Error::other("entry ceiling overflows"))?;
    let body_length = usize::try_from(count)?
        .checked_mul(ENTRY_WIDTH)
        .ok_or_else(|| io::Error::other("entry body overflows"))?;
    let oversized = reframe(&vec![0_u8; body_length], count)?;
    assert!(matches!(
        AdmittedRetentionManifest::decode(&oversized),
        Err(Refusal::EntryCountExceeded {
            maximum: MAXIMUM_ENTRY_COUNT,
            observed,
        }) if observed == count
    ));
    Ok(())
}

fn fixture_entry_bytes() -> io::Result<Vec<u8>> {
    let bytes = fixture_bytes(ONE_ROOT_MANIFEST)?;
    bytes
        .get(ENTRY_BODY_OFFSET..MANIFEST_DIGEST_OFFSET)
        .map(<[u8]>::to_vec)
        .ok_or_else(|| io::Error::other("frozen manifest lacks its entry body"))
}

/// Builds a manifest from the frozen header with a replaced entry body.
fn reframe(entries: &[u8], entry_count: u32) -> io::Result<Vec<u8>> {
    let fixture = fixture_bytes(ONE_ROOT_MANIFEST)?;
    let mut bytes = fixture
        .get(..HEADER_LENGTH)
        .map(<[u8]>::to_vec)
        .ok_or_else(|| io::Error::other("frozen manifest lacks its header"))?;
    bytes.extend_from_slice(entries);
    bytes.extend_from_slice(&[0_u8; TRAILER_LENGTH]);
    let total_length = u64::try_from(bytes.len())
        .map_err(|_| io::Error::other("record exceeds the u64 length field"))?;
    patch(&mut bytes, 24, &total_length.to_be_bytes())?;
    patch(&mut bytes, ENTRY_COUNT_OFFSET, &entry_count.to_be_bytes())?;
    seal(&mut bytes, Seal::Everything)?;
    Ok(bytes)
}

fn seal(bytes: &mut [u8], seal: Seal) -> io::Result<()> {
    let checksum_offset = bytes
        .len()
        .checked_sub(32)
        .ok_or_else(|| io::Error::other("retention manifest lacks a checksum"))?;
    let digest_offset = checksum_offset
        .checked_sub(32)
        .ok_or_else(|| io::Error::other("retention manifest lacks a digest"))?;
    if matches!(seal, Seal::Everything) {
        let count = read_u32(bytes, ENTRY_COUNT_OFFSET)?;
        let body = bytes
            .get(HEADER_LENGTH..digest_offset)
            .ok_or_else(|| io::Error::other("retention manifest lacks its entry body"))?;
        let digest = counted_domain_hash(b"keep.retention-manifest-entries/v2\0", count, body);
        patch(bytes, ENTRY_SET_DIGEST_OFFSET, &digest)?;
    }
    if matches!(seal, Seal::Everything | Seal::Digests) {
        let preimage = bytes
            .get(..digest_offset)
            .ok_or_else(|| io::Error::other("retention manifest lacks its digest preimage"))?;
        let digest = domain_hash(b"keep.retention-manifest/v2\0", preimage);
        patch(bytes, digest_offset, &digest)?;
    }
    if !matches!(seal, Seal::Nothing) {
        let preimage = bytes
            .get(..checksum_offset)
            .ok_or_else(|| io::Error::other("retention manifest lacks its checksum preimage"))?;
        let checksum = domain_hash(b"keep.retention-manifest-checksum/v2\0", preimage);
        patch(bytes, checksum_offset, &checksum)?;
    }
    Ok(())
}
