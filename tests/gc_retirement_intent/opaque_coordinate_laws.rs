//! Opaque GC coordinates are retained and bound by the intent digest.
//! Oracle: KEEP-GC-001's wire fields and the independently frozen intent.
//! Size: small (in-memory codec); execution-profile limits are recorded in the landing evidence.

use std::io;

use keep::{AdmittedGcRetirementIntent, GcCandidate, GcRetirementIntent};

use super::{
    CANDIDATE_SET_DIGEST_OFFSET, CHECKSUM_OFFSET, HEADER_LENGTH, INTENT_DIGEST,
    INTENT_DIGEST_OFFSET, fixture_bytes,
};
use crate::support::{counted_domain_hash, domain_hash, flip, patch};

struct Coordinate {
    name: &'static str,
    offset: usize,
    width: usize,
    observed: fn(&GcRetirementIntent) -> io::Result<Vec<u8>>,
}

const COORDINATES: &[Coordinate] = &[
    Coordinate {
        name: "manifest digest",
        offset: 56,
        width: 32,
        observed: |intent| Ok(intent.coordinates().manifest_digest.as_bytes().to_vec()),
    },
    Coordinate {
        name: "catalog digest",
        offset: 96,
        width: 32,
        observed: |intent| Ok(intent.coordinates().catalog_digest.as_bytes().to_vec()),
    },
    Coordinate {
        name: "catalog successor proof",
        offset: 168,
        width: 32,
        observed: |intent| {
            Ok(intent
                .coordinates()
                .catalog_successor_proof_digest
                .as_bytes()
                .to_vec())
        },
    },
    Coordinate {
        name: "segment pool identity",
        offset: 200,
        width: 32,
        observed: |intent| {
            Ok(intent
                .coordinates()
                .segment_pool_identity_digest
                .as_bytes()
                .to_vec())
        },
    },
    Coordinate {
        name: "disposition set",
        offset: 232,
        width: 32,
        observed: |intent| {
            Ok(intent
                .coordinates()
                .disposition_set_digest
                .as_bytes()
                .to_vec())
        },
    },
    Coordinate {
        name: "reader lock device",
        offset: 264,
        width: 8,
        observed: |intent| {
            Ok(intent
                .coordinates()
                .reader_lock
                .device()
                .get()
                .to_be_bytes()
                .to_vec())
        },
    },
    Coordinate {
        name: "reader lock mount",
        offset: 272,
        width: 8,
        observed: |intent| {
            Ok(intent
                .coordinates()
                .reader_lock
                .mount()
                .get()
                .to_be_bytes()
                .to_vec())
        },
    },
    Coordinate {
        name: "reader lock file",
        offset: 280,
        width: 8,
        observed: |intent| {
            Ok(intent
                .coordinates()
                .reader_lock
                .file()
                .get()
                .to_be_bytes()
                .to_vec())
        },
    },
    Coordinate {
        name: "candidate segment digest",
        offset: 320,
        width: 32,
        observed: |intent| Ok(candidate(intent)?.segment_digest().as_bytes().to_vec()),
    },
    Coordinate {
        name: "candidate segment length",
        offset: 352,
        width: 8,
        observed: |intent| Ok(candidate(intent)?.segment_length().to_be_bytes().to_vec()),
    },
    Coordinate {
        name: "candidate evidence digest",
        offset: 360,
        width: 32,
        observed: |intent| Ok(candidate(intent)?.evidence_digest().as_bytes().to_vec()),
    },
];

#[test]
fn resealed_opaque_coordinates_are_carried_into_a_distinct_intent()
-> Result<(), Box<dyn std::error::Error>> {
    for coordinate in COORDINATES {
        let mut bytes = fixture_bytes()?;
        flip(&mut bytes, coordinate.offset)?;
        seal_fixture(&mut bytes)?;
        let admitted = AdmittedGcRetirementIntent::decode(&bytes)?;
        let end = coordinate
            .offset
            .checked_add(coordinate.width)
            .ok_or_else(|| io::Error::other("coordinate end overflow"))?;
        let expected = bytes
            .get(coordinate.offset..end)
            .ok_or_else(|| io::Error::other("fixture lacks the named coordinate"))?;
        assert_eq!(
            (coordinate.observed)(admitted.intent())?,
            expected,
            "{} must retain the input coordinate",
            coordinate.name
        );
        assert_ne!(
            admitted.digest().as_bytes(),
            &INTENT_DIGEST,
            "{} must contribute to intent identity",
            coordinate.name
        );
    }
    Ok(())
}

fn candidate(intent: &GcRetirementIntent) -> io::Result<&GcCandidate> {
    intent
        .candidates()
        .first()
        .ok_or_else(|| io::Error::other("admitted intent lost its candidate"))
}

// Construct the fixed one-candidate input independently of the production encoder.
fn seal_fixture(bytes: &mut [u8]) -> io::Result<()> {
    let body = bytes
        .get(HEADER_LENGTH..INTENT_DIGEST_OFFSET)
        .ok_or_else(|| io::Error::other("fixture lacks candidate body"))?;
    let set_digest = counted_domain_hash(b"keep.gc-candidate-set/v2\0", 1, body);
    patch(bytes, CANDIDATE_SET_DIGEST_OFFSET, &set_digest)?;
    let preimage = bytes
        .get(..INTENT_DIGEST_OFFSET)
        .ok_or_else(|| io::Error::other("fixture lacks intent preimage"))?;
    let digest = domain_hash(b"keep.gc-retirement-intent/v2\0", preimage);
    patch(bytes, INTENT_DIGEST_OFFSET, &digest)?;
    let preimage = bytes
        .get(..CHECKSUM_OFFSET)
        .ok_or_else(|| io::Error::other("fixture lacks checksum preimage"))?;
    let checksum = domain_hash(b"keep.gc-retirement-intent-checksum/v2\0", preimage);
    patch(bytes, CHECKSUM_OFFSET, &checksum)
}
