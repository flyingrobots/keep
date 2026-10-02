//! This boundary module owns admission of available big-endian closure-limit prefixes.

use super::RetentionRootDecodeError as Error;
use crate::{RetentionClosureLimit as Limit, RetentionClosureLimitError, RetentionClosureLimits};

pub(super) fn admit(encoded: &[u8]) -> Result<(), Error> {
    admit_field(encoded, 88, 8, Limit::Nodes)?;
    admit_field(encoded, 96, 2, Limit::Depth)?;
    admit_field(encoded, 100, 8, Limit::EncodedBytes)?;
    admit_field(encoded, 108, 8, Limit::PhysicalBytes)
}

fn admit_field(encoded: &[u8], offset: usize, width: usize, limit: Limit) -> Result<(), Error> {
    let available = encoded.get(offset..).unwrap_or_default();
    let length = available.len().min(width);
    let mut completed = [0_u8; 8];
    let padding = completed
        .len()
        .checked_sub(width)
        .ok_or(Error::LengthOverflow)?;
    let target = completed.get_mut(padding..).ok_or(Error::LengthOverflow)?;
    for (target, source) in target.iter_mut().zip(available) {
        *target = *source;
    }
    let minimum = u64::from_be_bytes(completed);
    let admission = RetentionClosureLimits::admit_limit(limit, minimum);
    if length == width {
        return admission
            .map(|_| ())
            .map_err(|source| Error::ClosureLimit { source });
    }
    // An unfinished all-zero field still admits a positive completion. For
    // any other prefix, its zero-filled minimum is a possible completion.
    match admission {
        Err(RetentionClosureLimitError::AboveMaximum { maximum, .. }) => {
            Err(Error::ClosureLimitPrefixAboveMaximum {
                limit,
                minimum,
                maximum,
            })
        }
        Ok(_) | Err(RetentionClosureLimitError::Zero { .. }) => Ok(()),
    }
}
