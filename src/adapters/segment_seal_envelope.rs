//! Structurally admitted seal envelope awaiting physical-digest verification.

use super::segment_seal_decoder::DecodedSeal;
use super::{SegmentSeal, SegmentSealError, segment_seal_decoder, segment_seal_envelope_admission};

pub(super) struct SegmentSealEnvelope {
    fields: DecodedSeal,
}

impl SegmentSealEnvelope {
    pub(super) fn decode(prefix: &[u8], encoded: &[u8]) -> Result<Self, SegmentSealError> {
        let fields = segment_seal_decoder::decode_fields(encoded)?;
        segment_seal_envelope_admission::admit(prefix, encoded, &fields)?;
        Ok(Self { fields })
    }

    pub(super) const fn record_count(&self) -> u32 {
        self.fields.record_count
    }

    pub(super) fn verify(self, prefix: &[u8]) -> Result<SegmentSeal, SegmentSealError> {
        segment_seal_envelope_admission::verify(prefix, &self.fields)
    }
}
