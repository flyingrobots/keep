//! This boundary module owns feasibility of incomplete retention size-field combinations.

use super::{manifest_header_decoder as manifest, root_header_decoder as root};
use crate::{RetentionManifest, RetentionNamespace, RetentionRoot};

pub(super) fn root(encoded: &[u8]) -> Option<()> {
    let namespace = Bounds::read::<2>(encoded, 40)?
        .intersect(1, u64::from(RetentionNamespace::MAXIMUM_BYTE_LENGTH))?;
    let count = Bounds::read::<4>(encoded, 44)?
        .intersect(0, u64::from(RetentionRoot::MAXIMUM_ANCHOR_COUNT))?;
    possible_length(
        Bounds::read::<8>(encoded, 24)?,
        namespace,
        count,
        u64::try_from(root::HEADER_LENGTH.checked_add(root::TRAILER_LENGTH)?).ok()?,
        u64::try_from(root::ANCHOR_WIDTH).ok()?,
    )
}

pub(super) fn manifest(encoded: &[u8]) -> Option<()> {
    let count = Bounds::read::<4>(encoded, 44)?
        .intersect(0, u64::from(RetentionManifest::MAXIMUM_ENTRY_COUNT))?;
    manifest_length(Bounds::read::<8>(encoded, 24)?, count)
}

pub(super) fn head(encoded: &[u8]) -> Option<()> {
    manifest_length(
        Bounds::read::<8>(encoded, 32)?,
        Bounds {
            minimum: 0,
            maximum: u64::from(RetentionManifest::MAXIMUM_ENTRY_COUNT),
        },
    )
}

fn manifest_length(length: Bounds, count: Bounds) -> Option<()> {
    possible_length(
        length,
        Bounds {
            minimum: 0,
            maximum: 0,
        },
        count,
        u64::try_from(manifest::HEADER_LENGTH.checked_add(manifest::TRAILER_LENGTH)?).ok()?,
        u64::try_from(manifest::ENTRY_WIDTH).ok()?,
    )
}

/// Chooses the greatest count whose smallest record fits the length ceiling.
/// If its largest possible record cannot reach the length floor, no smaller
/// count can. Bounds are feasibility witnesses, never decoded observations.
fn possible_length(
    length: Bounds,
    extra: Bounds,
    count: Bounds,
    base: u64,
    stride: u64,
) -> Option<()> {
    let minimum_base = base.checked_add(extra.minimum)?;
    let greatest_count = length
        .maximum
        .checked_sub(minimum_base)?
        .checked_div(stride)?
        .min(count.maximum);
    if greatest_count < count.minimum {
        return None;
    }
    let greatest_length = base
        .checked_add(extra.maximum)?
        .checked_add(greatest_count.checked_mul(stride)?)?;
    (greatest_length >= length.minimum).then_some(())
}

#[derive(Clone, Copy)]
struct Bounds {
    minimum: u64,
    maximum: u64,
}

impl Bounds {
    fn read<const WIDTH: usize>(encoded: &[u8], offset: usize) -> Option<Self> {
        let available = encoded.get(offset..).unwrap_or_default();
        let mut minimum = [0_u8; WIDTH];
        let mut maximum = [u8::MAX; WIDTH];
        for ((low, high), observed) in minimum.iter_mut().zip(&mut maximum).zip(available) {
            *low = *observed;
            *high = *observed;
        }
        Some(Self {
            minimum: integer(&minimum)?,
            maximum: integer(&maximum)?,
        })
    }

    fn intersect(self, minimum: u64, maximum: u64) -> Option<Self> {
        let minimum = self.minimum.max(minimum);
        let maximum = self.maximum.min(maximum);
        (minimum <= maximum).then_some(Self { minimum, maximum })
    }
}

fn integer(bytes: &[u8]) -> Option<u64> {
    bytes.iter().try_fold(0_u64, |value, byte| {
        value.checked_mul(256)?.checked_add(u64::from(*byte))
    })
}
