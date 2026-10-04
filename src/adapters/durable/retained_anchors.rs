//! This module owns bounded selected-root admission and canonical anchor lookup.

use super::DurableStoreError;
use crate::adapters::{
    AdmittedRetentionRoot, FilesystemRetentionSnapshot, verify_retention_closure,
};
use crate::{BlobId, LayoutId, RetentionNamespaceDigest};

pub(super) fn verify(view: &FilesystemRetentionSnapshot) -> Result<(), DurableStoreError> {
    let Some(manifest) = view.manifest() else {
        return Ok(());
    };
    let catalog = view
        .catalog()
        .snapshot()
        .map_err(|source| DurableStoreError::Catalog(Box::new(source)))?;
    for entry in manifest.entries() {
        let namespace = entry.namespace();
        let bytes = root_bytes(view, namespace)?;
        let root = AdmittedRetentionRoot::decode(&bytes)
            .map_err(|source| DurableStoreError::RootDecode { namespace, source })?;
        let _verified = verify_retention_closure(root.root(), &catalog).map_err(|source| {
            DurableStoreError::Closure {
                namespace,
                source: Box::new(source),
            }
        })?;
    }
    Ok(())
}

pub(super) fn first_layout(
    view: &FilesystemRetentionSnapshot,
    target: BlobId,
) -> Result<Option<LayoutId>, DurableStoreError> {
    let Some(manifest) = view.manifest() else {
        return Ok(None);
    };
    let mut selected: Option<LayoutId> = None;
    for entry in manifest.entries() {
        let namespace = entry.namespace();
        let bytes = root_bytes(view, namespace)?;
        let root = AdmittedRetentionRoot::decode(&bytes)
            .map_err(|source| DurableStoreError::RootDecode { namespace, source })?;
        for anchor in root.root().anchors() {
            if anchor.blob_id() == target {
                selected = Some(selected.map_or_else(
                    || anchor.layout_id(),
                    |previous| previous.min(anchor.layout_id()),
                ));
            }
        }
    }
    Ok(selected)
}

fn root_bytes(
    view: &FilesystemRetentionSnapshot,
    namespace: RetentionNamespaceDigest,
) -> Result<Box<[u8]>, DurableStoreError> {
    view.retained_root(namespace)
        .map_err(|source| DurableStoreError::Snapshot(Box::new(source)))?
        .ok_or(DurableStoreError::RootMissing { namespace })
}
