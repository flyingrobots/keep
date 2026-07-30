//! This boundary module owns canonical migration-receipt encoding.

use super::migration_receipt_initial_state::{
    empty_disposition_digest, initial_gc_digest, initial_retention_digest,
};
use super::{
    CanonicalStoreFormatMarker, CanonicalStoreMigrationIntent, CanonicalStoreMigrationReceipt,
    MigrationSynchronizationMask, migration_receipt_format as format,
};

pub(super) fn encode(
    intent: &CanonicalStoreMigrationIntent,
    marker: &CanonicalStoreFormatMarker,
) -> CanonicalStoreMigrationReceipt {
    let mut encoded = [0_u8; format::ENCODED_LENGTH];
    let (preimage, checksum_slot) = encoded.split_at_mut(format::CHECKSUM_OFFSET);
    write_preimage(preimage, intent, marker);
    checksum_slot.copy_from_slice(&format::checksum(preimage));
    CanonicalStoreMigrationReceipt::admitted(encoded)
}

fn write_preimage(
    output: &mut [u8],
    intent: &CanonicalStoreMigrationIntent,
    marker: &CanonicalStoreFormatMarker,
) {
    let (magic, output) = output.split_at_mut(16);
    magic.copy_from_slice(&format::MAGIC);
    let (version, output) = output.split_at_mut(2);
    version.copy_from_slice(&format::VERSION.to_be_bytes());
    let (record_length, output) = output.split_at_mut(2);
    record_length.copy_from_slice(&format::RECORD_LENGTH.to_be_bytes());
    let (flags, output) = output.split_at_mut(4);
    flags.copy_from_slice(&0_u32.to_be_bytes());
    let (intent_digest, output) = output.split_at_mut(32);
    intent_digest.copy_from_slice(intent.digest().as_bytes());
    let (store_identifier, output) = output.split_at_mut(32);
    store_identifier.copy_from_slice(intent.store_identifier().as_bytes());
    let (marker_digest, output) = output.split_at_mut(32);
    marker_digest.copy_from_slice(marker.digest().as_bytes());
    let (retention_digest, output) = output.split_at_mut(32);
    retention_digest.copy_from_slice(initial_retention_digest().as_bytes());
    let (gc_digest, output) = output.split_at_mut(32);
    gc_digest.copy_from_slice(initial_gc_digest().as_bytes());
    let (disposition_digest, synchronization_mask) = output.split_at_mut(32);
    disposition_digest.copy_from_slice(empty_disposition_digest().as_bytes());
    synchronization_mask.copy_from_slice(
        &MigrationSynchronizationMask::complete()
            .bits()
            .to_be_bytes(),
    );
}
