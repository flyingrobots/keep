//! Medium filesystem laws: compaction refuses changed lengths before allocation.
//! Oracle: admitted catalog/receipt length, unchanged HEAD and retained evidence.
//! Deterministic raw growth is fault injection, not supported writer isolation.
//! Delete when compaction is removed or stronger bounded-read laws subsume these.

use std::{error::Error, fs, io};

use super::filesystem::ReadTarget;
use super::test_fixture::{authority, mixed_store, plan};
use crate::adapters::{CatalogRestartError, physical_pool_name, publish_catalog_generation};

#[test]
fn retained_segment_growth_is_refused_before_materialization() -> Result<(), Box<dyn Error>> {
    growth_is_refused(Selection::Retained)
}

#[test]
fn sealed_stage_growth_is_refused_before_materialization() -> Result<(), Box<dyn Error>> {
    growth_is_refused(Selection::Staged)
}

enum Selection {
    Retained,
    Staged,
}

fn growth_is_refused(selection: Selection) -> Result<(), Box<dyn Error>> {
    let staged = matches!(selection, Selection::Staged);
    let sandbox = mixed_store(if staged {
        "bounded-stage"
    } else {
        "bounded-retained"
    })?;
    let plan = plan(sandbox.path())?;
    let retained = plan.retained().next().ok_or("missing retained segment")?;
    let target = if staged {
        ReadTarget::Staged
    } else {
        ReadTarget::Retained(retained)
    };
    let path = if staged {
        sandbox.path().join("staging/current.seg")
    } else {
        sandbox
            .path()
            .join("segments")
            .join(physical_pool_name::segment(retained))
    };
    let head = fs::read(sandbox.path().join("HEAD"))?;
    let mut writer = authority(sandbox.path())?;
    let length = 16_u64 * 1024 * 1024;
    let mut expected = None;
    let mut result = None;
    let allocation = allocation_counter::measure(|| {
        result = Some(writer.execute_before_read(
            &plan,
            &mut |publisher, expectation, segment, catalog, segments| {
                publish_catalog_generation(publisher, expectation, segment, catalog, segments)
            },
            &mut |observed| {
                if observed == target {
                    let file = fs::OpenOptions::new().write(true).open(&path)?;
                    expected = Some(file.metadata()?.len());
                    file.set_len(length)?;
                }
                Ok::<(), io::Error>(())
            },
        ));
    });
    let expected = expected.ok_or("growth checkpoint did not run")?;
    let error = result
        .ok_or("compaction did not run")?
        .err()
        .ok_or("growth admitted")?;
    assert!(
        allocation.bytes_max < length,
        "changed evidence must refuse before allocation: peak={} file={length}",
        allocation.bytes_max
    );
    let mut source: &(dyn Error + 'static) = &error;
    while source.downcast_ref::<CatalogRestartError>().is_none() {
        source = source.source().ok_or("typed length cause missing")?;
    }
    assert!(
        matches!(source.downcast_ref::<CatalogRestartError>(),
        Some(CatalogRestartError::Length { minimum, maximum, observed, .. })
            if *minimum == expected && *maximum == expected && *observed == length),
        "exact observed-length refusal: {error:?}"
    );
    assert_eq!(
        fs::metadata(&path)?.len(),
        length,
        "changed evidence retained"
    );
    assert_eq!(
        fs::read(sandbox.path().join("HEAD"))?,
        head,
        "publication unchanged"
    );
    Ok(())
}
