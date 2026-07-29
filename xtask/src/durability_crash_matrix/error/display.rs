//! This module owns human-readable crash-matrix failure rendering.

use std::fmt;

use super::DurabilityCrashMatrixError;

impl fmt::Debug for DurabilityCrashMatrixError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for DurabilityCrashMatrixError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ArtifactBytesMismatch { .. }
            | Self::ArtifactClassificationMismatch { .. }
            | Self::HardLinkIdentityMismatch { .. }
            | Self::InventoryMismatch { .. }
            | Self::MissingVisibleRecord { .. }
            | Self::RepeatedInventoryPath { .. }
            | Self::SnapshotGenerationMismatch { .. }
            | Self::UnexpectedArtifactKind { .. } => format_state(self, formatter),
            Self::ChildExitedEarly { .. }
            | Self::ChildSurvivedTermination { .. }
            | Self::InvalidReadinessSignal { .. }
            | Self::Timeout { .. } => format_process(self, formatter),
            Self::Fixture { .. }
            | Self::FixtureLength { .. }
            | Self::FixtureRange
            | Self::FixtureTerminator { .. } => format_fixture(self, formatter),
            Self::Case { .. }
            | Self::InvalidCase(_)
            | Self::InvalidPointEncoding
            | Self::InvalidPositionEncoding
            | Self::UnknownPoint(_)
            | Self::UnknownPosition(_)
            | Self::Usage => format_command(self, formatter),
            Self::Io { .. }
            | Self::NonUnicodeStatePath
            | Self::PointSequenceMismatch { .. }
            | Self::Verification { .. } => format_boundary(self, formatter),
        }
    }
}

fn format_state(
    error: &DurabilityCrashMatrixError,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match error {
        DurabilityCrashMatrixError::ArtifactBytesMismatch {
            artifact,
            expected_length,
            observed_length,
            offset,
            expected,
            observed,
        } => write!(
            formatter,
            "post-crash artifact `{artifact}` differs at byte {offset}: \
             expected {expected:?} within {expected_length} bytes, \
             observed {observed:?} within {observed_length} bytes"
        ),
        DurabilityCrashMatrixError::ArtifactClassificationMismatch { expected, observed } => {
            write!(
                formatter,
                "post-crash artifact classification mismatch: \
                 expected `{expected}`, observed `{observed}`"
            )
        }
        DurabilityCrashMatrixError::HardLinkIdentityMismatch { .. } => {
            format_hard_link(error, formatter)
        }
        DurabilityCrashMatrixError::InventoryMismatch { expected, observed } => write!(
            formatter,
            "post-crash path inventory mismatch: expected {expected:?}, observed {observed:?}"
        ),
        DurabilityCrashMatrixError::MissingVisibleRecord { record } => {
            write!(
                formatter,
                "post-crash snapshot lacks visible record `{record}`"
            )
        }
        DurabilityCrashMatrixError::RepeatedInventoryPath { path } => {
            write!(formatter, "post-crash inventory repeated path `{path}`")
        }
        DurabilityCrashMatrixError::SnapshotGenerationMismatch { expected, observed } => write!(
            formatter,
            "post-crash snapshot generation is {observed}, expected {expected}"
        ),
        DurabilityCrashMatrixError::UnexpectedArtifactKind {
            artifact,
            expected,
            observed,
        } => write!(
            formatter,
            "post-crash model assigned `{artifact}` kind `{observed}`, expected `{expected}`"
        ),
        _ => Err(fmt::Error),
    }
}

fn format_hard_link(
    error: &DurabilityCrashMatrixError,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    let DurabilityCrashMatrixError::HardLinkIdentityMismatch {
        source,
        target,
        source_device,
        source_inode,
        target_device,
        target_inode,
    } = error
    else {
        return Err(fmt::Error);
    };
    write!(
        formatter,
        "post-crash hard-link identity mismatch between `{source}` \
         ({source_device}:{source_inode}) and `{target}` \
         ({target_device}:{target_inode})"
    )
}

fn format_process(
    error: &DurabilityCrashMatrixError,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match error {
        DurabilityCrashMatrixError::ChildExitedEarly { code } => write!(
            formatter,
            "crash child exited before readiness with code {code:?}"
        ),
        DurabilityCrashMatrixError::ChildSurvivedTermination { code } => write!(
            formatter,
            "crash child survived termination with code {code:?}"
        ),
        DurabilityCrashMatrixError::InvalidReadinessSignal { observed } => {
            write!(formatter, "crash child sent readiness byte {observed}")
        }
        DurabilityCrashMatrixError::Timeout { duration } => {
            write!(formatter, "crash child exceeded its {duration:?} deadline")
        }
        _ => Err(fmt::Error),
    }
}

fn format_fixture(
    error: &DurabilityCrashMatrixError,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match error {
        DurabilityCrashMatrixError::Fixture { artifact, .. } => {
            write!(formatter, "cannot decode {artifact} crash fixture")
        }
        DurabilityCrashMatrixError::FixtureLength {
            artifact,
            expected,
            observed,
        } => write!(
            formatter,
            "{artifact} crash fixture has length {observed}, expected {expected}"
        ),
        DurabilityCrashMatrixError::FixtureRange => {
            formatter.write_str("crash fixture range is invalid")
        }
        DurabilityCrashMatrixError::FixtureTerminator { artifact } => write!(
            formatter,
            "{artifact} crash fixture lacks its final line feed"
        ),
        _ => Err(fmt::Error),
    }
}

fn format_command(
    error: &DurabilityCrashMatrixError,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match error {
        DurabilityCrashMatrixError::Case {
            point,
            position,
            source,
        } => write!(
            formatter,
            "{} {}: {source}",
            point.identifier(),
            position.identifier()
        ),
        DurabilityCrashMatrixError::InvalidCase(error) => {
            write!(formatter, "invalid crash case: {error}")
        }
        DurabilityCrashMatrixError::InvalidPointEncoding => {
            formatter.write_str("crash point is not valid Unicode")
        }
        DurabilityCrashMatrixError::InvalidPositionEncoding => {
            formatter.write_str("crash position is not valid Unicode")
        }
        DurabilityCrashMatrixError::UnknownPoint(point) => {
            write!(formatter, "unknown crash point `{point}`")
        }
        DurabilityCrashMatrixError::UnknownPosition(position) => {
            write!(formatter, "unknown crash position `{position}`")
        }
        DurabilityCrashMatrixError::Usage => formatter.write_str(
            "usage: cargo xtask durability-crash-matrix \
             --case <KEEP-CRASH-NNN> <before|during|after>",
        ),
        _ => Err(fmt::Error),
    }
}

fn format_boundary(
    error: &DurabilityCrashMatrixError,
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match error {
        DurabilityCrashMatrixError::Io { action, .. } => write!(formatter, "cannot {action}"),
        DurabilityCrashMatrixError::NonUnicodeStatePath => {
            formatter.write_str("post-crash store path is not valid Unicode")
        }
        DurabilityCrashMatrixError::PointSequenceMismatch { point } => write!(
            formatter,
            "{} is outside the selected crash sequence",
            point.identifier()
        ),
        DurabilityCrashMatrixError::Verification { phase, source } => write!(
            formatter,
            "post-crash verification failed while attempting to {phase}: {source}"
        ),
        _ => Err(fmt::Error),
    }
}
