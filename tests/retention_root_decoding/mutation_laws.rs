//! Field-by-field corruption matrix for version-2 retention roots.
//!
//! Every structural field of the root header, body, and trailer has one
//! mutation and one exact first refusal (`KEEP-RETENTION-003`). The sealed
//! matrix recomputes every digest and checksum that the mutation did not
//! target, so each case proves the named field check and nothing else.

use std::io;

use keep::{AdmittedRetentionRoot, RetentionRootDecodeError as Refusal, RetentionRootError};

use super::{ANCHOR_BODY_OFFSET, ANCHOR_SET_DIGEST_OFFSET, ROOT_DIGEST_OFFSET, fixture_bytes};
use crate::support::{counted_domain_hash, domain_hash, flip, patch, read_u16, read_u32};

const HEADER_LENGTH: usize = 192;
const ANCHOR_WIDTH: usize = 119;
const TRAILER_LENGTH: usize = 64;
const NAMESPACE_LENGTH_OFFSET: usize = 40;
const ANCHOR_COUNT_OFFSET: usize = 44;
const MAXIMUM_ANCHOR_COUNT: u32 = 65_536;

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
        mutate: |bytes| patch(bytes, 18, &191_u16.to_be_bytes()),
        refuses: |error| {
            matches!(
                error,
                Refusal::InvalidHeaderLength {
                    expected: 192,
                    observed: 191
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
        mutate: |bytes| patch(bytes, 24, &377_u64.to_be_bytes()),
        refuses: |error| {
            matches!(
                error,
                Refusal::DeclaredLengthMismatch {
                    expected: 378,
                    observed: 377
                }
            )
        },
    },
    Mutation {
        field: "root generation zero",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, 32, &0_u64.to_be_bytes()),
        refuses: |error| matches!(error, Refusal::Generation { .. }),
    },
    Mutation {
        field: "root generation two without predecessor",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, 32, &2_u64.to_be_bytes()),
        refuses: |error| {
            matches!(
                error,
                Refusal::Semantic {
                    source: RetentionRootError::MissingPredecessor { .. }
                }
            )
        },
    },
    Mutation {
        field: "anchor width",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, 42, &118_u16.to_be_bytes()),
        refuses: |error| {
            matches!(
                error,
                Refusal::InvalidAnchorWidth {
                    expected: 119,
                    observed: 118
                }
            )
        },
    },
    Mutation {
        field: "anchor count participates in the declared length",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, ANCHOR_COUNT_OFFSET, &2_u32.to_be_bytes()),
        refuses: |error| {
            matches!(
                error,
                Refusal::DeclaredLengthMismatch {
                    expected: 497,
                    observed: 378
                }
            )
        },
    },
    Mutation {
        field: "profile identity",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, 48, &2_u32.to_be_bytes()),
        refuses: |error| matches!(error, Refusal::Profile { .. }),
    },
    Mutation {
        field: "profile version",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, 52, &2_u32.to_be_bytes()),
        refuses: |error| matches!(error, Refusal::Profile { .. }),
    },
    Mutation {
        field: "profile-definition digest",
        seal: Seal::Everything,
        mutate: |bytes| flip(bytes, 56),
        refuses: |error| matches!(error, Refusal::Profile { .. }),
    },
    Mutation {
        field: "closure-node limit zero",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, 88, &0_u64.to_be_bytes()),
        refuses: |error| matches!(error, Refusal::ClosureLimit { .. }),
    },
    Mutation {
        field: "closure-depth limit above ceiling",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, 96, &9_u16.to_be_bytes()),
        refuses: |error| matches!(error, Refusal::ClosureLimit { .. }),
    },
    Mutation {
        field: "reserved limit bytes",
        seal: Seal::Everything,
        mutate: |bytes| flip(bytes, 98),
        refuses: |error| matches!(error, Refusal::NonZeroReserved { field: "limit" }),
    },
    Mutation {
        field: "encoded-byte limit zero",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, 100, &0_u64.to_be_bytes()),
        refuses: |error| matches!(error, Refusal::ClosureLimit { .. }),
    },
    Mutation {
        field: "physical-byte limit zero",
        seal: Seal::Everything,
        mutate: |bytes| patch(bytes, 108, &0_u64.to_be_bytes()),
        refuses: |error| matches!(error, Refusal::ClosureLimit { .. }),
    },
    Mutation {
        field: "predecessor digest at generation one",
        seal: Seal::Everything,
        mutate: |bytes| flip(bytes, 116),
        refuses: |error| {
            matches!(
                error,
                Refusal::Semantic {
                    source: RetentionRootError::InitialGenerationHasPredecessor { .. },
                }
            )
        },
    },
    Mutation {
        field: "anchor-set digest",
        seal: Seal::Digests,
        mutate: |bytes| flip(bytes, ANCHOR_SET_DIGEST_OFFSET),
        refuses: |error| matches!(error, Refusal::AnchorSetDigestMismatch { .. }),
    },
    Mutation {
        field: "reserved trailing header bytes",
        seal: Seal::Everything,
        mutate: |bytes| flip(bytes, 191),
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
        field: "anchor blob identity",
        seal: Seal::Everything,
        mutate: |bytes| flip(bytes, ANCHOR_BODY_OFFSET),
        refuses: |error| matches!(error, Refusal::BlobId { index: 0, .. }),
    },
    Mutation {
        field: "anchor layout identity",
        seal: Seal::Everything,
        mutate: |bytes| flip(bytes, ANCHOR_BODY_OFFSET + 59),
        refuses: |error| matches!(error, Refusal::LayoutId { index: 0, .. }),
    },
    Mutation {
        field: "root digest",
        seal: Seal::Checksum,
        mutate: |bytes| flip(bytes, ROOT_DIGEST_OFFSET),
        refuses: |error| matches!(error, Refusal::RootDigestMismatch { .. }),
    },
    Mutation {
        field: "checksum",
        seal: Seal::Nothing,
        mutate: |bytes| flip(bytes, 377),
        refuses: |error| matches!(error, Refusal::ChecksumMismatch { .. }),
    },
];

