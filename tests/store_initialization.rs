//! Deterministic segment-store initialization laws.

use std::error::Error;
use std::io;

use keep::{
    StoreInitializationError, StoreInitializationPhase, StoreInitializationStorage,
    initialize_store,
};

const PHASES: [StoreInitializationPhase; 6] = [
    StoreInitializationPhase::AdmitPlatform,
    StoreInitializationPhase::OpenAndLockWriterFile,
    StoreInitializationPhase::AdmitStagingDirectory,
    StoreInitializationPhase::AdmitSegmentPoolDirectory,
    StoreInitializationPhase::AdmitCatalogPoolDirectory,
    StoreInitializationPhase::SynchronizeRoot,
];

#[test]
fn initialization_admits_platform_before_every_namespace_transition() -> Result<(), Box<dyn Error>>
{
    let mut storage = RecordingStorage::new(None);

    let _receipt = initialize_store(&mut storage)?;

    assert_eq!(storage.attempted, PHASES);
    Ok(())
}

#[test]
fn initialization_stops_at_and_preserves_every_exact_failure_phase() -> Result<(), Box<dyn Error>> {
    let mut expected_prefix = Vec::new();
    for phase in PHASES {
        expected_prefix.push(phase);
        let mut storage = RecordingStorage::new(Some(phase));
        let Err(error) = initialize_store(&mut storage) else {
            return Err(format!("initialization ignored injected failure at {phase}").into());
        };

        match error {
            StoreInitializationError::Io {
                phase: observed,
                source,
            } => {
                assert_eq!(observed, phase);
                assert_eq!(source.kind(), io::ErrorKind::Other);
            }
        }
        assert_eq!(storage.attempted, expected_prefix);
    }
    Ok(())
}

struct RecordingStorage {
    attempted: Vec<StoreInitializationPhase>,
    fail_at: Option<StoreInitializationPhase>,
}

impl RecordingStorage {
    const fn new(fail_at: Option<StoreInitializationPhase>) -> Self {
        Self {
            attempted: Vec::new(),
            fail_at,
        }
    }

    fn attempt(&mut self, phase: StoreInitializationPhase) -> io::Result<()> {
        self.attempted.push(phase);
        if self.fail_at == Some(phase) {
            Err(io::Error::other("injected initialization failure"))
        } else {
            Ok(())
        }
    }
}

impl StoreInitializationStorage for RecordingStorage {
    fn admit_platform(&mut self) -> io::Result<()> {
        self.attempt(StoreInitializationPhase::AdmitPlatform)
    }

    fn open_and_lock_writer_file(&mut self) -> io::Result<()> {
        self.attempt(StoreInitializationPhase::OpenAndLockWriterFile)
    }

    fn admit_staging_directory(&mut self) -> io::Result<()> {
        self.attempt(StoreInitializationPhase::AdmitStagingDirectory)
    }

    fn admit_segment_pool_directory(&mut self) -> io::Result<()> {
        self.attempt(StoreInitializationPhase::AdmitSegmentPoolDirectory)
    }

    fn admit_catalog_pool_directory(&mut self) -> io::Result<()> {
        self.attempt(StoreInitializationPhase::AdmitCatalogPoolDirectory)
    }

    fn synchronize_root(&mut self) -> io::Result<()> {
        self.attempt(StoreInitializationPhase::SynchronizeRoot)
    }
}
