//! This boundary module owns canonical recovery-disposition receipt encoding.

use super::{CanonicalRecoveryDispositionReceipt, RecoveryDispositionReceipt, disposition_format};

pub(super) fn encode(receipt: &RecoveryDispositionReceipt) -> CanonicalRecoveryDispositionReceipt {
    let mut encoded = [0_u8; disposition_format::ENCODED_LENGTH];
    let (preimage, checksum_slot) = encoded.split_at_mut(disposition_format::CHECKSUM_OFFSET);
    write_preimage(preimage, receipt);
    checksum_slot.copy_from_slice(&disposition_format::checksum(preimage));
    CanonicalRecoveryDispositionReceipt::admitted(&encoded, receipt)
}

fn write_preimage(output: &mut [u8], receipt: &RecoveryDispositionReceipt) {
    let artifact = receipt.artifact();
    let coordinates = receipt.coordinates();
    let (magic, output) = output.split_at_mut(16);
    magic.copy_from_slice(&disposition_format::MAGIC);
    let (version, output) = output.split_at_mut(2);
    version.copy_from_slice(&disposition_format::VERSION.to_be_bytes());
    let (record_length, output) = output.split_at_mut(2);
    record_length.copy_from_slice(&disposition_format::RECORD_LENGTH.to_be_bytes());
    let (flags, output) = output.split_at_mut(4);
    flags.copy_from_slice(&0_u32.to_be_bytes());
    let (kind, output) = output.split_at_mut(2);
    kind.copy_from_slice(&artifact.kind.code().to_be_bytes());
    let (decision, output) = output.split_at_mut(2);
    decision.copy_from_slice(&receipt.decision().code().to_be_bytes());
    let (classification, output) = output.split_at_mut(2);
    classification.copy_from_slice(&artifact.classification.code().to_be_bytes());
    let (reserved, output) = output.split_at_mut(2);
    reserved.fill(0);
    let (length, output) = output.split_at_mut(8);
    length.copy_from_slice(&artifact.length.to_be_bytes());
    let (identity, output) = output.split_at_mut(32);
    identity.copy_from_slice(artifact.identity_digest.as_bytes());
    let (content, output) = output.split_at_mut(32);
    content.copy_from_slice(artifact.content_digest.as_bytes());
    let (head_generation, output) = output.split_at_mut(8);
    head_generation.copy_from_slice(&coordinates.publication_generation.get().to_be_bytes());
    let (head_checksum, output) = output.split_at_mut(32);
    head_checksum.copy_from_slice(coordinates.publication_checksum.as_bytes());
    let (catalog_generation, output) = output.split_at_mut(8);
    catalog_generation.copy_from_slice(&coordinates.catalog_generation.get().to_be_bytes());
    let (catalog_digest, output) = output.split_at_mut(32);
    catalog_digest.copy_from_slice(coordinates.catalog_digest.as_bytes());
    let (liveness, output) = output.split_at_mut(8);
    let (manifest, output) = output.split_at_mut(32);
    match coordinates.retention {
        super::GcRetentionState::Empty => {
            liveness.copy_from_slice(&0_u64.to_be_bytes());
            manifest.copy_from_slice(&super::disposition_format::empty_retention_digest());
        }
        super::GcRetentionState::Published {
            generation,
            manifest_digest,
        } => {
            liveness.copy_from_slice(&generation.get().to_be_bytes());
            manifest.copy_from_slice(manifest_digest.as_bytes());
        }
    }
    let (device, output) = output.split_at_mut(8);
    device.copy_from_slice(&coordinates.reader_lock.device().get().to_be_bytes());
    let (mount, output) = output.split_at_mut(8);
    mount.copy_from_slice(&coordinates.reader_lock.mount().get().to_be_bytes());
    let (file, output) = output.split_at_mut(8);
    file.copy_from_slice(&coordinates.reader_lock.file().get().to_be_bytes());
    let (evidence, reserved) = output.split_at_mut(32);
    evidence.copy_from_slice(receipt.evidence_digest().as_bytes());
    reserved.fill(0);
}
