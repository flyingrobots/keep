//! This module owns retrying catalog admission when the collected heads move.

use std::error::Error;
use std::fs;
use std::io;

use super::{Source, collect_retention_view, pool_name};
use crate::adapters::retention::filesystem_retention_test_fixture::{
    CATALOG_NAME, ROOT_HEX, fixture, initial_preparation, open_authority,
};
use crate::adapters::retention::{
    ReaderAttemptLimit, ReaderFence, RetentionViewCoordinates, RetentionViewSource,
};
use crate::adapters::{CatalogRestartByteLimit, CatalogRestartPolicy, SegmentReadPolicy};
use crate::execute_retention_publication;
use cap_fs_ext::DirExt;
use cap_std::fs::Dir;

struct AfterLoad<F> {
    source: Source,
    action: Option<F>,
}

impl<F: FnOnce() -> io::Result<()>> RetentionViewSource for AfterLoad<F> {
    type View = <Source as RetentionViewSource>::View;

    fn coordinates(&mut self) -> io::Result<RetentionViewCoordinates> {
        self.source.coordinates()
    }

    fn load(&mut self) -> io::Result<Self::View> {
        let view = self.source.load()?;
        if let Some(action) = self.action.take() {
            action()?;
        }
        Ok(view)
    }
}

// Size: medium. Oracle: a head change discards even a failed catalog admission;
// only a stable pair may select a catalog result, within the attempt budget.
// Delete when collected reader results are removed or a stronger law subsumes this schedule.
#[test]
fn a_head_change_discards_the_superseded_catalog_refusal() -> Result<(), Box<dyn Error>> {
    let (sandbox, mut authority) = open_authority("reader-moving-catalog-error")?;
    let root = Dir::open_ambient_dir(sandbox.path(), cap_std::ambient_authority())?;
    let _fence = ReaderFence::acquire(&root)?;
    let retention = root.open_dir_nofollow(pool_name::RETENTION)?;
    let manifests = retention.open_dir_nofollow(pool_name::MANIFESTS)?;
    let mut source = Source {
        root,
        retention,
        manifests,
        policy: CatalogRestartPolicy::new(
            SegmentReadPolicy::MAXIMUM,
            CatalogRestartByteLimit::new(1_048_576)?,
        ),
    };
    let expected = source.coordinates()?.catalog.ok_or("catalog HEAD absent")?;
    let path = sandbox.path().join("catalogs").join(CATALOG_NAME);
    let bytes = fs::read(&path)?;
    fs::remove_file(&path)?;
    let root_bytes = fixture(ROOT_HEX)?;
    let preparation = initial_preparation(&root_bytes)?;
    let expected_retention = preparation.liveness_generation();
    let mut scheduled = AfterLoad {
        source,
        action: Some(|| {
            fs::write(&path, &bytes)?;
            let _published = execute_retention_publication(&mut authority, &preparation)
                .map_err(io::Error::other)?;
            Ok(())
        }),
    };

    let collected = collect_retention_view(&mut scheduled, ReaderAttemptLimit::DEFAULT);
    assert!(
        matches!(&collected, Ok(Ok(_))),
        "moved heads must discard the superseded catalog refusal"
    );
    let view = collected??;

    assert_eq!(
        view.catalog.generation(),
        expected.0,
        "accepted catalog generation"
    );
    assert_eq!(
        view.catalog.catalog_digest(),
        expected.1,
        "accepted catalog digest"
    );
    assert_eq!(
        view.retention
            .as_ref()
            .map(|state| state.head().generation()),
        Some(expected_retention),
        "the returned reader view must bind the newly published retention head"
    );
    Ok(())
}
