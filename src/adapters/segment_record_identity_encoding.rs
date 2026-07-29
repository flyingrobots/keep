//! Canonical segment-record identity-slot encoding.

use super::SegmentRecordIdentity;

pub(super) const fn encode(identity: SegmentRecordIdentity) -> [u8; 60] {
    let mut slot = [0_u8; 60];
    match identity {
        SegmentRecordIdentity::Chunk(id) => {
            let (length, remaining) = slot.split_at_mut(4);
            length.copy_from_slice(&id.length().get().to_be_bytes());
            let (digest, _unused) = remaining.split_at_mut(32);
            digest.copy_from_slice(id.digest());
        }
        SegmentRecordIdentity::Layout(id) => slot.copy_from_slice(&id.encode_binary()),
    }
    slot
}
