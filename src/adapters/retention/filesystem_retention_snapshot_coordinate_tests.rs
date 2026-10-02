//! This module owns reader refusal of a changed catalog-length coordinate.

use std::error::Error;
use std::fs;
use std::io;
use std::num::NonZeroU32;

use super::{Source, collect_retention_view, pool_name};
use crate::adapters::retention::filesystem_retention_test_fixture::migrated_store;
use crate::adapters::retention::{
    ReaderAttemptLimit, ReaderFence, RetentionViewCoordinates, RetentionViewError,
    RetentionViewSource,
};
use crate::adapters::{
    CatalogRestartByteLimit, CatalogRestartPolicy, ChecksummedPublicationHead, SegmentReadPolicy,
    publication_head_decoder,
};
use cap_fs_ext::DirExt;
use cap_std::fs::Dir;

struct RewriteHead<F> {
    source: Source,
    rewrite: F,
}

impl<F: FnMut() -> io::Result<()>> RetentionViewSource for RewriteHead<F> {
    type View = <Source as RetentionViewSource>::View;

    fn coordinates(&mut self) -> io::Result<RetentionViewCoordinates> {
        self.source.coordinates()
    }

    fn load(&mut self) -> io::Result<Self::View> {
        let view = self.source.load()?;
        (self.rewrite)()?;
        Ok(view)
    }
}

// Size: medium. Oracle: every validated HEAD coordinate must agree across collection.
// Delete when double collection is removed or a stronger filesystem law subsumes this schedule.
#[test]
fn a_changed_catalog_length_refuses_the_loaded_reader_view() -> Result<(), Box<dyn Error>> {
    let sandbox = migrated_store("reader-changed-catalog-length")?;
    let root = Dir::open_ambient_dir(sandbox.path(), cap_std::ambient_authority())?;
    let _fence = ReaderFence::acquire(&root)?;
    let retention = root.open_dir_nofollow(pool_name::RETENTION)?;
    let manifests = retention.open_dir_nofollow(pool_name::MANIFESTS)?;
    let path = sandbox.path().join("HEAD");
    let mut head = fs::read(&path)?;
    let decoded = ChecksummedPublicationHead::decode(&head)?;
    let changed_length = decoded
        .catalog_length()
        .get()
        .checked_add(160)
        .ok_or("length overflow")?;
    head.get_mut(32..40)
        .ok_or("HEAD lacks catalog length")?
        .copy_from_slice(&changed_length.to_be_bytes());
    let (covered, checksum) = head.split_at_mut(publication_head_decoder::CHECKSUM_INPUT_LENGTH);
    checksum.copy_from_slice(&publication_head_decoder::checksum(covered));
    let _admitted_changed_head = ChecksummedPublicationHead::decode(&head)?;
    let source = Source {
        root,
        retention,
        manifests,
        policy: CatalogRestartPolicy::new(
            SegmentReadPolicy::MAXIMUM,
            CatalogRestartByteLimit::new(1_048_576)?,
        ),
    };
    let mut scheduled = RewriteHead {
        source,
        rewrite: || fs::write(&path, &head),
    };
    let limit = ReaderAttemptLimit::new(NonZeroU32::new(1).ok_or("zero attempts")?);

    let result = collect_retention_view(&mut scheduled, limit);

    assert!(
        matches!(
            result,
            Err(RetentionViewError::AttemptsExhausted { attempts: 1 })
        ),
        "a changed catalog length must refuse the previously loaded view"
    );
    Ok(())
}
