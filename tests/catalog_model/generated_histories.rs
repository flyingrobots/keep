//! Generated finite catalog histories checked against pre-encoding input maps.

#[path = "record_inputs.rs"]
mod record_inputs;
#[path = "transition_refusals.rs"]
mod transition_refusals;

use keep::{
    AdmittedSegment, CanonicalCatalog, CanonicalPublicationHead, CatalogGeneration,
    CatalogSnapshot, ChecksummedPublicationHead, LayoutEntryLimit, SegmentReadPolicy,
    SegmentRecordLimit,
};
use record_inputs::{Packing, RecordMap};
use std::error::Error;

type ResultOf<T> = Result<T, Box<dyn Error>>;

// Size: small (memory only). Oracle: the input maps, chosen before any writer,
// decoder or catalog executes. Exhaustive bounded generator, no ambient seed.
#[test]
fn generated_histories_preserve_exact_membership_in_pinned_snapshots() -> ResultOf<()> {
    let inputs = record_inputs::inputs()?;
    for packing in [Packing::Bundled, Packing::SeparateReversed] {
        check_packing(&inputs, packing)?;
    }
    Ok(())
}

fn check_packing(inputs: &RecordMap, packing: Packing) -> ResultOf<()> {
    for first in 0..16_u8 {
        for second in 0..16_u8 {
            for third in 0..16_u8 {
                check_history(inputs, [first, second, third], packing)?;
            }
        }
    }
    Ok(())
}

fn check_history(inputs: &RecordMap, history: [u8; 3], packing: Packing) -> ResultOf<()> {
    let models = history
        .map(|mask| record_inputs::select(inputs, mask))
        .into_iter()
        .collect::<ResultOf<Vec<_>>>()?;
    let encoded = models
        .iter()
        .map(|model| record_inputs::encode(model, packing))
        .collect::<ResultOf<Vec<_>>>()?;
    let segments = encoded
        .iter()
        .map(|generation| {
            generation
                .iter()
                .map(|bytes| AdmittedSegment::decode(bytes, policy()))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;
    let catalogs = build_catalogs(&segments)?;
    let heads: Vec<_> = catalogs
        .iter()
        .map(|catalog| CanonicalPublicationHead::for_catalog(catalog.checksummed()))
        .collect();
    let snapshots = catalogs
        .iter()
        .zip(&segments)
        .zip(&heads)
        .map(|((catalog, records), head)| {
            ChecksummedPublicationHead::decode(head.encoded())?
                .admit(catalog.checksummed().admit(records)?)
                .map_err(Into::into)
        })
        .collect::<ResultOf<Vec<_>>>()?;
    for (index, (snapshot, model)) in snapshots.iter().zip(&models).enumerate() {
        assert_eq!(
            snapshot.generation().get(),
            u64::try_from(index)?
                .checked_add(1)
                .ok_or("generation overflow")?,
            "pinned generation history={history:?} packing={packing:?}"
        );
        check_membership(snapshot, model, inputs, history, packing)?;
    }
    check_successors(&catalogs, &segments, history, packing)
}

fn build_catalogs(segments: &[Vec<AdmittedSegment<'_>>]) -> ResultOf<Vec<CanonicalCatalog>> {
    let mut catalogs = Vec::new();
    let mut predecessor = None;
    for (index, records) in segments.iter().enumerate() {
        let generation = CatalogGeneration::new(
            u64::try_from(index)?
                .checked_add(1)
                .ok_or("generation overflow")?,
        )?;
        let catalog = CanonicalCatalog::from_segments(generation, predecessor, records)?;
        predecessor = Some(catalog.checksummed().digest());
        catalogs.push(catalog);
    }
    Ok(catalogs)
}

fn check_successors(
    catalogs: &[CanonicalCatalog],
    segments: &[Vec<AdmittedSegment<'_>>],
    history: [u8; 3],
    packing: Packing,
) -> ResultOf<()> {
    for (index, (current, next)) in catalogs.windows(2).zip(segments.windows(2)).enumerate() {
        let [before, after] = current else {
            return Err("missing catalog pair".into());
        };
        let [old_records, new_records] = next else {
            return Err("missing segment pair".into());
        };
        let successor = before
            .checksummed()
            .admit(old_records)?
            .validate_successor(after.checksummed().admit(new_records)?)?;
        assert_eq!(
            successor.generation(),
            CatalogGeneration::new(
                u64::try_from(index)?
                    .checked_add(2)
                    .ok_or("generation overflow")?
            )?,
            "successor history={history:?} packing={packing:?}"
        );
    }
    Ok(())
}

fn check_membership(
    snapshot: &CatalogSnapshot<'_, '_, '_>,
    model: &RecordMap,
    inputs: &RecordMap,
    history: [u8; 3],
    packing: Packing,
) -> ResultOf<()> {
    assert_eq!(
        snapshot.record_count(),
        u64::try_from(model.len())?,
        "membership count history={history:?} packing={packing:?}"
    );
    for identity in inputs.keys() {
        let observed = snapshot.record(*identity);
        assert_eq!(
            observed.map(keep::AdmittedSegmentRecord::payload),
            model.get(identity).map(Vec::as_slice),
            "exact payload/absence history={history:?} packing={packing:?} generation={} identity={identity:?}",
            snapshot.generation().get()
        );
    }
    Ok(())
}

const fn policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}
