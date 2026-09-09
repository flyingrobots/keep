//! This module owns crash injection around production retention publication.

use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use keep::{
    AdmittedRetentionRoot, CanonicalRetentionHead, CanonicalRetentionManifest,
    FilesystemRetentionPublicationAuthority, RetentionNamespaceAdmission,
    RetentionPublicationPreparation, RetentionPublicationStorage, RetentionTransitionDisposition,
};
use xtask::{DurabilityCrashPoint, DurabilityCrashPosition};

use super::control::{CrashControl, DuringTiming};

/// Bytes an interrupted stage write leaves behind: inside every record's
/// fixed framing, so restart classifies the stage as truncated.
const STAGE_INTERRUPTION: usize = 100;

pub(super) struct CrashRetentionStorage<'control> {
    inner: FilesystemRetentionPublicationAuthority,
    control: &'control mut CrashControl,
    retention: PathBuf,
}

impl<'control> CrashRetentionStorage<'control> {
    pub(super) fn new(
        inner: FilesystemRetentionPublicationAuthority,
        control: &'control mut CrashControl,
        store_root: &Path,
    ) -> Self {
        Self {
            inner,
            control,
            retention: store_root.join("retention"),
        }
    }

    fn execute<T>(
        &mut self,
        point: DurabilityCrashPoint,
        during: DuringTiming,
        operation: impl FnOnce(&mut FilesystemRetentionPublicationAuthority) -> io::Result<T>,
    ) -> io::Result<T> {
        self.control.before(point, during)?;
        let result = operation(&mut self.inner)?;
        self.control.after(point, during)?;
        Ok(result)
    }

    fn execute_write(
        &mut self,
        point: DurabilityCrashPoint,
        stage: &str,
        bytes: &[u8],
        complete: impl FnOnce(&mut FilesystemRetentionPublicationAuthority) -> io::Result<()>,
    ) -> io::Result<()> {
        match self.control.position(point) {
            None => complete(&mut self.inner),
            Some(DurabilityCrashPosition::Before) => self.control.await_process_death(),
            Some(DurabilityCrashPosition::During) => {
                let partial = bytes.get(..STAGE_INTERRUPTION).ok_or_else(|| {
                    io::Error::other("retention record shorter than the interruption prefix")
                })?;
                let mut file = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(self.retention.join(stage))?;
                file.write_all(partial)?;
                self.control.await_process_death()
            }
            Some(DurabilityCrashPosition::After) => {
                complete(&mut self.inner)?;
                self.control.await_process_death()
            }
        }
    }
}

impl RetentionPublicationStorage for CrashRetentionStorage<'_> {
    fn verify_current(
        &mut self,
        preparation: &RetentionPublicationPreparation<'_>,
    ) -> io::Result<RetentionTransitionDisposition> {
        self.inner.verify_current(preparation)
    }

    fn write_root_stage(&mut self, root: &AdmittedRetentionRoot<'_>) -> io::Result<()> {
        self.execute_write(
            DurabilityCrashPoint::WriteRootStage,
            "root.next",
            root.encoded(),
            |inner| inner.write_root_stage(root),
        )
    }

    fn synchronize_root_stage(&mut self) -> io::Result<()> {
        self.execute(
            DurabilityCrashPoint::SynchronizeRootStage,
            DuringTiming::Before,
            FilesystemRetentionPublicationAuthority::synchronize_root_stage,
        )
    }

    fn admit_root_namespace(
        &mut self,
        root: &AdmittedRetentionRoot<'_>,
    ) -> io::Result<RetentionNamespaceAdmission> {
        self.execute(
            DurabilityCrashPoint::AdmitRootNamespace,
            DuringTiming::After,
            |inner| inner.admit_root_namespace(root),
        )
    }

    fn synchronize_roots_after_namespace(&mut self) -> io::Result<()> {
        self.execute(
            DurabilityCrashPoint::SynchronizeRootsAfterNamespace,
            DuringTiming::Before,
            FilesystemRetentionPublicationAuthority::synchronize_roots_after_namespace,
        )
    }

    fn link_root(&mut self, root: &AdmittedRetentionRoot<'_>) -> io::Result<()> {
        self.execute(
            DurabilityCrashPoint::LinkRoot,
            DuringTiming::After,
            |inner| inner.link_root(root),
        )
    }

    fn synchronize_root_namespace(&mut self, root: &AdmittedRetentionRoot<'_>) -> io::Result<()> {
        self.execute(
            DurabilityCrashPoint::SynchronizeRootNamespace,
            DuringTiming::Before,
            |inner| inner.synchronize_root_namespace(root),
        )
    }

    fn write_manifest_stage(&mut self, manifest: &CanonicalRetentionManifest) -> io::Result<()> {
        self.execute_write(
            DurabilityCrashPoint::WriteManifestStage,
            "manifest.next",
            manifest.encoded(),
            |inner| inner.write_manifest_stage(manifest),
        )
    }

    fn synchronize_manifest_stage(&mut self) -> io::Result<()> {
        self.execute(
            DurabilityCrashPoint::SynchronizeManifestStage,
            DuringTiming::Before,
            FilesystemRetentionPublicationAuthority::synchronize_manifest_stage,
        )
    }

    fn link_manifest(&mut self, manifest: &CanonicalRetentionManifest) -> io::Result<()> {
        self.execute(
            DurabilityCrashPoint::LinkManifest,
            DuringTiming::After,
            |inner| inner.link_manifest(manifest),
        )
    }

    fn synchronize_manifest_pool(&mut self) -> io::Result<()> {
        self.execute(
            DurabilityCrashPoint::SynchronizeManifestPool,
            DuringTiming::Before,
            FilesystemRetentionPublicationAuthority::synchronize_manifest_pool,
        )
    }

    fn write_head_stage(&mut self, head: &CanonicalRetentionHead) -> io::Result<()> {
        self.execute_write(
            DurabilityCrashPoint::WriteHeadStage,
            "head.next",
            head.encoded(),
            |inner| inner.write_head_stage(head),
        )
    }

    fn synchronize_head_stage(&mut self) -> io::Result<()> {
        self.execute(
            DurabilityCrashPoint::SynchronizeHeadStage,
            DuringTiming::Before,
            FilesystemRetentionPublicationAuthority::synchronize_head_stage,
        )
    }

    fn replace_head(&mut self) -> io::Result<()> {
        self.execute(
            DurabilityCrashPoint::ReplaceRetentionHead,
            DuringTiming::After,
            FilesystemRetentionPublicationAuthority::replace_head,
        )
    }

    fn synchronize_retention_namespace(&mut self) -> io::Result<()> {
        self.execute(
            DurabilityCrashPoint::SynchronizeRetentionNamespace,
            DuringTiming::Before,
            FilesystemRetentionPublicationAuthority::synchronize_retention_namespace,
        )
    }

    fn remove_root_stage(&mut self) -> io::Result<()> {
        self.execute(
            DurabilityCrashPoint::RemoveRootStage,
            DuringTiming::After,
            FilesystemRetentionPublicationAuthority::remove_root_stage,
        )
    }

    fn remove_manifest_stage(&mut self) -> io::Result<()> {
        self.execute(
            DurabilityCrashPoint::RemoveManifestStage,
            DuringTiming::After,
            FilesystemRetentionPublicationAuthority::remove_manifest_stage,
        )
    }

    fn synchronize_cleanup(&mut self) -> io::Result<()> {
        self.execute(
            DurabilityCrashPoint::SynchronizeRetentionCleanup,
            DuringTiming::Before,
            FilesystemRetentionPublicationAuthority::synchronize_cleanup,
        )
    }
}
