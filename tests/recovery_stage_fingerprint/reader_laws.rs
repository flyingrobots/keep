//! Adversarial recovery-stage reader laws.

use std::error::Error;
use std::io::{self, Cursor, Read};

use keep::{
    RecoveryStage, RecoveryStageFingerprintError, RecoveryStageMetadata,
    RecoveryStageMetadataError, fingerprint_recovery_stage,
};

#[test]
fn oversized_metadata_refuses_before_reading() -> Result<(), Box<dyn Error>> {
    let reader = CountingReader::new(Cursor::new(Vec::<u8>::new()));
    let observed = RecoveryStage::NextHead
        .maximum_length()
        .checked_add(1)
        .ok_or("test maximum overflow")?;

    let Err(error) = RecoveryStageMetadata::new(RecoveryStage::NextHead, observed) else {
        return Err("oversized metadata was admitted".into());
    };

    assert!(matches!(
        error,
        RecoveryStageMetadataError::Oversized {
            stage: RecoveryStage::NextHead,
            maximum: 128,
            observed: 129,
        }
    ));
    assert_eq!(reader.calls, 0);
    Ok(())
}

#[test]
fn maximum_plus_one_stream_refuses_at_the_first_excess_byte() -> Result<(), Box<dyn Error>> {
    let bytes = vec![0_u8; 129];
    let metadata = RecoveryStageMetadata::new(RecoveryStage::NextHead, 128)?;
    let Err(error) = fingerprint_recovery_stage(metadata, Cursor::new(bytes)) else {
        return Err("oversized stream was admitted".into());
    };

    assert!(matches!(
        error,
        RecoveryStageFingerprintError::EvidenceOversized {
            stage: RecoveryStage::NextHead,
            maximum: 128,
            observed_at_least: 129,
        }
    ));
    Ok(())
}

#[test]
fn interrupted_and_short_reads_preserve_the_fingerprint() -> Result<(), Box<dyn Error>> {
    let bytes = b"partition-independent stage evidence";
    let metadata = RecoveryStageMetadata::new(RecoveryStage::Catalog, u64::try_from(bytes.len())?)?;
    let expected = fingerprint_recovery_stage(metadata, Cursor::new(bytes))?;
    let reader = InterruptedReader::new(bytes);

    let observed = fingerprint_recovery_stage(metadata, reader)?;

    assert_eq!(observed, expected);
    Ok(())
}

#[test]
fn read_failure_retains_stage_offset_and_source() -> Result<(), Box<dyn Error>> {
    let reader = FailingReader::new(b"prefix", 6);
    let metadata = RecoveryStageMetadata::new(RecoveryStage::Segment, 6)?;
    let Err(error) = fingerprint_recovery_stage(metadata, reader) else {
        return Err("failing stage reader was admitted".into());
    };

    assert!(matches!(
        error,
        RecoveryStageFingerprintError::Read {
            stage: RecoveryStage::Segment,
            offset: 6,
            ref source,
        } if source.kind() == io::ErrorKind::PermissionDenied
    ));
    Ok(())
}

#[test]
fn overreported_read_count_is_an_exact_contract_refusal() -> Result<(), Box<dyn Error>> {
    let metadata = RecoveryStageMetadata::new(RecoveryStage::NextHead, 0)?;
    let Err(error) = fingerprint_recovery_stage(metadata, OverreportingReader) else {
        return Err("overreporting stage reader was admitted".into());
    };

    assert!(matches!(
        error,
        RecoveryStageFingerprintError::ReaderContract {
            stage: RecoveryStage::NextHead,
            offset: 0,
            offered: 129,
            observed: 130,
        }
    ));
    Ok(())
}

struct CountingReader<R> {
    inner: R,
    calls: usize,
}

impl<R> CountingReader<R> {
    const fn new(inner: R) -> Self {
        Self { inner, calls: 0 }
    }
}

impl<R: Read> Read for CountingReader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.calls = self
            .calls
            .checked_add(1)
            .ok_or_else(|| io::Error::other("test call count overflow"))?;
        self.inner.read(buffer)
    }
}

struct InterruptedReader<'a> {
    bytes: &'a [u8],
    offset: usize,
    interrupt: bool,
}

impl<'a> InterruptedReader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            offset: 0,
            interrupt: true,
        }
    }
}

impl Read for InterruptedReader<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.interrupt {
            self.interrupt = false;
            return Err(io::ErrorKind::Interrupted.into());
        }
        self.interrupt = true;
        let remaining = self.bytes.get(self.offset..).unwrap_or_default();
        let count = remaining.len().min(3).min(buffer.len());
        let target = buffer
            .get_mut(..count)
            .ok_or_else(|| io::Error::other("test buffer range missing"))?;
        let source = remaining
            .get(..count)
            .ok_or_else(|| io::Error::other("test source range missing"))?;
        target.copy_from_slice(source);
        self.offset = self
            .offset
            .checked_add(count)
            .ok_or_else(|| io::Error::other("test offset overflow"))?;
        Ok(count)
    }
}

struct FailingReader<'a> {
    prefix: Cursor<&'a [u8]>,
    failure_offset: u64,
}

impl<'a> FailingReader<'a> {
    const fn new(prefix: &'a [u8], failure_offset: u64) -> Self {
        Self {
            prefix: Cursor::new(prefix),
            failure_offset,
        }
    }
}

impl Read for FailingReader<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.prefix.position() == self.failure_offset {
            Err(io::ErrorKind::PermissionDenied.into())
        } else {
            self.prefix.read(buffer)
        }
    }
}

struct OverreportingReader;

impl Read for OverreportingReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        buffer
            .len()
            .checked_add(1)
            .ok_or_else(|| io::Error::other("test overreport overflow"))
    }
}
