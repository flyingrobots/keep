//! Private-state corruption laws for staged publication.

use std::error::Error;
use std::io::Cursor;

use crate::{LayoutEntryLimit, PublishError, ReferenceStore, ReferenceStoreCapacity};

#[test]
fn committed_layout_with_a_missing_chunk_is_never_silently_repaired() -> Result<(), Box<dyn Error>>
{
    let source = b"committed state cannot be repaired by ordinary publication";
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let mut first_source = Cursor::new(source);
    let published = store
        .stage(&mut first_source, LayoutEntryLimit::MAXIMUM)?
        .commit(&mut store)?;
    let missing = store
        .layout(published.layout_id())
        .and_then(|layout| layout.entries().first())
        .ok_or("published layout has no chunk")?
        .chunk_id();
    store.chunks.remove(&missing);
    let mut second_source = Cursor::new(source);
    let staged = store.stage(&mut second_source, LayoutEntryLimit::MAXIMUM)?;

    let error = staged
        .commit(&mut store)
        .err()
        .ok_or("ordinary publication silently repaired committed state")?;

    assert!(matches!(
        error,
        PublishError::CommittedChunkMissing {
            layout,
            chunk
        } if layout == published.layout_id() && chunk == missing
    ));
    assert!(!store.chunks.contains_key(&missing));
    Ok(())
}

#[test]
fn committed_layout_without_its_blob_index_is_never_silently_repaired() -> Result<(), Box<dyn Error>>
{
    let source = b"layout index loss requires an explicit recovery operation";
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let mut first_source = Cursor::new(source);
    let published = store
        .stage(&mut first_source, LayoutEntryLimit::MAXIMUM)?
        .commit(&mut store)?;
    let removed = store
        .blob_layouts
        .get_mut(&published.target())
        .is_some_and(|layouts| layouts.remove(&published.layout_id()));
    assert!(removed);
    let mut second_source = Cursor::new(source);
    let staged = store.stage(&mut second_source, LayoutEntryLimit::MAXIMUM)?;

    let error = staged
        .commit(&mut store)
        .err()
        .ok_or("ordinary publication silently repaired the blob index")?;

    assert!(matches!(
        error,
        PublishError::CommittedLayoutIndexMissing { layout }
            if layout == published.layout_id()
    ));
    assert!(!store.contains_blob(published.target()));
    Ok(())
}

#[test]
fn committed_blob_index_without_its_layout_is_never_silently_repaired() -> Result<(), Box<dyn Error>>
{
    let source = b"layout loss requires an explicit recovery operation";
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let mut first_source = Cursor::new(source);
    let published = store
        .stage(&mut first_source, LayoutEntryLimit::MAXIMUM)?
        .commit(&mut store)?;
    let removed = store.layouts.remove(&published.layout_id());
    assert!(removed.is_some());
    let mut second_source = Cursor::new(source);
    let staged = store.stage(&mut second_source, LayoutEntryLimit::MAXIMUM)?;

    let error = staged
        .commit(&mut store)
        .err()
        .ok_or("ordinary publication silently repaired the missing layout")?;

    assert!(matches!(
        error,
        PublishError::CommittedLayoutMissing { layout }
            if layout == published.layout_id()
    ));
    assert!(store.contains_blob(published.target()));
    assert!(store.layout(published.layout_id()).is_none());
    Ok(())
}

#[test]
fn layout_indexed_under_the_wrong_blob_is_never_silently_extended() -> Result<(), Box<dyn Error>> {
    let source = b"a layout identity cannot migrate between target indexes";
    let wrong_target = crate::BlobId::hash_bytes(b"different target")?;
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let mut first_source = Cursor::new(source);
    let published = store
        .stage(&mut first_source, LayoutEntryLimit::MAXIMUM)?
        .commit(&mut store)?;
    store.layouts.remove(&published.layout_id());
    let removed = store
        .blob_layouts
        .get_mut(&published.target())
        .is_some_and(|layouts| layouts.remove(&published.layout_id()));
    assert!(removed);
    store
        .blob_layouts
        .entry(wrong_target)
        .or_default()
        .insert(published.layout_id());
    let mut second_source = Cursor::new(source);
    let staged = store.stage(&mut second_source, LayoutEntryLimit::MAXIMUM)?;

    let error = staged
        .commit(&mut store)
        .err()
        .ok_or("ordinary publication extended a wrong-target layout index")?;

    assert!(matches!(
        error,
        PublishError::CommittedLayoutMisindexed { layout, observed }
            if layout == published.layout_id() && observed == wrong_target
    ));
    assert!(!store.contains_blob(published.target()));
    assert!(store.contains_blob(wrong_target));
    assert!(store.layout(published.layout_id()).is_none());
    Ok(())
}
