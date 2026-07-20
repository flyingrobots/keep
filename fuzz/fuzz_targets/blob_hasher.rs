#![no_main]

use keep::{BlobHasher, BlobId};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    let expected_result = BlobId::hash_bytes(bytes);
    assert!(expected_result.is_ok());
    let Ok(expected) = expected_result else {
        return;
    };
    let mut hasher = BlobHasher::new();
    let mut remainder = bytes;
    for width_source in bytes.iter().copied() {
        if remainder.is_empty() {
            break;
        }
        let requested = usize::from(width_source).saturating_add(1);
        let width = requested.min(remainder.len());
        let (partition, next) = remainder.split_at(width);
        let update = hasher.update(partition);
        assert!(update.is_ok());
        if update.is_err() {
            return;
        }
        remainder = next;
    }
    let final_update = hasher.update(remainder);
    assert!(final_update.is_ok());
    if final_update.is_err() {
        return;
    }
    assert_eq!(hasher.finish(), expected);
});
