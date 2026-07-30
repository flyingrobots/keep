//! This module owns bounded catalog admission for migration pool inventory.

use crate::adapters::{
    AdmittedSegment, CatalogAdmissionError, ChecksummedCatalog, SegmentDigest, SegmentReadPolicy,
};
use crate::{CatalogDigest, CatalogGeneration, CatalogLength};

use super::{migration_catalog_plan, migration_catalog_records};

pub(super) struct AdmittedMigrationCatalog<'a> {
    catalog: ChecksummedCatalog<'a>,
}

impl AdmittedMigrationCatalog<'_> {
    pub(super) const fn generation(&self) -> CatalogGeneration {
        self.catalog.generation()
    }

    pub(super) const fn length(&self) -> CatalogLength {
        self.catalog.length()
    }

    pub(super) const fn digest(&self) -> CatalogDigest {
        self.catalog.digest()
    }
}

pub(super) enum MigrationCatalogAdmissionError<E> {
    Catalog(Box<CatalogAdmissionError>),
    SegmentSource {
        digest: SegmentDigest,
        source: E,
    },
    SegmentCoordinate {
        expected: SegmentDigest,
        observed: SegmentDigest,
    },
}

pub(super) enum MigrationSegmentLoadError<E> {
    Missing,
    Source(E),
}

pub(super) fn admit<E, F>(
    catalog: ChecksummedCatalog<'_>,
    policy: SegmentReadPolicy,
    mut load: F,
) -> Result<AdmittedMigrationCatalog<'_>, MigrationCatalogAdmissionError<E>>
where
    F: FnMut(SegmentDigest) -> Result<Vec<u8>, MigrationSegmentLoadError<E>>,
{
    let mut plan = migration_catalog_plan::plan(catalog)?;
    plan.sort_unstable_by_key(|entry| entry.physical_order());
    for entries in
        plan.chunk_by(|first, second| first.entry.segment_digest() == second.entry.segment_digest())
    {
        admit_segment(entries, policy, &mut load)?;
    }
    Ok(AdmittedMigrationCatalog { catalog })
}

fn admit_segment<E, F>(
    entries: &[migration_catalog_plan::PlannedEntry],
    policy: SegmentReadPolicy,
    load: &mut F,
) -> Result<(), MigrationCatalogAdmissionError<E>>
where
    F: FnMut(SegmentDigest) -> Result<Vec<u8>, MigrationSegmentLoadError<E>>,
{
    let Some(first) = entries.first() else {
        return Ok(());
    };
    let expected = first.entry.segment_digest();
    let encoded = match load(expected) {
        Ok(encoded) => encoded,
        Err(MigrationSegmentLoadError::Missing) => {
            return Err(catalog_error(CatalogAdmissionError::MissingSegment {
                digest: expected,
            }));
        }
        Err(MigrationSegmentLoadError::Source(source)) => {
            return Err(MigrationCatalogAdmissionError::SegmentSource {
                digest: expected,
                source,
            });
        }
    };
    let segment = AdmittedSegment::decode(&encoded, policy).map_err(|source| {
        catalog_error(CatalogAdmissionError::Segment {
            digest: expected,
            source: Box::new(source),
        })
    })?;
    if segment.digest() != expected {
        return Err(MigrationCatalogAdmissionError::SegmentCoordinate {
            expected,
            observed: segment.digest(),
        });
    }
    migration_catalog_records::validate(entries, &segment).map_err(catalog_error)
}

pub(super) fn catalog_error<E>(source: CatalogAdmissionError) -> MigrationCatalogAdmissionError<E> {
    MigrationCatalogAdmissionError::Catalog(Box::new(source))
}
