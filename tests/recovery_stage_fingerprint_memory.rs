//! Isolated heap-allocation evidence for recovery-stage fingerprinting.

use std::error::Error;
use std::io::Cursor;

use allocation_counter::{AllocationInfo, measure};
use keep::{RecoveryStage, RecoveryStageMetadata, fingerprint_recovery_stage};

#[test]
fn fingerprinting_retains_no_stage_bytes_and_allocates_nothing() -> Result<(), Box<dyn Error>> {
    let bytes = [0x5a_u8; 16_384];
    let length = u64::try_from(bytes.len())?;
    let metadata = RecoveryStageMetadata::new(RecoveryStage::Segment, length)?;
    let mut result = None;

    let allocations = measure(|| {
        result = Some(fingerprint_recovery_stage(
            metadata,
            Cursor::new(bytes.as_slice()),
        ));
    });
    let evidence = result.ok_or("stage fingerprint measurement did not run")??;

    assert_eq!(evidence.length().get(), length);
    assert_eq!(allocations, AllocationInfo::default());
    Ok(())
}
