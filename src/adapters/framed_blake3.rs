//! Named version-1 framed BLAKE3 boundary primitive.

use blake3::Hasher;

const VERSION: u16 = 1;
const ALGORITHM: u8 = 1;

pub(super) fn hash(domain: &[u8], parts: &[&[u8]], length: u64) -> [u8; 32] {
    let mut state = State::new(domain);
    for part in parts {
        state.update(part);
    }
    state.finalize(length)
}

pub(super) struct State {
    hasher: Hasher,
}

impl State {
    pub(super) fn new(domain: &[u8]) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(domain);
        hasher.update(&VERSION.to_be_bytes());
        hasher.update(&[ALGORITHM]);
        Self { hasher }
    }

    pub(super) fn update(&mut self, bytes: &[u8]) {
        self.hasher.update(bytes);
    }

    pub(super) fn finalize(mut self, length: u64) -> [u8; 32] {
        self.hasher.update(&length.to_be_bytes());
        *self.hasher.finalize().as_bytes()
    }
}
