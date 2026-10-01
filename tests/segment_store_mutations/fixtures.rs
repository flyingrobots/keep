//! The frozen segment-store fixtures a mutation may start from.

use std::io;

use super::ledger::Format;
use crate::support::invalid_corpus;

const V1: [(&str, &str); 6] = [
    (
        "one-zero-segment.hex",
        include_str!("../../conformance/segment-store/v1/one-zero-segment.hex"),
    ),
    (
        "empty-segment.hex",
        include_str!("../../conformance/segment-store/v1/empty-segment.hex"),
    ),
    (
        "one-zero-catalog.hex",
        include_str!("../../conformance/segment-store/v1/one-zero-catalog.hex"),
    ),
    (
        "one-zero-catalog-generation-two.hex",
        include_str!("../../conformance/segment-store/v1/one-zero-catalog-generation-two.hex"),
    ),
    (
        "one-zero-head.hex",
        include_str!("../../conformance/segment-store/v1/one-zero-head.hex"),
    ),
    (
        "one-zero-head-generation-two.hex",
        include_str!("../../conformance/segment-store/v1/one-zero-head-generation-two.hex"),
    ),
];

const V2: [(&str, &str); 9] = [
    (
        "format-marker.hex",
        include_str!("../../conformance/segment-store/v2/format-marker.hex"),
    ),
    (
        "migration-intent.hex",
        include_str!("../../conformance/segment-store/v2/migration-intent.hex"),
    ),
    (
        "migration-receipt.hex",
        include_str!("../../conformance/segment-store/v2/migration-receipt.hex"),
    ),
    (
        "one-anchor-root.hex",
        include_str!("../../conformance/segment-store/v2/one-anchor-root.hex"),
    ),
    (
        "one-root-manifest.hex",
        include_str!("../../conformance/segment-store/v2/one-root-manifest.hex"),
    ),
    (
        "one-root-head.hex",
        include_str!("../../conformance/segment-store/v2/one-root-head.hex"),
    ),
    (
        "one-candidate-gc-intent.hex",
        include_str!("../../conformance/segment-store/v2/one-candidate-gc-intent.hex"),
    ),
    (
        "one-candidate-gc-receipt.hex",
        include_str!("../../conformance/segment-store/v2/one-candidate-gc-receipt.hex"),
    ),
    (
        "one-orphan-retire-disposition.hex",
        include_str!("../../conformance/segment-store/v2/one-orphan-retire-disposition.hex"),
    ),
];

/// Returns one fixture's hexadecimal text.
///
/// # Errors
///
/// Returns a corpus error when the fixture is not frozen.
pub fn fixture(format: Format, name: &str) -> Result<&'static str, io::Error> {
    let table: &[(&str, &str)] = match format {
        Format::V1 => &V1,
        Format::V2 => &V2,
    };
    table
        .iter()
        .find(|(fixture, _)| *fixture == name)
        .map(|(_, hex)| *hex)
        .ok_or_else(|| invalid_corpus("mutation ledger names an unknown fixture"))
}
