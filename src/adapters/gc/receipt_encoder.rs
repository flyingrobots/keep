//! This boundary module owns canonical GC retirement receipt encoding.

use super::{CanonicalGcRetirementReceipt, GcRetirementReceipt, receipt_format as format};

// Every emitted expression must have its declared array width, and their total
// must equal the destination. Together these compile-time checks prove that the
// iterator writes cannot truncate or leave a slot unfilled. No input sets a width.
macro_rules! receipt_bytes {
    ($length:expr; $($width:expr => $value:expr),+ $(,)?) => {{
        const _: [(); $length] = [(); 0 $(+ $width)+];
        let mut encoded = [0_u8; $length];
        let mut slots = encoded.iter_mut();
        $(
            let bytes: [u8; $width] = $value;
            for (slot, byte) in slots.by_ref().take($width).zip(bytes) {
                *slot = byte;
            }
        )+
        encoded
    }};
}

pub(super) fn encode(receipt: GcRetirementReceipt) -> CanonicalGcRetirementReceipt {
    let preimage = receipt_bytes!(format::CHECKSUM_OFFSET;
        format::RESERVED_OFFSET => fields(&receipt),
        format::RESERVED_LENGTH => [0; format::RESERVED_LENGTH],
    );
    let encoded = receipt_bytes!(format::ENCODED_LENGTH;
        format::CHECKSUM_OFFSET => preimage,
        32 => format::checksum(&preimage),
    );
    CanonicalGcRetirementReceipt::admitted(&encoded, receipt)
}

fn fields(receipt: &GcRetirementReceipt) -> [u8; format::RESERVED_OFFSET] {
    receipt_bytes!(format::RESERVED_OFFSET;
        16 => format::MAGIC,
        2 => format::VERSION.to_be_bytes(),
        2 => format::RECORD_LENGTH.to_be_bytes(),
        4 => 0_u32.to_be_bytes(),
        8 => receipt.generation().get().to_be_bytes(),
        32 => *receipt.intent_digest().as_bytes(),
        32 => *receipt.retired_candidate_set_digest().as_bytes(),
        32 => *receipt.pool_state_digest().as_bytes(),
        8 => receipt.liveness_generation().get().to_be_bytes(),
        32 => *receipt.manifest_digest().as_bytes(),
        8 => receipt.catalog_generation().get().to_be_bytes(),
        32 => *receipt.catalog_digest().as_bytes(),
        8 => receipt.reader_lock().device().get().to_be_bytes(),
        8 => receipt.reader_lock().mount().get().to_be_bytes(),
        8 => receipt.reader_lock().file().get().to_be_bytes(),
        8 => receipt.synchronization_count().to_be_bytes(),
    )
}
