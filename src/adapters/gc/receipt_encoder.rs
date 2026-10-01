//! This boundary module owns canonical GC retirement receipt encoding.

use super::{CanonicalGcRetirementReceipt, GcRetirementReceipt, receipt_format as format};

pub(super) fn encode(receipt: GcRetirementReceipt) -> CanonicalGcRetirementReceipt {
    let mut encoded = [0_u8; format::ENCODED_LENGTH];
    let (preimage, checksum_slot) = encoded.split_at_mut(format::CHECKSUM_OFFSET);
    write_preimage(preimage, &receipt);
    checksum_slot.copy_from_slice(&format::checksum(preimage));
    CanonicalGcRetirementReceipt::admitted(&encoded, receipt)
}

fn write_preimage(output: &mut [u8], receipt: &GcRetirementReceipt) {
    let (magic, output) = output.split_at_mut(16);
    magic.copy_from_slice(&format::MAGIC);
    let (version, output) = output.split_at_mut(2);
    version.copy_from_slice(&format::VERSION.to_be_bytes());
    let (record_length, output) = output.split_at_mut(2);
    record_length.copy_from_slice(&format::RECORD_LENGTH.to_be_bytes());
    let (flags, output) = output.split_at_mut(4);
    flags.copy_from_slice(&0_u32.to_be_bytes());
    let (generation, output) = output.split_at_mut(8);
    generation.copy_from_slice(&receipt.generation().get().to_be_bytes());
    let (intent_digest, output) = output.split_at_mut(32);
    intent_digest.copy_from_slice(receipt.intent_digest().as_bytes());
    let (retired_set, output) = output.split_at_mut(32);
    retired_set.copy_from_slice(receipt.retired_candidate_set_digest().as_bytes());
    let (pool_state, output) = output.split_at_mut(32);
    pool_state.copy_from_slice(receipt.pool_state_digest().as_bytes());
    let (liveness, output) = output.split_at_mut(8);
    liveness.copy_from_slice(&receipt.liveness_generation().get().to_be_bytes());
    let (manifest_digest, output) = output.split_at_mut(32);
    manifest_digest.copy_from_slice(receipt.manifest_digest().as_bytes());
    let (catalog_generation, output) = output.split_at_mut(8);
    catalog_generation.copy_from_slice(&receipt.catalog_generation().get().to_be_bytes());
    let (catalog_digest, output) = output.split_at_mut(32);
    catalog_digest.copy_from_slice(receipt.catalog_digest().as_bytes());
    let (device, output) = output.split_at_mut(8);
    device.copy_from_slice(&receipt.reader_lock().device().to_be_bytes());
    let (mount, output) = output.split_at_mut(8);
    mount.copy_from_slice(&receipt.reader_lock().mount().to_be_bytes());
    let (file, output) = output.split_at_mut(8);
    file.copy_from_slice(&receipt.reader_lock().file().to_be_bytes());
    let (synchronization_count, reserved) = output.split_at_mut(8);
    synchronization_count.copy_from_slice(&receipt.synchronization_count().to_be_bytes());
    reserved.fill(0);
}
