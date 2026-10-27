//! This module owns admission of the committed `FORMAT` marker record.

use std::io;

use cap_std::fs::Dir;

use super::StoreRecordRefusal as Refusal;
use super::filesystem_exact_record::{self as exact_record, ExactRecordError, ExactRecordRefusal};
use super::{AdmittedStoreFormatMarker, FORMAT_MARKER_LENGTH};

const MARKER_NAME: &str = "FORMAT";

/// Reads and admits `FORMAT` as the one registered format marker.
///
/// The record is reopened without following links, bounded to its exact
/// canonical length, and decoded. Writer authority over a store root
/// must not be returned before this admission succeeds. Every refusal is an
/// `InvalidData` error whose source is a
/// [`StoreRecordRefusal`](super::StoreRecordRefusal). Nothing about
/// the root directory's own identity is compared: the marker binds the format
/// contract, not a location.
pub(super) fn admit(root: &Dir) -> io::Result<()> {
    let bytes = exact_record::read_exact_regular(root, MARKER_NAME, FORMAT_MARKER_LENGTH)
        .map_err(refusal)?;
    AdmittedStoreFormatMarker::decode(&bytes)
        .map(|_marker| ())
        .map_err(|source| Refusal::Marker { source }.into_io())
}

/// Maps a shared exact-record failure onto the marker's refusal.
fn refusal(error: ExactRecordError) -> io::Error {
    match error {
        ExactRecordError::Io(source) => source,
        ExactRecordError::Refused(refusal) => match refusal {
            ExactRecordRefusal::LengthOverflow => Refusal::LengthOverflow { name: MARKER_NAME },
            ExactRecordRefusal::TrailingBytes => Refusal::TrailingBytes { name: MARKER_NAME },
            ExactRecordRefusal::KindOrLength
            | ExactRecordRefusal::KindLengthOrIdentity
            | ExactRecordRefusal::Bytes
            | ExactRecordRefusal::RemainedVisible => Refusal::KindOrLength { name: MARKER_NAME },
        }
        .into_io(),
    }
}
