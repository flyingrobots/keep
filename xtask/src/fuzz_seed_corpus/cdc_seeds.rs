//! This module owns deterministic `FastCDC` seed recipes and byte witnesses.

use super::{FuzzSeedError, MAX_SEED_BYTES, Seed};

const EXPECTED: [(&str, &str); 5] = [
    (
        "minimum",
        "111f6c2f2ac0fc43154414a6e3e4c104cb04907e9453d3ac85cc5f55cc015b48",
    ),
    (
        "short-mask-match",
        "28927d8ca26328dbfe0e17bc8792247e0482f8eef0f8600c74b652dd0dce524e",
    ),
    (
        "probe-byte-carry",
        "9d4b6dff387e803238437cb6ff5c533a83a85c5e7da520efaac2b90c12f8f2bf",
    ),
    (
        "forced-maximum",
        "56a48fec7bfb95b432d6f995255cd06c180f320e41ff62f210cb1de4b0956ce6",
    ),
    (
        "random-long",
        "6b370f49d70940cad7cc4ee27988d67117c7a6f559c62888290e28c0e9e9cafc",
    ),
];

pub(super) fn seeds() -> Result<Vec<Seed>, FuzzSeedError> {
    let recipes = [
        ("minimum", vec![0_u8; 16_384]),
        ("short-mask-match", xorshift64(9, 60_000)?),
        (
            "probe-byte-carry",
            xorshift64(0x0123_4567_89ab_cdef, 150_000)?,
        ),
        ("forced-maximum", vec![0_u8; 262_145]),
        (
            "random-long",
            xorshift64(0x0123_4567_89ab_cdef, MAX_SEED_BYTES)?,
        ),
    ];
    recipes
        .into_iter()
        .map(|(name, content)| checked_seed(name, content))
        .collect()
}

fn checked_seed(name: &'static str, content: Vec<u8>) -> Result<Seed, FuzzSeedError> {
    let expected = EXPECTED
        .iter()
        .find_map(|(candidate, digest)| (*candidate == name).then_some(*digest))
        .ok_or_else(|| FuzzSeedError::violation(format!("CDC seed {name:?} has no witness")))?;
    let observed = blake3::hash(&content).to_hex();
    if observed.as_str() != expected {
        return Err(FuzzSeedError::violation(format!(
            "CDC seed {name:?} moved from its reviewed bytes"
        )));
    }
    Seed::new("fast_cdc", name, content)
}

fn xorshift64(seed: u64, count: usize) -> Result<Vec<u8>, FuzzSeedError> {
    if seed == 0 || count > MAX_SEED_BYTES {
        return Err(FuzzSeedError::violation(
            "xorshift seed or count is outside its bound",
        ));
    }
    let mut output = Vec::with_capacity(count);
    let mut state = seed;
    for _ in 0..count {
        state ^= state.wrapping_shl(13);
        state ^= state.wrapping_shr(7);
        state ^= state.wrapping_shl(17);
        let byte = u8::try_from(state & u64::from(u8::MAX)).map_err(|source| {
            FuzzSeedError::violation(format!("xorshift byte is invalid: {source}"))
        })?;
        output.push(byte);
    }
    Ok(output)
}
