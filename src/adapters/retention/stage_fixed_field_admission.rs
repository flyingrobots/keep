//! This boundary module owns bytewise admission of available fixed stage fields.

pub(super) fn admit<E>(
    encoded: &[u8],
    fields: &[(usize, &[u8])],
    mismatch: impl Fn(usize, u8, u8) -> E,
) -> Result<(), E> {
    for &(start, canonical) in fields {
        for (&expected, (offset, &observed)) in
            canonical.iter().zip(encoded.iter().enumerate().skip(start))
        {
            if expected != observed {
                return Err(mismatch(offset, expected, observed));
            }
        }
    }
    Ok(())
}
