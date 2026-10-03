//! Specified successor refusal coordinates over every generated record subset.

use super::{
    ResultOf, policy,
    record_inputs::{self, Packing},
};
use crate::support::require_error;
use keep::{
    AdmittedSegment, CanonicalCatalog, CanonicalPublicationHead, CatalogGeneration,
    CatalogSnapshotError, CatalogTransitionError, ChecksummedPublicationHead,
};

// Size: small. Oracle: specified exact successor arithmetic and digest equality,
// independent of the implementation's transition validator.
#[test]
fn generated_catalogs_refuse_stale_and_skipped_generations_exactly() -> ResultOf<()> {
    let inputs = record_inputs::inputs()?;
    for mask in 0..16 {
        let bytes =
            record_inputs::encode(&record_inputs::select(&inputs, mask)?, Packing::Bundled)?;
        let segments = bytes
            .iter()
            .map(|bytes| AdmittedSegment::decode(bytes, policy()))
            .collect::<Result<Vec<_>, _>>()?;
        let first = CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &segments)?;
        let current = CanonicalCatalog::from_segments(
            CatalogGeneration::new(2)?,
            Some(first.checksummed().digest()),
            &segments,
        )?;
        for observed in [2, 4] {
            let candidate = CanonicalCatalog::from_segments(
                CatalogGeneration::new(observed)?,
                Some(current.checksummed().digest()),
                &segments,
            )?;
            let error = require_error(
                current
                    .checksummed()
                    .admit(&segments)?
                    .validate_successor(candidate.checksummed().admit(&segments)?),
                "stale/skipped generation admitted",
            )?;
            assert_eq!(
                error,
                CatalogTransitionError::Generation {
                    expected: CatalogGeneration::new(3)?,
                    observed: CatalogGeneration::new(observed)?
                },
                "mask={mask} candidate={observed}"
            );
        }
    }
    Ok(())
}

#[test]
fn generated_catalogs_refuse_wrong_predecessors_exactly() -> ResultOf<()> {
    let inputs = record_inputs::inputs()?;
    for mask in 0..16 {
        let bytes = record_inputs::encode(
            &record_inputs::select(&inputs, mask)?,
            Packing::SeparateReversed,
        )?;
        let segments = bytes
            .iter()
            .map(|bytes| AdmittedSegment::decode(bytes, policy()))
            .collect::<Result<Vec<_>, _>>()?;
        let first = CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &segments)?;
        let wrong = first.checksummed().digest();
        let current =
            CanonicalCatalog::from_segments(CatalogGeneration::new(2)?, Some(wrong), &segments)?;
        let candidate =
            CanonicalCatalog::from_segments(CatalogGeneration::new(3)?, Some(wrong), &segments)?;
        let error = require_error(
            current
                .checksummed()
                .admit(&segments)?
                .validate_successor(candidate.checksummed().admit(&segments)?),
            "wrong predecessor admitted",
        )?;
        assert_eq!(
            error,
            CatalogTransitionError::Predecessor {
                expected: current.checksummed().digest(),
                observed: Some(wrong)
            },
            "mask={mask}"
        );
    }
    Ok(())
}

#[test]
fn generated_heads_cannot_bind_a_different_generation() -> ResultOf<()> {
    let inputs = record_inputs::inputs()?;
    for mask in 0..16 {
        let bytes =
            record_inputs::encode(&record_inputs::select(&inputs, mask)?, Packing::Bundled)?;
        let segments = bytes
            .iter()
            .map(|bytes| AdmittedSegment::decode(bytes, policy()))
            .collect::<Result<Vec<_>, _>>()?;
        let first = CanonicalCatalog::from_segments(CatalogGeneration::new(1)?, None, &segments)?;
        let later = CanonicalCatalog::from_segments(
            CatalogGeneration::new(2)?,
            Some(first.checksummed().digest()),
            &segments,
        )?;
        let head = CanonicalPublicationHead::for_catalog(first.checksummed());
        let error = require_error(
            ChecksummedPublicationHead::decode(head.encoded())?
                .admit(later.checksummed().admit(&segments)?),
            "mixed-generation snapshot admitted",
        )?;
        assert_eq!(
            error,
            CatalogSnapshotError::Generation {
                expected: CatalogGeneration::new(1)?,
                observed: CatalogGeneration::new(2)?
            },
            "mask={mask}"
        );
    }
    Ok(())
}
