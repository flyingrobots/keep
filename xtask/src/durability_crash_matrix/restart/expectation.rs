//! This module owns the independent expected crash-state model.

mod sequence;
mod steps;

use std::collections::{BTreeMap, BTreeSet};

use super::super::DurabilityCrashMatrixError;
use super::super::state::fixture::GoldenFixture;
use xtask::{DurabilityCrashCase, DurabilityCrashSequence};

pub(super) const WRITER_LOCK: &str = "writer.lock";
pub(super) const STAGING: &str = "staging";
pub(super) const SEGMENTS: &str = "segments";
pub(super) const CATALOGS: &str = "catalogs";
pub(super) const SEGMENT_STAGE: &str = "staging/current.seg";
pub(super) const CATALOG_STAGE: &str = "staging/current.cat";
pub(super) const NEXT_HEAD: &str = "head.next";
pub(super) const HEAD: &str = "HEAD";

pub(super) enum ArtifactBytes {
    Empty,
    Segment(usize),
    Catalog(usize),
    Head(usize),
}

impl ArtifactBytes {
    pub(super) const fn kind(&self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Segment(_) => "segment",
            Self::Catalog(_) => "catalog",
            Self::Head(_) => "head",
        }
    }

    pub(super) fn resolve<'a>(
        &self,
        segment: &'a GoldenFixture,
        catalog: &'a GoldenFixture,
        head: &'a GoldenFixture,
    ) -> Result<&'a [u8], DurabilityCrashMatrixError> {
        match self {
            Self::Empty => Ok(&[]),
            Self::Segment(end) => segment.prefix(*end),
            Self::Catalog(end) => catalog.prefix(*end),
            Self::Head(end) => head.prefix(*end),
        }
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

    fn initialized() -> Self {
        let directories = [STAGING, SEGMENTS, CATALOGS].into_iter().collect();
        let artifacts = std::iter::once((WRITER_LOCK, ArtifactBytes::Empty)).collect();
        Self {
            directories,
            artifacts,
            hard_link: None,
        }
    }
}
