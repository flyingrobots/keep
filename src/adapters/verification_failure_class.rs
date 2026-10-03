//! This module owns resource-versus-content classification at durable ingress.

use crate::{
    CatalogAdmissionError, CatalogRestartError, LayoutDecodeError, SegmentReadError,
    SegmentRecordAdmissionError,
};

#[derive(Clone, Copy)]
pub(super) enum FailureClass {
    Operational,
    Corrupt,
}

pub(super) const fn layout_class(error: &LayoutDecodeError) -> FailureClass {
    match error {
        LayoutDecodeError::Allocation { .. }
        | LayoutDecodeError::EntryCountHostWidth { .. }
        | LayoutDecodeError::HostRecordLengthOutOfRange { .. }
        | LayoutDecodeError::ConfiguredEntryLimitExceeded { .. } => FailureClass::Operational,
        LayoutDecodeError::TruncatedHeader { .. }
        | LayoutDecodeError::InvalidMagic { .. }
        | LayoutDecodeError::UnsupportedFormatVersion { .. }
        | LayoutDecodeError::UnsupportedCodec { .. }
        | LayoutDecodeError::UnknownFlags { .. }
        | LayoutDecodeError::WrongHeaderLength { .. }
        | LayoutDecodeError::WrongEntryLength { .. }
        | LayoutDecodeError::UnsupportedChecksumAlgorithm { .. }
        | LayoutDecodeError::UnsupportedChunkHashAlgorithm { .. }
        | LayoutDecodeError::UnsupportedChunkIdentityVersion { .. }
        | LayoutDecodeError::NonzeroReserved { .. }
        | LayoutDecodeError::EntryCountLimitExceeded { .. }
        | LayoutDecodeError::RecordLengthLimitExceeded { .. }
        | LayoutDecodeError::RecordLengthArithmetic { .. }
        | LayoutDecodeError::RecordLengthMismatch { .. }
        | LayoutDecodeError::EntryCountLengthMismatch { .. }
        | LayoutDecodeError::TruncatedRecord { .. }
        | LayoutDecodeError::TrailingData { .. }
        | LayoutDecodeError::ChecksumMismatch { .. }
        | LayoutDecodeError::BlobId { .. }
        | LayoutDecodeError::UnsupportedStorageProfileVersion { .. }
        | LayoutDecodeError::UnsupportedStorageProfileAlgorithm { .. }
        | LayoutDecodeError::StorageProfile { .. }
        | LayoutDecodeError::ZeroChunkLength { .. }
        | LayoutDecodeError::Validation { .. }
        | LayoutDecodeError::LayoutIdentity { .. } => FailureClass::Corrupt,
    }
}

pub(super) const fn segment_class(error: &SegmentReadError) -> FailureClass {
    match error {
        SegmentReadError::RecordCountLimit { .. }
        | SegmentReadError::RecordCountHostWidth { .. }
        | SegmentReadError::IdentityIndexAllocation { .. }
        | SegmentReadError::RecordLengthHostWidth { .. }
        | SegmentReadError::OffsetArithmetic { .. }
        | SegmentReadError::RecordIndexArithmetic { .. }
        | SegmentReadError::RecordCountArithmetic { .. } => FailureClass::Operational,
        SegmentReadError::RecordAdmission { source, .. } => match source {
            SegmentRecordAdmissionError::Layout { source } => layout_class(source),
            SegmentRecordAdmissionError::ChunkHash { .. }
            | SegmentRecordAdmissionError::PayloadLengthHostWidth { .. }
            | SegmentRecordAdmissionError::RecordLengthArithmetic { .. } => {
                FailureClass::Operational
            }
            SegmentRecordAdmissionError::Header { .. }
            | SegmentRecordAdmissionError::ChunkIdentityMismatch { .. }
            | SegmentRecordAdmissionError::PayloadLengthMismatch { .. } => FailureClass::Corrupt,
        },
        SegmentReadError::WrongLength { .. }
        | SegmentReadError::Header { .. }
        | SegmentReadError::Seal { .. }
        | SegmentReadError::RecordHeaderTruncated { .. }
        | SegmentReadError::RecordHeader { .. }
        | SegmentReadError::RecordTruncated { .. }
        | SegmentReadError::RecordDecode { .. }
        | SegmentReadError::TrailingRecordBytes { .. }
        | SegmentReadError::DuplicateRecordIdentity { .. } => FailureClass::Corrupt,
    }
}

pub(super) fn catalog_class(error: &CatalogRestartError) -> FailureClass {
    match error {
        CatalogRestartError::Io { .. }
        | CatalogRestartError::LengthArithmetic { .. }
        | CatalogRestartError::Allocation { .. }
        | CatalogRestartError::SegmentIndexLength
        | CatalogRestartError::SegmentIndexAllocation { .. }
        | CatalogRestartError::RetainedSegmentBytes { .. }
        | CatalogRestartError::RetainedSegmentByteArithmetic { .. } => FailureClass::Operational,
        CatalogRestartError::Segment { source, .. } => segment_class(source),
        CatalogRestartError::CatalogAdmission { source } => admission_class(source),
        CatalogRestartError::NotRegular { .. }
        | CatalogRestartError::Length { .. }
        | CatalogRestartError::Head { .. }
        | CatalogRestartError::Catalog { .. }
        | CatalogRestartError::CatalogCoordinate { .. }
        | CatalogRestartError::SegmentCoordinate { .. }
        | CatalogRestartError::Snapshot { .. } => FailureClass::Corrupt,
    }
}

fn admission_class(error: &CatalogAdmissionError) -> FailureClass {
    match error {
        CatalogAdmissionError::EntryCountHostWidth { .. }
        | CatalogAdmissionError::Allocation { .. } => FailureClass::Operational,
        CatalogAdmissionError::Segment { source, .. } => segment_class(source),
        CatalogAdmissionError::Catalog { .. }
        | CatalogAdmissionError::SegmentCountOutOfBounds { .. }
        | CatalogAdmissionError::DuplicateSegment { .. }
        | CatalogAdmissionError::MissingSegment { .. }
        | CatalogAdmissionError::UnreferencedSegment { .. }
        | CatalogAdmissionError::LocationNotTopLevel { .. }
        | CatalogAdmissionError::RecordIdentityMismatch { .. }
        | CatalogAdmissionError::RecordChecksumMismatch { .. } => FailureClass::Corrupt,
    }
}
