//! Supported segment-record kind coordinates.

use super::segment_header::MAXIMUM_RECORD_PAYLOAD_LENGTH;
use super::{SegmentRecordHeaderError, SegmentRecordIdentity};
use crate::LayoutRecordLength;

#[derive(Clone, Copy)]
pub(super) enum SegmentRecordKind {
    Chunk,
    Layout,
}

impl SegmentRecordKind {
    pub(super) const fn admit(observed: u8) -> Result<Self, SegmentRecordHeaderError> {
        match observed {
            1 => Ok(Self::Chunk),
            2 => Ok(Self::Layout),
            _ => Err(SegmentRecordHeaderError::UnknownRecordKind { observed }),
        }
    }

    pub(super) const fn from_identity(identity: SegmentRecordIdentity) -> Self {
        match identity {
            SegmentRecordIdentity::Chunk(_) => Self::Chunk,
            SegmentRecordIdentity::Layout(_) => Self::Layout,
        }
    }

    pub(super) const fn code(self) -> u8 {
        match self {
            Self::Chunk => 1,
            Self::Layout => 2,
        }
    }

    pub(super) const fn identity_length(self) -> u16 {
        match self {
            Self::Chunk => 36,
            Self::Layout => 60,
        }
    }

    pub(super) const fn payload_bounds(self) -> (u64, u64) {
        match self {
            Self::Chunk => (1, MAXIMUM_RECORD_PAYLOAD_LENGTH),
            Self::Layout => (
                LayoutRecordLength::MINIMUM.get(),
                LayoutRecordLength::MAXIMUM.get(),
            ),
        }
    }
}
