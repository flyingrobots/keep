//! Segment-seal physical-digest and checksum corruption laws.

use std::error::Error;

use keep::{SegmentSeal, SegmentSealError};

use super::{canonical_empty_parts, mutate_seal};

#[test]
fn segment_digest_binds_every_supplied_prefix_byte() -> Result<(), Box<dyn Error>> {
    let parts = canonical_empty_parts()?;
    let mut prefix = parts.prefix;
    let seal = parts.seal;
    let canonical = SegmentSeal::decode(&prefix, &seal)?;
    let first = prefix
        .first_mut()
        .ok_or("segment prefix must not be empty")?;
    *first ^= u8::MAX;

    let error = match SegmentSeal::decode(&prefix, &seal) {
        Ok(_admitted) => return Err("a changed prefix must invalidate the physical digest".into()),
        Err(error) => error,
    };
    let SegmentSealError::SegmentDigestMismatch { expected, observed } = error else {
        return Err(format!("unexpected segment-prefix refusal: {error}").into());
    };
    assert_ne!(expected, observed);
    assert_eq!(observed, canonical.digest());
    Ok(())
}

#[test]
fn segment_digest_bytes_are_admitted_before_the_seal_checksum() -> Result<(), Box<dyn Error>> {
    let parts = canonical_empty_parts()?;
    let prefix = parts.prefix;
    let seal = parts.seal;
    let canonical = SegmentSeal::decode(&prefix, &seal)?;
    let digest_first = seal
        .get(64)
        .copied()
        .ok_or("segment seal lacks its digest")?
        ^ u8::MAX;
    let mutated = mutate_seal(&seal, 64, &[digest_first])?;

    let error = match SegmentSeal::decode(&prefix, &mutated) {
        Ok(_admitted) => return Err("a changed physical digest must be refused".into()),
        Err(error) => error,
    };
    let SegmentSealError::SegmentDigestMismatch { expected, observed } = error else {
        return Err(format!("unexpected segment-digest refusal: {error}").into());
    };
    assert_eq!(expected, canonical.digest());
    assert_ne!(observed, canonical.digest());
    Ok(())
}

#[test]
fn segment_seal_checksum_binds_the_admitted_digest() -> Result<(), Box<dyn Error>> {
    let parts = canonical_empty_parts()?;
    let prefix = parts.prefix;
    let seal = parts.seal;
    let expected = seal
        .get(96..128)
        .ok_or("segment seal lacks its checksum")?
        .try_into()?;
    let checksum_first = seal
        .get(96)
        .copied()
        .ok_or("segment seal lacks its checksum")?
        ^ u8::MAX;
    let mutated = mutate_seal(&seal, 96, &[checksum_first])?;
    let observed = mutated
        .get(96..128)
        .ok_or("mutated segment seal lacks its checksum")?
        .try_into()?;

    assert_eq!(
        SegmentSeal::decode(&prefix, &mutated),
        Err(SegmentSealError::SealChecksumMismatch { expected, observed })
    );
    Ok(())
}
