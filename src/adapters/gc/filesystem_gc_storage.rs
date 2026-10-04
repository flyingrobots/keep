//! This module owns the filesystem effects of every GC execution phase.

use std::io;

use super::filesystem_gc_authority::GcExecutionContext;
use super::filesystem_gc_residue::{INTENT, INTENT_STAGE, RECEIPT, RECEIPT_STAGE, invalid};
use super::{
    AdmittedGcRetirementIntent, AdmittedGcRetirementReceipt, CanonicalGcRetirementReceipt,
    FilesystemGcAuthority, PoolStateDigest, segment_pool_identity,
    segment_pool_inventory,
};
use crate::adapters::filesystem_catalog_artifact::synchronize_directory;
use crate::adapters::filesystem_exact_record::{self as exact_record, ExactRecordError};
use crate::adapters::physical_pool_name;
use crate::adapters::retention::{FilesystemRetentionStage, RetentionStorageBoundary as Boundary};

fn no_retirement() -> io::Error {
    invalid(super::FilesystemGcRefusal::NoRetirement)
}

impl FilesystemGcAuthority {
    fn context(&mut self) -> io::Result<&mut GcExecutionContext> {
        self.context.as_mut().ok_or_else(no_retirement)
    }

    fn intent_stage(&self) -> io::Result<&FilesystemRetentionStage> {
        self.context
            .as_ref()
            .and_then(|context| context.intent_stage.as_ref())
            .ok_or_else(no_retirement)
    }

    fn receipt_stage(&self) -> io::Result<&FilesystemRetentionStage> {
        self.context
            .as_ref()
            .and_then(|context| context.receipt_stage.as_ref())
            .ok_or_else(no_retirement)
    }

    /// The receipt this retirement completes with: every candidate proven
    /// absent, then the pool identity over the exact remaining inventory.
    fn completing_receipt(&self) -> io::Result<CanonicalGcRetirementReceipt> {
        let context = self.context.as_ref().ok_or_else(no_retirement)?;
        for candidate in context.intent.intent().candidates() {
            let name = physical_pool_name::segment(candidate.segment_digest());
            match self.segments.symlink_metadata(&name) {
                Err(source) if source.kind() == io::ErrorKind::NotFound => {}
                Err(source) => return Err(source),
                Ok(_present) => {
                    return Err(invalid(super::FilesystemGcRefusal::CandidateStillPresent));
                }
            }
        }
        let inventory = segment_pool_inventory::read(
            &self.segments,
            self.policy.segment_read(),
            self.policy.retained_segment_bytes().get(),
        )
        .map_err(|source| io::Error::new(io::ErrorKind::InvalidData, source))?
        .into_iter()
        .collect();
        let pool_state = PoolStateDigest::new(segment_pool_identity(&inventory));
        Ok(CanonicalGcRetirementReceipt::from_intent(
            &context.intent,
            pool_state,
        ))
    }

    fn reopen_receipt_stage(&mut self) -> io::Result<()> {
        if self.context()?.receipt.is_none() {
            let receipt = self.completing_receipt()?;
            self.context()?.receipt = Some(receipt);
        }
        let gc = self.gc.try_clone()?;
        let context = self.context()?;
        if context.receipt_stage.is_none() {
            let receipt = context.receipt.as_ref().ok_or_else(no_retirement)?;
            context.receipt_stage = Some(FilesystemRetentionStage::reopen(
                &gc,
                RECEIPT_STAGE,
                receipt.encoded(),
                context
                    .receipt_observation
                    .as_ref()
                    .ok_or_else(no_retirement)?
                    .identity(),
            )?);
        }
        Ok(())
    }

    fn reopen_intent_stage(&mut self) -> io::Result<()> {
        let gc = self.gc.try_clone()?;
        let context = self.context()?;
        if context.intent_stage.is_none() {
            context.intent_stage = Some(FilesystemRetentionStage::reopen(
                &gc,
                INTENT_STAGE,
                context.intent.encoded(),
                context
                    .intent_observation
                    .as_ref()
                    .ok_or_else(no_retirement)?
                    .identity(),
            )?);
        }
        Ok(())
    }
}

impl FilesystemGcAuthority {
    pub(super) fn store_write_intent_stage(&mut self) -> io::Result<()> {
        let gc = self.gc.try_clone()?;
        let context = self.context()?;
        context.intent_stage = Some(FilesystemRetentionStage::create(
            &gc,
            INTENT_STAGE,
            context.intent.encoded(),
        )?);
        Ok(())
    }

    pub(super) fn store_synchronize_intent_stage(&mut self) -> io::Result<()> {
        self.reopen_intent_stage()?;
        self.intent_stage()?
            .synchronize(&self.gc)
            .map_err(Into::into)
    }

    pub(super) fn store_link_intent(&mut self) -> io::Result<()> {
        self.reopen_intent_stage()?;
        self.intent_stage()?
            .link(&self.gc, &self.gc, INTENT)
            .map(|_outcome| ())
            .map_err(Into::into)
    }

    pub(super) fn store_synchronize_gc_after_intent(&mut self) -> io::Result<()> {
        self.storage_checkpoint(Boundary::GcIntentSynchronization)?;
        synchronize_directory(&self.gc)
    }

