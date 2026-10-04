//! Canonical GC retirement intent laws.

#[path = "gc_retirement_intent/mutation_laws.rs"]
mod mutation_laws;
#[path = "gc_retirement_intent/opaque_coordinate_laws.rs"]
mod opaque_coordinate_laws;
mod support;

use std::io;

use keep::{
    AdmittedGcRetirementIntent, CanonicalGcRetirementIntent, GcRetirementIntent,
    GcRetirementIntentError,
};

pub(crate) const GC_INTENT: &str =
    include_str!("../conformance/segment-store/v2/one-candidate-gc-intent.hex");
pub(crate) const INTENT_DIGEST: [u8; 32] = [
    0xa9, 0xdd, 0x52, 0x33, 0x26, 0xa6, 0x86, 0xb8, 0x9a, 0xc8, 0xf0, 0xc4, 0x10, 0x42, 0x17, 0x66,
    0xaf, 0xc2, 0x21, 0xf2, 0x96, 0x3b, 0xda, 0xfd, 0x43, 0x0d, 0xce, 0x7f, 0x59, 0xed, 0xc7, 0x01,
];
pub(crate) const HEADER_LENGTH: usize = 320;
pub(crate) const CANDIDATE_WIDTH: usize = 72;
pub(crate) const CANDIDATE_SET_DIGEST_OFFSET: usize = 288;
pub(crate) const INTENT_DIGEST_OFFSET: usize = 392;
pub(crate) const CHECKSUM_OFFSET: usize = 424;

#[test]
fn frozen_intent_decodes_and_reencodes_canonically() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fixture_bytes()?;
    let admitted = AdmittedGcRetirementIntent::decode(&bytes)?;
    assert_eq!(admitted.encoded(), bytes);
    assert_eq!(admitted.digest().as_bytes(), &INTENT_DIGEST);
    assert_eq!(
        admitted.candidate_set_digest().as_bytes(),
        bytes
            .get(CANDIDATE_SET_DIGEST_OFFSET..HEADER_LENGTH)
            .ok_or_else(|| io::Error::other("frozen intent lacks its candidate-set digest"))?
    );

    let intent = admitted.intent();
    let coordinates = intent.coordinates();
    assert_eq!(coordinates.generation.get(), 1);
    assert_eq!(coordinates.liveness_generation.get(), 1);
    assert_eq!(coordinates.catalog_generation.get(), 2);
    assert_eq!(coordinates.profile.identity(), 1);
    assert_eq!(coordinates.reader_lock.device().get(), 4);
    assert_eq!(coordinates.reader_lock.mount().get(), 5);
    assert_eq!(coordinates.reader_lock.file().get(), 6);
    assert_eq!(intent.candidate_count(), 1);
    let candidate = intent
        .candidates()
        .first()
        .ok_or_else(|| io::Error::other("frozen intent names no candidate"))?;
    assert_eq!(candidate.segment_length(), 337);
    assert_eq!(
        candidate.segment_digest().as_bytes(),
        bytes
            .get(HEADER_LENGTH..HEADER_LENGTH + 32)
            .ok_or_else(|| io::Error::other("frozen intent lacks its candidate digest"))?
    );

    let canonical = CanonicalGcRetirementIntent::from_intent(intent)?;
    assert_eq!(canonical.encoded(), bytes);
    assert_eq!(canonical.digest(), admitted.digest());
    assert_eq!(
        canonical.candidate_set_digest(),
        admitted.candidate_set_digest()
    );
    assert_eq!(canonical.intent(), intent);
    Ok(())
}

#[test]
fn semantic_intent_refuses_empty_and_repeated_candidate_sets()
-> Result<(), Box<dyn std::error::Error>> {
    let bytes = fixture_bytes()?;
    let admitted = AdmittedGcRetirementIntent::decode(&bytes)?;
    let coordinates = *admitted.intent().coordinates();
    let candidate = *admitted
        .intent()
        .candidates()
        .first()
        .ok_or_else(|| io::Error::other("frozen intent names no candidate"))?;

    assert!(matches!(
        GcRetirementIntent::new(coordinates, Vec::new()),
        Err(GcRetirementIntentError::NoCandidates)
    ));
    assert!(matches!(
        GcRetirementIntent::new(coordinates, vec![candidate, candidate]),
        Err(GcRetirementIntentError::DuplicateCandidate { index: 1 })
    ));
    let one = GcRetirementIntent::new(coordinates, vec![candidate])?;
    assert_eq!(one, *admitted.intent());
    Ok(())
}

pub(crate) fn fixture_bytes() -> Result<Vec<u8>, io::Error> {
    let encoded = GC_INTENT
        .strip_suffix('\n')
        .ok_or_else(|| io::Error::other("GC intent fixture lacks final newline"))?;
    support::decode_hex(encoded)
}
