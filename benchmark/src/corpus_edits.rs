//! Deterministic edit relationships between benchmark corpus members.

use crate::CorpusError;
use crate::corpus_bytes::reserved;

const EDIT_OFFSET: usize = 4_096;
pub(super) const EDIT_LENGTH: usize = 4_096;
const NEIGHBOR_OFFSET: usize = 8_192;

pub(super) fn insert(base: &[u8], inserted: &[u8]) -> Result<Vec<u8>, CorpusError> {
    let length = base
        .len()
        .checked_add(inserted.len())
        .ok_or(CorpusError::TotalLengthOverflow)?;
    let mut output = reserved(length, "early-insertion")?;
    output.extend_from_slice(range(base, ..EDIT_OFFSET, "early-insertion-prefix")?);
    output.extend_from_slice(inserted);
    output.extend_from_slice(range(base, EDIT_OFFSET.., "early-insertion-suffix")?);
    Ok(output)
}

pub(super) fn delete(base: &[u8]) -> Result<Vec<u8>, CorpusError> {
    let end = EDIT_OFFSET
        .checked_add(EDIT_LENGTH)
        .ok_or(CorpusError::TotalLengthOverflow)?;
    let length = base
        .len()
        .checked_sub(EDIT_LENGTH)
        .ok_or(CorpusError::InvalidGeneratedRange {
            target: "early-deletion",
        })?;
    let mut output = reserved(length, "early-deletion")?;
    output.extend_from_slice(range(base, ..EDIT_OFFSET, "early-deletion-prefix")?);
    output.extend_from_slice(range(base, end.., "early-deletion-suffix")?);
    Ok(output)
}

pub(super) fn substitute(base: &[u8]) -> Result<Vec<u8>, CorpusError> {
    let mut output = reserved(base.len(), "near-neighbor")?;
    output.extend_from_slice(base);
    let end = NEIGHBOR_OFFSET
        .checked_add(EDIT_LENGTH)
        .ok_or(CorpusError::TotalLengthOverflow)?;
    let changed =
        output
            .get_mut(NEIGHBOR_OFFSET..end)
            .ok_or(CorpusError::InvalidGeneratedRange {
                target: "near-neighbor",
            })?;
    for byte in changed {
        *byte ^= 0xa5;
    }
    Ok(output)
}

fn range<'a, R>(source: &'a [u8], range: R, target: &'static str) -> Result<&'a [u8], CorpusError>
where
    R: std::slice::SliceIndex<[u8], Output = [u8]>,
{
    source
        .get(range)
        .ok_or(CorpusError::InvalidGeneratedRange { target })
}
