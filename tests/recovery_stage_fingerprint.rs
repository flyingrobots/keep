//! Bounded recovery-stage fingerprint laws.

#[path = "recovery_stage_fingerprint/reader_laws.rs"]
mod reader_laws;

use std::error::Error;
use std::io::Cursor;

use keep::{RecoveryStage, RecoveryStageMetadata, fingerprint_recovery_stage};

const DOMAIN: &[u8] = b"KEEP:RECOVERY:STAGE\0";

#[test]
fn every_stage_uses_its_exact_protocol_maximum() {
    assert_eq!(RecoveryStage::Segment.maximum_length(), 1_073_741_824);
    assert_eq!(RecoveryStage::Catalog.maximum_length(), 167_772_352);
    assert_eq!(RecoveryStage::NextHead.maximum_length(), 128);
}

#[test]
fn exact_bytes_match_an_independent_framed_blake3_oracle() -> Result<(), Box<dyn Error>> {
    for bytes in [b"".as_slice(), b"recovery evidence".as_slice()] {
        let metadata =
            RecoveryStageMetadata::new(RecoveryStage::Segment, u64::try_from(bytes.len())?)?;
        let evidence = fingerprint_recovery_stage(metadata, Cursor::new(bytes))?;

        assert_eq!(evidence.stage(), RecoveryStage::Segment);
        assert_eq!(evidence.length().get(), u64::try_from(bytes.len())?);
        assert_eq!(evidence.fingerprint().algorithm().code(), 1);
        assert_eq!(evidence.fingerprint().as_bytes(), &oracle(bytes)?);
    }
    Ok(())
}

fn oracle(bytes: &[u8]) -> Result<[u8; 32], Box<dyn Error>> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(DOMAIN);
    hasher.update(&1_u16.to_be_bytes());
    hasher.update(&[1_u8]);
    hasher.update(bytes);
    hasher.update(&u64::try_from(bytes.len())?.to_be_bytes());
    Ok(*hasher.finalize().as_bytes())
}
