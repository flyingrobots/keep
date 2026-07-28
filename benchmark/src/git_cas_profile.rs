//! Pinned git-cas default Buzhash comparison baseline.

use crate::ProfileError;

const WINDOW_SIZE: usize = 64;
const WINDOW_MASK: usize = 63;
const MINIMUM: usize = 65_536;
const TARGET: usize = 262_144;
const MAXIMUM: usize = 1_048_576;
const HARD_MASK: u32 = 0x0007_ffff;
const EASY_MASK: u32 = 0x0001_ffff;

#[allow(
    clippy::redundant_pub_crate,
    reason = "the sibling profile dispatcher is the only consumer"
)]
pub(super) fn partition(source: &[u8]) -> Result<Vec<usize>, ProfileError> {
    let capacity = source
        .len()
        .checked_div(MINIMUM)
        .and_then(|chunks| chunks.checked_add(1))
        .ok_or(ProfileError::CoordinateOverflow {
            current: source.len(),
            incoming: MINIMUM,
        })?;
    let table = generate_table();
    let mut ends = Vec::new();
    ends.try_reserve_exact(capacity)
        .map_err(|source| ProfileError::Allocation {
            target: "git-cas-boundaries",
            source,
        })?;
    scan(source, &table, &mut ends)?;
    Ok(ends)
}

fn scan(source: &[u8], table: &[u32; 256], ends: &mut Vec<usize>) -> Result<(), ProfileError> {
    let mut state = BuzState::new();
    for (index, byte) in source.iter().copied().enumerate() {
        let check_boundary = state.chunk_length >= MINIMUM;
        state.accept(byte, table)?;
        if check_boundary && state.boundary() {
            ends.push(next(index)?);
            state = BuzState::new();
        }
    }
    if ends.last().copied().unwrap_or(0) != source.len() {
        ends.push(source.len());
    }
    Ok(())
}

struct BuzState {
    hash: u32,
    window: [u8; WINDOW_SIZE],
    window_position: usize,
    hash_fed: usize,
    chunk_length: usize,
}

impl BuzState {
    const fn new() -> Self {
        Self {
            hash: 0,
            window: [0; WINDOW_SIZE],
            window_position: 0,
            hash_fed: 0,
            chunk_length: 0,
        }
    }

    fn accept(&mut self, byte: u8, table: &[u32; 256]) -> Result<(), ProfileError> {
        let incoming = gear(table, byte)?;
        if self.hash_fed < WINDOW_SIZE {
            self.hash = self.hash.rotate_left(1) ^ incoming;
            self.hash_fed =
                self.hash_fed
                    .checked_add(1)
                    .ok_or(ProfileError::CoordinateOverflow {
                        current: self.hash_fed,
                        incoming: 1,
                    })?;
        } else {
            let outgoing = self.window.get(self.window_position).copied().ok_or(
                ProfileError::CoordinateOverflow {
                    current: self.window_position,
                    incoming: 1,
                },
            )?;
            self.hash = self.hash.rotate_left(1) ^ gear(table, outgoing)? ^ incoming;
        }
        let slot =
            self.window
                .get_mut(self.window_position)
                .ok_or(ProfileError::CoordinateOverflow {
                    current: self.window_position,
                    incoming: 1,
                })?;
        *slot = byte;
        self.window_position = next(self.window_position)? & WINDOW_MASK;
        self.chunk_length =
            self.chunk_length
                .checked_add(1)
                .ok_or(ProfileError::CoordinateOverflow {
                    current: self.chunk_length,
                    incoming: 1,
                })?;
        Ok(())
    }

    const fn boundary(&self) -> bool {
        let mask = if self.chunk_length < TARGET {
            HARD_MASK
        } else {
            EASY_MASK
        };
        self.hash & mask == 0 || self.chunk_length >= MAXIMUM
    }
}

fn generate_table() -> [u32; 256] {
    let mut table = [0_u32; 256];
    let mut state = 0x6a09_e667_f3bc_c908_u64;
    for slot in &mut table {
        state ^= state.wrapping_shl(13);
        state ^= state.wrapping_shr(7);
        state ^= state.wrapping_shl(17);
        let [first, second, third, fourth, ..] = state.to_le_bytes();
        *slot = u32::from_le_bytes([first, second, third, fourth]);
    }
    table
}

fn gear(table: &[u32; 256], byte: u8) -> Result<u32, ProfileError> {
    table
        .get(usize::from(byte))
        .copied()
        .ok_or(ProfileError::MissingGearEntry { byte })
}

fn next(current: usize) -> Result<usize, ProfileError> {
    current
        .checked_add(1)
        .ok_or(ProfileError::CoordinateOverflow {
            current,
            incoming: 1,
        })
}
