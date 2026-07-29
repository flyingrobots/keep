//! Optional sealed segment transition preceding catalog publication.

use super::AdmittedSegment;

/// Segment-pool work required before publishing one catalog generation.
#[derive(Clone, Copy)]
pub enum SegmentPublication<'selection, 'records> {
    /// Every catalog-referenced segment is already durable in the pool.
    None,
    /// One fixed sealed segment stage must become a durable pool entry.
    One(&'selection AdmittedSegment<'records>),
}
