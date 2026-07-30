//! This module owns stable retention-closure verification diagnostics.

use std::fmt;

use super::RetentionClosureVerificationError;

pub(super) fn display(
    error: &RetentionClosureVerificationError,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match error {
        RetentionClosureVerificationError::CounterOverflow {
            counter,
            current,
            incoming,
        } => write!(
            formatter,
            "{counter} overflowed while adding {incoming} to {current}"
        ),
        RetentionClosureVerificationError::LimitExceeded {
            counter,
            maximum,
            observed,
        } => write!(
            formatter,
            "{counter} limit {maximum} was exceeded by observed value {observed}"
        ),
        RetentionClosureVerificationError::LayoutEntryLimitHostWidth { observed } => write!(
            formatter,
            "closure node limit {observed} does not fit the layout entry-limit width"
        ),
        RetentionClosureVerificationError::LayoutEntryLimit { source } => {
            write!(
                formatter,
                "closure-derived layout entry limit is invalid: {source}"
            )
        }
        RetentionClosureVerificationError::MissingMember { identity } => match identity {
            crate::SegmentRecordIdentity::Chunk(chunk) => write!(
                formatter,
                "pinned catalog omits closure chunk length {} digest {}",
                chunk.length(),
                DigestHex(chunk.digest())
            ),
            crate::SegmentRecordIdentity::Layout(layout) => {
                write!(formatter, "pinned catalog omits closure layout {layout}")
            }
        },
        RetentionClosureVerificationError::LayoutDecode { layout, source } => {
            write!(
                formatter,
                "retained layout {layout} is not admissible: {source}"
            )
        }
        RetentionClosureVerificationError::AnchorTargetMismatch {
            layout,
            expected,
            observed,
        } => write!(
            formatter,
            "retained layout {layout} names blob {observed}, not anchor blob {expected}"
        ),
        RetentionClosureVerificationError::ProfileVerifierUnavailable { layout, profile } => {
            write!(
                formatter,
                "retained layout {layout} has no replay verifier for storage profile {profile}"
            )
        }
        RetentionClosureVerificationError::ProfileChunking { layout, source } => {
            write!(
                formatter,
                "storage-profile replay failed for retained layout {layout}: {source}"
            )
        }
        RetentionClosureVerificationError::ProfileBoundaryMismatch {
            layout,
            index,
            expected,
            observed,
        } => write!(
            formatter,
            "storage-profile boundary {index} for retained layout {layout} expected {} but observed {}",
            BoundaryDisplay(*expected),
            BoundaryDisplay(*observed)
        ),
        RetentionClosureVerificationError::BlobHash { layout, source } => {
            write!(
                formatter,
                "blob hashing failed for retained layout {layout}: {source}"
            )
        }
        RetentionClosureVerificationError::BlobIdentityMismatch {
            layout,
            expected,
            observed,
        } => write!(
            formatter,
            "retained layout {layout} reconstructs {observed}, not anchor blob {expected}"
        ),
    }
}

struct BoundaryDisplay(Option<crate::ProfileBoundary>);

impl fmt::Display for BoundaryDisplay {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(boundary) => write!(formatter, "{boundary}"),
            None => formatter.write_str("no boundary"),
        }
    }
}

struct DigestHex<'a>(&'a [u8; 32]);

impl fmt::Display for DigestHex<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}
