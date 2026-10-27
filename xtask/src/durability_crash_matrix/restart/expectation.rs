//! This module owns the independent expected crash-state model.

mod sequence;
mod steps;

use std::collections::{BTreeMap, BTreeSet};

use super::super::DurabilityCrashMatrixError;
use super::super::production_protocol::fixture::GoldenFixture;
use xtask::{DurabilityCrashCase, DurabilityCrashSequence};

pub(super) const WRITER_LOCK: &str = "writer.lock";
pub(super) const STAGING: &str = "staging";
pub(super) const SEGMENTS: &str = "segments";
pub(super) const CATALOGS: &str = "catalogs";
pub(super) const SEGMENT_STAGE: &str = "staging/current.seg";
pub(super) const CATALOG_STAGE: &str = "staging/current.cat";
pub(super) const NEXT_HEAD: &str = "head.next";
pub(super) const HEAD: &str = "HEAD";
pub(super) const READER_LOCK: &str = "reader.lock";
pub(super) const FORMAT: &str = "FORMAT";

/// Every protocol directory of a committed single-format store, parents first.
const PROTOCOL_DIRECTORIES: [&str; 9] = [
    STAGING,
    SEGMENTS,
    CATALOGS,
    "retention",
    "retention/roots",
    "retention/manifests",
    "gc",
    "recovery",
    "recovery/dispositions",
];

pub(super) enum ArtifactBytes {
    Empty,
    /// The canonical `FORMAT` marker of the corpus.
    Marker,
    Segment(usize),
    Catalog(usize),
    Head(usize),
}

impl ArtifactBytes {
    pub(super) const fn kind(&self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Marker => "marker",
            Self::Segment(_) => "segment",
            Self::Catalog(_) => "catalog",
            Self::Head(_) => "head",
        }
    }

    pub(super) fn resolve<'a>(
        &self,
        fixtures: &'a ArtifactFixtures,
    ) -> Result<&'a [u8], DurabilityCrashMatrixError> {
        let ArtifactFixtures {
            marker,
            segment,
            catalog,
            head,
        } = fixtures;
        match self {
            Self::Empty => Ok(&[]),
            Self::Marker => Ok(marker.bytes()),
            Self::Segment(end) => segment.prefix(*end),
            Self::Catalog(end) => catalog.prefix(*end),
            Self::Head(end) => head.prefix(*end),
        }
    }
}

/// The decoded corpus artifacts expected bytes are prefixes of.
pub(super) struct ArtifactFixtures {
    pub(super) marker: GoldenFixture,
    pub(super) segment: GoldenFixture,
    pub(super) catalog: GoldenFixture,
    pub(super) head: GoldenFixture,
}

impl ArtifactFixtures {
    pub(super) fn load() -> Result<Self, DurabilityCrashMatrixError> {
        Ok(Self {
            marker: GoldenFixture::format_marker()?,
            segment: GoldenFixture::segment()?,
            catalog: GoldenFixture::catalog()?,
            head: GoldenFixture::head()?,
        })
    }
}

pub(super) struct ExpectedStoreState {
    directories: BTreeSet<&'static str>,
    artifacts: BTreeMap<&'static str, ArtifactBytes>,
    hard_link: Option<(&'static str, &'static str)>,
}

impl ExpectedStoreState {
    pub(super) fn for_case(case: DurabilityCrashCase) -> Result<Self, DurabilityCrashMatrixError> {
        match case.point().sequence() {
            DurabilityCrashSequence::Segment => sequence::segment(case),
            DurabilityCrashSequence::Catalog => sequence::catalog(case),
            DurabilityCrashSequence::Head => sequence::head(case),
            DurabilityCrashSequence::RecoveryDiscard => sequence::recovery(case),
            DurabilityCrashSequence::Initialization => sequence::initialization(case),
            DurabilityCrashSequence::Retention
            | DurabilityCrashSequence::VersionTwoCatalog
            | DurabilityCrashSequence::VersionTwoInitialization => {
                Err(DurabilityCrashMatrixError::PointSequenceMismatch {
                    point: case.point(),
                })
            }
        }
    }

    pub(super) fn paths(&self) -> BTreeSet<String> {
        self.directories
            .iter()
            .copied()
            .chain(self.artifacts.keys().copied())
            .map(Into::into)
            .collect()
    }

    pub(super) const fn artifacts(&self) -> &BTreeMap<&'static str, ArtifactBytes> {
        &self.artifacts
    }

    pub(super) fn artifact(&self, path: &str) -> Option<&ArtifactBytes> {
        self.artifacts.get(path)
    }

    pub(super) const fn hard_link(&self) -> Option<(&'static str, &'static str)> {
        self.hard_link
    }

    /// The committed single-format store that direct initialization leaves.
    fn initialized() -> Self {
        let directories = PROTOCOL_DIRECTORIES.into_iter().collect();
        let artifacts = [
            (WRITER_LOCK, ArtifactBytes::Empty),
            (READER_LOCK, ArtifactBytes::Empty),
            (FORMAT, ArtifactBytes::Marker),
        ]
        .into_iter()
        .collect();
        Self {
            directories,
            artifacts,
            hard_link: None,
        }
    }
}
