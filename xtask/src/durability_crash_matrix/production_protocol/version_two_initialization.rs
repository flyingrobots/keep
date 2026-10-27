//! This module owns execution of the production version-two initializer.
//!
//! An observer installed on the initializer's own phases blocks at the
//! selected boundary for process death; no phase is reimplemented here. The
//! recovery row first stops one initialization after its writer lock, with an
//! incomplete `FORMAT.next` in place, then dies inside the resuming call.

use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use keep::{
    CanonicalStoreFormatMarker, FilesystemVersionTwoAdmission, VersionTwoInitializationBoundary,
    VersionTwoInitializationDirectory, VersionTwoInitializationObserver,
    VersionTwoInitializationPhase,
};
use xtask::{DurabilityCrashOccurrence, DurabilityCrashPoint, DurabilityCrashPosition};

use super::control::{CrashControl, DuringTiming};
use super::{DurabilityCrashMatrixError, verification};

/// Length of the incomplete marker stage a torn write leaves; a nonempty
/// strict prefix of the 96-byte marker.
pub(in crate::durability_crash_matrix) const INCOMPLETE_STAGE_LENGTH: usize = 48;

/// Initializes the store root, dying at the selected coordinate.
pub(super) fn run(
    store_root: &Path,
    control: CrashControl,
) -> Result<(), DurabilityCrashMatrixError> {
    if control
        .position(DurabilityCrashPoint::InitializeDiscardIncompleteMarkerStage)
        .is_some()
    {
        leave_incomplete_stage(store_root)?;
    }
    let observer = observer(control, store_root.to_path_buf());
    FilesystemVersionTwoAdmission::initialize_observed_for_repository_tasks(store_root, observer)
        .map(drop)
        .map_err(|source| verification("execute production initialization", source))
}

/// Stops a real initialization at its marker write with a torn stage in place.
fn leave_incomplete_stage(store_root: &Path) -> Result<(), DurabilityCrashMatrixError> {
    let root = store_root.to_path_buf();
    let observer: VersionTwoInitializationObserver = Box::new(move |phase, boundary| {
        if phase == VersionTwoInitializationPhase::WriteMarkerStage
            && boundary == VersionTwoInitializationBoundary::Before
        {
            write_incomplete_stage(&root)?;
            return Err(io::Error::other("stopped with an incomplete marker stage"));
        }
        Ok(())
    });
    match FilesystemVersionTwoAdmission::initialize_observed_for_repository_tasks(
        store_root, observer,
    ) {
        Err(_) => Ok(()),
        Ok(_) => Err(verification(
            "prepare an incomplete marker stage",
            io::Error::other("initialization completed past the marker write"),
        )),
    }
}

/// Writes the strict canonical prefix a torn write could leave, unsynchronized.
fn write_incomplete_stage(root: &Path) -> io::Result<()> {
    let marker = CanonicalStoreFormatMarker::version_two();
    let prefix = marker
        .encoded()
        .get(..INCOMPLETE_STAGE_LENGTH)
        .ok_or_else(|| io::Error::other("marker shorter than the incomplete stage"))?;
    let mut stage = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join("FORMAT.next"))?;
    stage.write_all(prefix)?;
    stage.flush()
}

/// Maps the initializer's own phases onto the eleven initialization crash points.
fn observer(mut control: CrashControl, root: PathBuf) -> VersionTwoInitializationObserver {
    Box::new(move |phase, boundary| {
        if phase == VersionTwoInitializationPhase::WriteMarkerStage
            && boundary == VersionTwoInitializationBoundary::Before
            && control.position(DurabilityCrashPoint::InitializeWriteMarkerStage)
                == Some(DurabilityCrashPosition::During)
        {
            write_incomplete_stage(&root)?;
            return control.await_process_death();
        }
        if let VersionTwoInitializationPhase::AdmitDirectory(directory) = phase {
            return directory_boundary(&mut control, directory, boundary);
        }
        let Some((point, timing)) = crash_point(phase) else {
            return Ok(());
        };
        match boundary {
            VersionTwoInitializationBoundary::Before => control.before(point, timing),
            VersionTwoInitializationBoundary::After => control.after(point, timing),
        }
    })
}

/// Directory admission is one occurrence-counted point: `before` precedes the
/// first directory, `after` follows the last, and `during` occurrence *k*
/// follows directory *k* + 1.
fn directory_boundary(
    control: &mut CrashControl,
    directory: VersionTwoInitializationDirectory,
    boundary: VersionTwoInitializationBoundary,
) -> io::Result<()> {
    let point = DurabilityCrashPoint::InitializeAdmitDirectory;
    let ordinal = VersionTwoInitializationDirectory::ALL
        .iter()
        .position(|candidate| *candidate == directory)
        .ok_or_else(|| io::Error::other("directory outside the creation order"))?;
    let last = VersionTwoInitializationDirectory::ALL
        .len()
        .saturating_sub(1);
    let selected = match (control.position(point), boundary) {
        (Some(DurabilityCrashPosition::Before), VersionTwoInitializationBoundary::Before) => {
            ordinal == 0
        }
        (Some(DurabilityCrashPosition::After), VersionTwoInitializationBoundary::After) => {
            ordinal == last
        }
        (Some(DurabilityCrashPosition::During), VersionTwoInitializationBoundary::After) => {
            let occurrence = control
                .occurrence()
                .map_or(0, DurabilityCrashOccurrence::get);
            usize::try_from(occurrence).map_err(io::Error::other)? == ordinal
        }
        _ => false,
    };
    if selected {
        control.await_process_death()
    } else {
        Ok(())
    }
}

const fn crash_point(
    phase: VersionTwoInitializationPhase,
) -> Option<(DurabilityCrashPoint, DuringTiming)> {
    use DurabilityCrashPoint as Point;
    use VersionTwoInitializationPhase as Phase;
    Some(match phase {
        Phase::OpenAndLockWriterFile => {
            (Point::InitializeOpenAndLockWriterFile, DuringTiming::After)
        }
        Phase::WriteMarkerStage => (Point::InitializeWriteMarkerStage, DuringTiming::After),
        Phase::SynchronizeMarkerStage => (
            Point::InitializeSynchronizeMarkerStage,
            DuringTiming::Before,
        ),
        Phase::SynchronizeRootAfterMarkerStage => (
            Point::InitializeSynchronizeRootAfterMarkerStage,
            DuringTiming::Before,
        ),
        Phase::AdmitReaderFence => (Point::InitializeAdmitReaderFence, DuringTiming::After),
        Phase::LinkMarker => (Point::InitializeLinkMarker, DuringTiming::After),
        Phase::SynchronizeRootAfterMarker => (
            Point::InitializeSynchronizeRootAfterMarker,
            DuringTiming::Before,
        ),
        Phase::RemoveMarkerStage => (Point::InitializeRemoveMarkerStage, DuringTiming::After),
        Phase::SynchronizeRootAfterCleanup => (
            Point::InitializeSynchronizeRootAfterCleanup,
            DuringTiming::Before,
        ),
        Phase::DiscardIncompleteMarkerStage => (
            Point::InitializeDiscardIncompleteMarkerStage,
            DuringTiming::After,
        ),
        _ => return None,
    })
}
