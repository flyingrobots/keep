//! Append-only segment stage before explicit sealing.

use super::segment_digest_builder::SegmentDigestBuilder;
use super::{
    AdmittedSegmentRecord, SealedSegment, SegmentDurabilityPhase, SegmentHeader,
    SegmentRecordIdentity, SegmentRecordLimit, SegmentSeal, SegmentStage, SegmentWriteError,
    SegmentWritePhase, segment_seal_builder, segment_stage_write,
};

/// An exclusively owned append-only segment stage.
///
/// Each mutating operation consumes this state. Any write or durability
/// failure therefore prevents accidental continuation from an ambiguous
/// partial tail. Dropping this value never seals or publishes the stage.
#[must_use]
pub struct StagedSegment<S>
where
    S: SegmentStage,
{
    stage: S,
    digest: SegmentDigestBuilder,
    identities: Vec<SegmentRecordIdentity>,
    record_limit: SegmentRecordLimit,
    record_count: u32,
    bytes_written: u64,
}

impl<S> StagedSegment<S>
where
    S: SegmentStage,
{
    /// Begins a new stage by writing the complete canonical segment header.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentWriteError`] when the stage cannot complete the exact
    /// fixed header write.
    pub fn begin(
        mut stage: S,
        record_limit: SegmentRecordLimit,
    ) -> Result<Self, SegmentWriteError> {
        let header = SegmentHeader::admitted().encode();
        let mut bytes_written = 0;
        segment_stage_write::write_all(
            &mut stage,
            &header,
            SegmentWritePhase::Header,
            &mut bytes_written,
        )?;
        let mut digest = SegmentDigestBuilder::new();
        digest.update(&header);
        Ok(Self {
            stage,
            digest,
            identities: Vec::new(),
            record_limit,
            record_count: 0,
            bytes_written,
        })
    }

    /// Appends one complete content-admitted record.
    ///
    /// All count, duplicate, allocation, and complete-segment length checks
    /// occur before the first record byte is written.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentWriteError`] when pre-admission or any exact record
    /// write fails. The consumed stage cannot be resumed through this API.
    pub fn append(mut self, record: AdmittedSegmentRecord<'_>) -> Result<Self, SegmentWriteError> {
        let next_count = self.prepare_append(record)?;
        let header = record.header().encode();
        self.write_record_part(&header, SegmentWritePhase::RecordHeader)?;
        self.write_record_part(record.payload(), SegmentWritePhase::RecordPayload)?;
        self.write_record_part(
            record.checksum().as_bytes(),
            SegmentWritePhase::RecordChecksum,
        )?;
        self.identities.push(record.identity());
        self.record_count = next_count;
        Ok(self)
    }

    /// Flushes and synchronizes the reusable prefix, appends the exact seal,
    /// then flushes and synchronizes the sealed stage.
    ///
    /// # Errors
    ///
    /// Returns [`SegmentWriteError`] at the exact failed flush,
    /// synchronization, seal-construction, or seal-write boundary.
    #[must_use = "a successful seal must be retained for verified publication"]
    pub fn seal(mut self) -> Result<SealedSegment<S>, SegmentWriteError> {
        self.flush(SegmentDurabilityPhase::RecordPrefix)?;
        self.synchronize(SegmentDurabilityPhase::RecordPrefix)?;
        let seal = self.build_seal()?;
        let encoded = seal.encode();
        segment_stage_write::write_all(
            &mut self.stage,
            &encoded,
            SegmentWritePhase::Seal,
            &mut self.bytes_written,
        )?;
        self.flush(SegmentDurabilityPhase::SealedSegment)?;
        self.synchronize(SegmentDurabilityPhase::SealedSegment)?;
        Ok(SealedSegment::admitted(
            self.stage,
            self.record_count,
            seal.segment_length(),
            seal.digest(),
        ))
    }

    fn prepare_append(
        &mut self,
        record: AdmittedSegmentRecord<'_>,
    ) -> Result<u32, SegmentWriteError> {
        let identity = record.identity();
        if self.identities.contains(&identity) {
            return Err(SegmentWriteError::DuplicateRecordIdentity { identity });
        }
        let next_count =
            self.record_count
                .checked_add(1)
                .ok_or(SegmentWriteError::RecordCountArithmetic {
                    observed: self.record_count,
                })?;
        if next_count > self.record_limit.get() {
            return Err(SegmentWriteError::RecordCountLimit {
                maximum: self.record_limit.get(),
                observed: next_count,
            });
        }
        self.validate_length(record.header().record_length().get())?;
        self.identities
            .try_reserve(1)
            .map_err(|source| SegmentWriteError::IdentityIndexAllocation { identity, source })?;
        Ok(next_count)
    }

    fn validate_length(&self, record_length: u64) -> Result<(), SegmentWriteError> {
        let bytes_before_seal = self.bytes_written.checked_add(record_length).ok_or(
            SegmentWriteError::SegmentLengthArithmetic {
                bytes_before_record: self.bytes_written,
                record_length,
            },
        )?;
        let observed = bytes_before_seal
            .checked_add(
                u64::try_from(SegmentSeal::ENCODED_LENGTH).map_err(|_source| {
                    SegmentWriteError::SegmentLengthArithmetic {
                        bytes_before_record: self.bytes_written,
                        record_length,
                    }
                })?,
            )
            .ok_or(SegmentWriteError::SegmentLengthArithmetic {
                bytes_before_record: self.bytes_written,
                record_length,
            })?;
        let maximum = SegmentHeader::admitted().maximum_segment_length();
        if observed > maximum {
            return Err(SegmentWriteError::SegmentLengthLimit { maximum, observed });
        }
        Ok(())
    }

    fn write_record_part(
        &mut self,
        bytes: &[u8],
        phase: SegmentWritePhase,
    ) -> Result<(), SegmentWriteError> {
        segment_stage_write::write_all(&mut self.stage, bytes, phase, &mut self.bytes_written)?;
        self.digest.update(bytes);
        Ok(())
    }

    fn build_seal(&self) -> Result<SegmentSeal, SegmentWriteError> {
        let provisional = segment_seal_builder::from_digest_builder(
            self.record_count,
            self.bytes_written,
            &self.digest,
        )
        .map_err(|source| SegmentWriteError::Seal { source })?;
        Ok(provisional)
    }

    fn flush(&mut self, phase: SegmentDurabilityPhase) -> Result<(), SegmentWriteError> {
        self.stage
            .flush()
            .map_err(|source| SegmentWriteError::Flush { phase, source })
    }

    fn synchronize(&mut self, phase: SegmentDurabilityPhase) -> Result<(), SegmentWriteError> {
        self.stage
            .synchronize()
            .map_err(|source| SegmentWriteError::Synchronize { phase, source })
    }
}