#[test]
fn every_root_field_has_one_exact_first_refusal() -> Result<(), Box<dyn std::error::Error>> {
    for mutation in MATRIX {
        let mut bytes = fixture_bytes()?;
        (mutation.mutate)(&mut bytes)?;
        seal(&mut bytes, mutation.seal)?;
        let Err(error) = AdmittedRetentionRoot::decode(&bytes) else {
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
fn namespace_length_bounds_refuse_after_complete_integrity()
-> Result<(), Box<dyn std::error::Error>> {
    let anchors = fixture_anchors()?;
    let empty = reframe(&[], &anchors, 1)?;
    assert!(matches!(
        AdmittedRetentionRoot::decode(&empty),
        Err(Refusal::Namespace { .. })
    ));

    let too_long = reframe(&[0x2f; 256], &anchors, 1)?;
    assert!(matches!(
        AdmittedRetentionRoot::decode(&too_long),
        Err(Refusal::Namespace { .. })
    ));
    Ok(())
}

#[test]
fn anchor_order_and_count_ceilings_refuse_after_complete_integrity()
-> Result<(), Box<dyn std::error::Error>> {
    let anchor = fixture_anchors()?;
    let mut duplicated = anchor.clone();
    duplicated.extend_from_slice(&anchor);
    let repeated = reframe(&[0x00, 0x2f, 0xff], &duplicated, 2)?;
    assert!(matches!(
        AdmittedRetentionRoot::decode(&repeated),
        Err(Refusal::NonCanonicalAnchorOrder { index: 1 })
    ));

    let count = MAXIMUM_ANCHOR_COUNT
        .checked_add(1)
        .ok_or_else(|| io::Error::other("anchor ceiling overflows"))?;
    let body_length = usize::try_from(count)?
        .checked_mul(ANCHOR_WIDTH)
        .ok_or_else(|| io::Error::other("anchor body overflows"))?;
    let oversized = reframe(&[0x00, 0x2f, 0xff], &vec![0_u8; body_length], count)?;
    assert!(matches!(
        AdmittedRetentionRoot::decode(&oversized),
        Err(Refusal::AnchorCountExceeded {
            maximum: MAXIMUM_ANCHOR_COUNT,
            observed,
        }) if observed == count
    ));
    Ok(())
}

fn fixture_anchors() -> io::Result<Vec<u8>> {
    let bytes = fixture_bytes()?;
    bytes
        .get(ANCHOR_BODY_OFFSET..ROOT_DIGEST_OFFSET)
        .map(<[u8]>::to_vec)
        .ok_or_else(|| io::Error::other("frozen retention root lacks its anchor body"))
}

/// Builds a root from the frozen header with a replaced namespace and body.
fn reframe(namespace: &[u8], anchors: &[u8], anchor_count: u32) -> io::Result<Vec<u8>> {
    let fixture = fixture_bytes()?;
    let mut bytes = fixture
        .get(..HEADER_LENGTH)
        .map(<[u8]>::to_vec)
        .ok_or_else(|| io::Error::other("frozen retention root lacks its header"))?;
    bytes.extend_from_slice(namespace);
    bytes.extend_from_slice(anchors);
    bytes.extend_from_slice(&[0_u8; TRAILER_LENGTH]);
    let namespace_length = u16::try_from(namespace.len())
        .map_err(|_| io::Error::other("namespace exceeds the u16 length field"))?;
    let total_length = u64::try_from(bytes.len())
        .map_err(|_| io::Error::other("record exceeds the u64 length field"))?;
    patch(&mut bytes, 24, &total_length.to_be_bytes())?;
    patch(
        &mut bytes,
        NAMESPACE_LENGTH_OFFSET,
        &namespace_length.to_be_bytes(),
    )?;
    patch(&mut bytes, ANCHOR_COUNT_OFFSET, &anchor_count.to_be_bytes())?;
    seal(&mut bytes, Seal::Everything)?;
    Ok(bytes)
}

fn seal(bytes: &mut [u8], seal: Seal) -> io::Result<()> {
    let checksum_offset = bytes
        .len()
        .checked_sub(32)
        .ok_or_else(|| io::Error::other("retention root lacks a checksum"))?;
    let digest_offset = checksum_offset
        .checked_sub(32)
        .ok_or_else(|| io::Error::other("retention root lacks a digest"))?;
    if matches!(seal, Seal::Everything) {
        let body_offset = HEADER_LENGTH
            .checked_add(usize::from(read_u16(bytes, NAMESPACE_LENGTH_OFFSET)?))
            .ok_or_else(|| io::Error::other("namespace length overflows"))?;
        let count = read_u32(bytes, ANCHOR_COUNT_OFFSET)?;
        let body = bytes
            .get(body_offset..digest_offset)
            .ok_or_else(|| io::Error::other("retention root lacks its anchor body"))?;
        let digest = counted_domain_hash(b"keep.retention-anchor-set/v2\0", count, body);
        patch(bytes, ANCHOR_SET_DIGEST_OFFSET, &digest)?;
    }
    if matches!(seal, Seal::Everything | Seal::Digests) {
        let preimage = bytes
            .get(..digest_offset)
            .ok_or_else(|| io::Error::other("retention root lacks its digest preimage"))?;
        let digest = domain_hash(b"keep.retention-root/v2\0", preimage);
        patch(bytes, digest_offset, &digest)?;
    }
    if !matches!(seal, Seal::Nothing) {
        let preimage = bytes
            .get(..checksum_offset)
            .ok_or_else(|| io::Error::other("retention root lacks its checksum preimage"))?;
        let checksum = domain_hash(b"keep.retention-root-checksum/v2\0", preimage);
        patch(bytes, checksum_offset, &checksum)?;
    }
    Ok(())
}
