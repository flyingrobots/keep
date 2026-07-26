//! Golden-vector tests for canonical chunk identity.

use std::error::Error;
use std::io;

use super::ChunkId;

const IDENTITIES: &str = include_str!("../../conformance/chunk-id/v1/identities.tsv");
const SCHEMA: &str = "keep.chunk-identities/v1";
const COLUMNS: &str = "case\trecipe\tparameter\tcount\tchunk_length\tdigest_hex";
const MAXIMUM_FIXTURE_BYTES: usize = 262_144;

#[test]
fn public_chunk_identity_matches_every_golden_vector() -> Result<(), Box<dyn Error>> {
    let mut lines = IDENTITIES.lines();
    if lines.next() != Some(SCHEMA) || lines.next() != Some(COLUMNS) {
        return Err(io::Error::other("chunk identity fixture header moved").into());
    }
    for line in lines {
        let fixture = parse_fixture(line)?;
        let observed = ChunkId::hash_bytes(&fixture.bytes)?;
        assert_eq!(
            observed.digest, fixture.digest,
            "ChunkId moved for {}",
            fixture.name
        );
        assert_eq!(observed.length().get(), fixture.length);
    }
    Ok(())
}

struct Fixture {
    name: String,
    bytes: Vec<u8>,
    length: u32,
    digest: [u8; 32],
}

fn parse_fixture(line: &str) -> Result<Fixture, Box<dyn Error>> {
    let mut fields = line.split('\t');
    let name = field(&mut fields)?.to_owned();
    let recipe = field(&mut fields)?;
    let parameter = decode_hex(field(&mut fields)?)?;
    let count = field(&mut fields)?.parse::<usize>()?;
    let length = field(&mut fields)?.parse::<u32>()?;
    let digest = decode_hex(field(&mut fields)?)?
        .try_into()
        .map_err(|_source| io::Error::other("fixture digest width moved"))?;
    if fields.next().is_some() {
        return Err(io::Error::other("fixture row has trailing fields").into());
    }
    let bytes = fixture_bytes(recipe, &parameter, count)?;
    if bytes.len() != usize::try_from(length)? {
        return Err(io::Error::other("fixture length moved").into());
    }
    Ok(Fixture {
        name,
        bytes,
        length,
        digest,
    })
}

fn fixture_bytes(recipe: &str, parameter: &[u8], count: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    if (recipe == "repeated-byte-v1" && parameter.len() != 1)
        || (recipe != "repeated-byte-v1" && recipe != "hex-repeat-v1")
        || parameter.is_empty()
    {
        return Err(io::Error::other("unsupported fixture recipe").into());
    }
    let length = parameter
        .len()
        .checked_mul(count)
        .ok_or_else(|| io::Error::other("fixture length overflow"))?;
    if length > MAXIMUM_FIXTURE_BYTES {
        return Err(io::Error::other("fixture exceeds chunk bound").into());
    }
    Ok(parameter.repeat(count))
}

fn decode_hex(encoded: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    if !encoded.len().is_multiple_of(2) {
        return Err(io::Error::other("fixture hex has odd length").into());
    }
    let capacity = encoded
        .len()
        .checked_div(2)
        .ok_or_else(|| io::Error::other("fixture hex division failed"))?;
    let mut bytes = Vec::with_capacity(capacity);
    for pair in encoded.as_bytes().chunks_exact(2) {
        let high = pair
            .first()
            .copied()
            .ok_or_else(|| io::Error::other("fixture hex pair is empty"))?;
        let low = pair
            .get(1)
            .copied()
            .ok_or_else(|| io::Error::other("fixture hex pair is truncated"))?;
        bytes.push((nibble(high)? << 4) | nibble(low)?);
    }
    Ok(bytes)
}

fn field<'a>(fields: &mut std::str::Split<'a, char>) -> Result<&'a str, Box<dyn Error>> {
    fields
        .next()
        .ok_or_else(|| io::Error::other("fixture row is missing a field").into())
}

fn nibble(value: u8) -> Result<u8, io::Error> {
    match value {
        b'0' => Ok(0),
        b'1' => Ok(1),
        b'2' => Ok(2),
        b'3' => Ok(3),
        b'4' => Ok(4),
        b'5' => Ok(5),
        b'6' => Ok(6),
        b'7' => Ok(7),
        b'8' => Ok(8),
        b'9' => Ok(9),
        b'a' => Ok(10),
        b'b' => Ok(11),
        b'c' => Ok(12),
        b'd' => Ok(13),
        b'e' => Ok(14),
        b'f' => Ok(15),
        _ => Err(io::Error::other("fixture hex is noncanonical")),
    }
}
