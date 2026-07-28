#![no_main]

//! This target owns nested immutable-segment format parser fuzzing.

use keep::{
    AdmittedSegment, ChecksummedSegmentRecord, LayoutEntryLimit, SegmentHeader, SegmentReadPolicy,
    SegmentRecordHeader, SegmentRecordLimit, SegmentSeal,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    let Some((&selector, input)) = bytes.split_first() else {
        return;
    };
    match selector {
        0 => header(input),
        1 => record_header(input),
        2 => record(input),
        3 => seal(input),
        _ => segment(input),
    }
});

fn header(input: &[u8]) {
    if let Ok(header) = SegmentHeader::decode(input) {
        assert_eq!(header.encode().as_slice(), input);
    }
}

fn record_header(input: &[u8]) {
    if let Ok(header) = SegmentRecordHeader::decode(input) {
        assert_eq!(header.encode().as_slice(), input);
    }
}

fn record(input: &[u8]) {
    if let Ok(record) = ChecksummedSegmentRecord::decode(input) {
        let _admitted = record.admit(LayoutEntryLimit::MAXIMUM);
    }
}

fn seal(input: &[u8]) {
    let Some(prefix_length) = input.len().checked_sub(SegmentSeal::ENCODED_LENGTH) else {
        return;
    };
    let Some(prefix) = input.get(..prefix_length) else {
        return;
    };
    let Some(encoded) = input.get(prefix_length..) else {
        return;
    };
    if let Ok(seal) = SegmentSeal::decode(prefix, encoded) {
        assert_eq!(seal.encode().as_slice(), encoded);
    }
}

fn segment(input: &[u8]) {
    let policy = SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM);
    if let Ok(segment) = AdmittedSegment::decode(input, policy) {
        assert_eq!(segment.encoded(), input);
        assert_eq!(
            u32::try_from(segment.records().count()),
            Ok(segment.record_count())
        );
    }
}
