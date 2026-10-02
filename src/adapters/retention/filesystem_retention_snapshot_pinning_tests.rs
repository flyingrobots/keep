//! This module owns catalog identity under ambient root replacement.

use std::error::Error;
use std::fs;

use super::{Source, collect_retention_view, pool_name};
use crate::adapters::filesystem_test_sandbox::TestDirectory;
use crate::adapters::retention::filesystem_retention_test_fixture::migrated_store;
use crate::adapters::retention::{ReaderAttemptLimit, RetentionViewSource};
use crate::adapters::{CatalogRestartByteLimit, CatalogRestartPolicy, SegmentReadPolicy};
use cap_fs_ext::DirExt;
use cap_std::fs::Dir;

// Size: medium (owned filesystem). Oracle: a pinned reader's catalog must
// match its original validated HEAD despite replacement of the ambient path.
// Delete only when pinned reader collection is removed or a stronger law subsumes it.
#[test]
fn replacing_the_ambient_root_preserves_the_pinned_catalog() -> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("reader-catalog-path-pinning")?;
    let moved = TestDirectory::create("reader-catalog-path-pinning-moved")?;
    let root = Dir::open_ambient_dir(sandbox.path(), cap_std::ambient_authority())?;
    let retention = root.open_dir_nofollow(pool_name::RETENTION)?;
    let manifests = retention.open_dir_nofollow(pool_name::MANIFESTS)?;
    let mut source = Source {
        root,
        retention,
        manifests,
        store_root: sandbox.path().to_path_buf(),
        policy: CatalogRestartPolicy::new(
            SegmentReadPolicy::MAXIMUM,
            CatalogRestartByteLimit::new(1_048_576)?,
        ),
    };
    let expected = source
        .coordinates()?
        .catalog
        .ok_or("original HEAD absent")?;

    fs::rename(sandbox.path(), moved.path().join("pinned"))?;
    fs::create_dir(sandbox.path())?;
    let result = collect_retention_view(&mut source, ReaderAttemptLimit::DEFAULT);
    assert!(
        result.is_ok(),
        "ambient replacement must not displace the pinned catalog"
    );
    let view = result?;

    assert_eq!(
        view.catalog.generation(),
        expected.0,
        "pinned catalog generation"
    );
    assert_eq!(
        view.catalog.catalog_digest(),
        expected.1,
        "pinned catalog digest"
    );
    Ok(())
}