    pub(super) fn store_remove_intent_stage(&mut self) -> io::Result<()> {
        self.reopen_intent_stage()?;
        let stage = self
            .context()?
            .intent_stage
            .take()
            .ok_or_else(no_retirement)?;
        stage.remove(&self.gc, &self.gc, INTENT).map_err(Into::into)
    }

    pub(super) fn store_synchronize_gc_after_intent_cleanup(&mut self) -> io::Result<()> {
        self.storage_checkpoint(Boundary::GcIntentCleanupSynchronization)?;
        synchronize_directory(&self.gc)
    }

    pub(super) fn store_unlink_candidate(&mut self, index: usize) -> io::Result<()> {
        let context = self.context.as_ref().ok_or_else(no_retirement)?;
        let candidate = context
            .intent
            .intent()
            .candidates()
            .get(index)
            .copied()
            .ok_or_else(|| invalid(super::FilesystemGcRefusal::CandidateIndex))?;
        let name = physical_pool_name::segment(candidate.segment_digest());
        let metadata = self.segments.symlink_metadata(&name)?;
        if !metadata.is_file() || metadata.len() != candidate.segment_length() {
            return Err(invalid(super::FilesystemGcRefusal::CandidateKindOrLength));
        }
        segment_pool_inventory::admit_entry(
            &self.segments,
            &name,
            candidate.segment_digest(),
            metadata.len(),
            self.policy.segment_read(),
        )
        .map_err(|source| io::Error::new(io::ErrorKind::InvalidData, source))?;
        self.storage_checkpoint(Boundary::CandidateUnlink)?;
        self.segments.remove_file(&name)?;
        self.storage_checkpoint(Boundary::CandidateAbsence)?;
        exact_record::require_absent(&self.segments, &name).map_err(ExactRecordError::into_io)
    }

    pub(super) fn store_synchronize_segment_pool(&mut self, _index: usize) -> io::Result<()> {
        self.storage_checkpoint(Boundary::PoolSynchronization)?;
        synchronize_directory(&self.segments)
    }

    pub(super) fn store_write_receipt_stage(&mut self) -> io::Result<()> {
        let receipt = self.completing_receipt()?;
        let gc = self.gc.try_clone()?;
        let context = self.context()?;
        context.receipt_stage = Some(FilesystemRetentionStage::create(
            &gc,
            RECEIPT_STAGE,
            receipt.encoded(),
        )?);
        context.receipt = Some(receipt);
        Ok(())
    }

    pub(super) fn store_synchronize_receipt_stage(&mut self) -> io::Result<()> {
        self.reopen_receipt_stage()?;
        self.receipt_stage()?
            .synchronize(&self.gc)
            .map_err(Into::into)
    }

    pub(super) fn store_replace_receipt(&mut self) -> io::Result<()> {
        self.reopen_receipt_stage()?;
        let stage = self
            .context()?
            .receipt_stage
            .take()
            .ok_or_else(no_retirement)?;
        stage
            .replace(
                &self.gc,
                RECEIPT,
                crate::adapters::retention::StageReplacement::Receipt,
            )
            .map_err(Into::into)
    }

    pub(super) fn store_synchronize_gc_after_receipt(&mut self) -> io::Result<()> {
        self.storage_checkpoint(Boundary::GcReceiptSynchronization)?;
        synchronize_directory(&self.gc)
    }

    pub(super) fn store_remove_intent(&mut self) -> io::Result<()> {
        let context = self.context.as_ref().ok_or_else(no_retirement)?;
        let bytes = match exact_record::read_exact_optional(&self.gc, RECEIPT, 320) {
            Ok(Some(bytes)) => bytes,
            Ok(None) => {
                return Err(invalid(
                    super::FilesystemGcRefusal::ReceiptAbsentBeforeIntentRemoval,
                ));
            }
            Err(ExactRecordError::Io(source)) => return Err(source),
            Err(error @ ExactRecordError::Refused(_)) => {
                return Err(error.into_io());
            }
        };
        let intent = AdmittedGcRetirementIntent::decode(context.intent.encoded())
            .map_err(|source| io::Error::new(io::ErrorKind::InvalidData, source))?;
        let complete = AdmittedGcRetirementReceipt::decode(&bytes, &intent)
            .map_err(|source| io::Error::new(io::ErrorKind::InvalidData, source))?;
        if self.context()?.receipt.is_none() {
            let receipt = CanonicalGcRetirementReceipt::from_intent(
                &context_intent(self)?,
                complete.receipt().pool_state_digest(),
            );
            self.context()?.receipt = Some(receipt);
        }
        self.storage_checkpoint(Boundary::IntentUnlink)?;
        self.gc.remove_file(INTENT)?;
        self.storage_checkpoint(Boundary::IntentAbsence)?;
        exact_record::require_absent(&self.gc, INTENT).map_err(ExactRecordError::into_io)
    }

    pub(super) fn store_synchronize_gc_after_intent_removal(&mut self) -> io::Result<()> {
        self.storage_checkpoint(Boundary::GcIntentRemovalSynchronization)?;
        synchronize_directory(&self.gc)
    }
}

fn context_intent(
    authority: &FilesystemGcAuthority,
) -> io::Result<super::CanonicalGcRetirementIntent> {
    authority
        .context
        .as_ref()
        .map(|context| context.intent.clone())
        .ok_or_else(no_retirement)
}
