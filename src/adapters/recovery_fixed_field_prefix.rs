//! This module owns completion of available fixed-field recovery prefixes.

pub(super) fn observed_field<const LENGTH: usize>(
    encoded: &[u8],
    offset: usize,
    canonical: [u8; LENGTH],
) -> [u8; LENGTH] {
    let Some(available) = encoded.get(offset..) else {
        return canonical;
    };
    let mut observed = canonical;
    for (target, source) in observed.iter_mut().zip(available) {
        *target = *source;
    }
    observed
}
