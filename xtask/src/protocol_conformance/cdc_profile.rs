//! This module owns the independent CDC profile v1 conformance world.

mod boundary;
mod profile;
mod scalar_fastcdc;
mod source;

use std::path::Path;

use super::ConformanceError;
use super::corpus::Corpus;

pub(super) const LONG_MASK: u64 = 0x0000_d903_1353_0000;
pub(super) const MAXIMUM: usize = 262_144;
pub(super) const MINIMUM: usize = 16_384;
pub(super) const NORMALIZATION: u8 = 2;
pub(super) const SEED: u64 = 0;
pub(super) const SHORT_MASK: u64 = 0x0000_d907_0753_7000;
pub(super) const STATE_WIDTH: u8 = 64;
pub(super) const TARGET: usize = 65_536;

pub(super) type GearTable = [u64; 256];

pub(super) fn check(repository_root: &Path) -> Result<(), ConformanceError> {
    let corpus = Corpus::open(repository_root.join("conformance/cdc-profile/v1"))?;
    let gear = profile::check(&corpus)?;
    let sources = source::load(&corpus)?;
    let boundaries = boundary::read(&corpus, &sources)?;
    boundary::validate(&sources, &boundaries, &gear)
}
