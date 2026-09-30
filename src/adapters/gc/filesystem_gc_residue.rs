//! This boundary module owns the exact read of `gc` and the segment pool
//! that restart classifies, and the names GC execution writes.

use std::io;

use cap_std::fs::Dir;

use super::{AdmittedGcRetirementIntent, GcResidue, GcRetirementIntent};
use crate::adapters::filesystem_exact_record::{self as exact_record, ExactRecordError};
use crate::adapters::physical_pool_name;

/// `gc/intent.next`.
pub(super) const INTENT_STAGE: &str = "intent.next";
/// `gc/intent`.
pub(super) const INTENT: &str = "intent";
/// `gc/receipt.next`.
pub(super) const RECEIPT_STAGE: &str = "receipt.next";
/// `gc/receipt`.
pub(super) const RECEIPT: &str = "receipt";
/// Every entry `gc` may hold.
pub(in crate::adapters) const GC_ENTRY_NAMES: [&str; 4] =
    [INTENT_STAGE, INTENT, RECEIPT_STAGE, RECEIPT];

const RECEIPT_LENGTH: usize = 320;

/// Reads every `gc` record as it is, bounded one byte past its maximum
/// length, and the presence of every candidate the durable intent names.
pub(super) fn read(gc: &Dir, segments: &Dir) -> io::Result<GcResidue> {
    let intent_bound =
        super::intent_format::canonical_length(GcRetirementIntent::MAXIMUM_CANDIDATE_COUNT)
            .and_then(|length| length.checked_add(1))
            .ok_or_else(|| invalid("GC intent bound overflow"))?;
    let receipt_bound = RECEIPT_LENGTH.saturating_add(1);
    let intent = read_bounded(gc, INTENT, intent_bound)?;
    let candidates_present = match intent.as_deref().map(AdmittedGcRetirementIntent::decode) {
        Some(Ok(admitted)) => candidates_present(segments, admitted.intent())?,
        Some(Err(_)) | None => Vec::new(),
    };
    Ok(GcResidue {
        intent_stage: read_bounded(gc, INTENT_STAGE, intent_bound)?,
        intent,
        receipt_stage: read_bounded(gc, RECEIPT_STAGE, receipt_bound)?,
        receipt: read_bounded(gc, RECEIPT, receipt_bound)?,
        candidates_present,
    })
}

/// Whether each candidate's pool entry exists, by name, without following
/// links; execution verifies bytes, restart only counts presence.
fn candidates_present(segments: &Dir, intent: &GcRetirementIntent) -> io::Result<Vec<bool>> {
    let mut present = Vec::new();
    for candidate in intent.candidates() {
        let name = physical_pool_name::segment(candidate.segment_digest());
        match segments.symlink_metadata(&name) {
            Ok(_metadata) => present.push(true),
            Err(source) if source.kind() == io::ErrorKind::NotFound => present.push(false),
            Err(source) => return Err(source),
        }
    }
    Ok(present)
}

fn read_bounded(gc: &Dir, name: &str, bound: usize) -> io::Result<Option<Box<[u8]>>> {
    match exact_record::read_bounded_optional(gc, name, bound) {
        Ok(bytes) => Ok(bytes.map(Vec::into_boxed_slice)),
        Err(ExactRecordError::Io(source)) => Err(source),
        Err(ExactRecordError::Refused(refusal)) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            refusal.to_string(),
        )),
    }
}

/// Removes a discardable stage and proves it gone.
pub(super) fn discard(gc: &Dir, name: &str) -> io::Result<()> {
    gc.remove_file(name)?;
    exact_record::require_absent(gc, name)
        .map_err(|_source| invalid("discarded GC stage remained visible"))?;
    crate::adapters::filesystem_catalog_artifact::synchronize_directory(gc)
}

pub(super) fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

pub(super) fn invalid_from(error: impl std::error::Error + Send + Sync + 'static) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}
