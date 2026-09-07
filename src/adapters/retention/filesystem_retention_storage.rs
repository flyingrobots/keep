//! This module owns forward filesystem retention publication execution.

use std::io::{self, Read};

use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
use cap_std::fs::{Dir, OpenOptions};

use super::filesystem_retention_authority::FilesystemRetentionPublicationAuthority;
use super::filesystem_retention_pool_name as pool_name;
use super::filesystem_retention_stage::{FilesystemRetentionStage, invalid_data};
use super::{
    AdmittedRetentionRoot, CanonicalRetentionHead, CanonicalRetentionManifest,
    ChecksummedRetentionHead, RetentionNamespaceAdmission, RetentionPublicationPreparation,
    RetentionPublicationStorage, RetentionTransitionDisposition,
};
use crate::adapters::filesystem_catalog_artifact::synchronize_directory;

const HEAD_LENGTH: usize = 144;

impl RetentionPublicationStorage for FilesystemRetentionPublicationAuthority {
    fn verify_current(
        &mut self,
        preparation: &RetentionPublicationPreparation<'_>,
    ) -> io::Result<RetentionTransitionDisposition> {
        self.liveness_generation = Some(preparation.liveness_generation());
        require_no_retained_stage(&self.retention)?;
        let Some(head_bytes) = read_head(&self.retention)? else {
            return Ok(RetentionTransitionDisposition::Publish);
        };
        let head = ChecksummedRetentionHead::decode(&head_bytes)
            .map_err(|_source| invalid_data("current retention head refused admission"))?;
        if head.head().generation() == preparation.liveness_generation()
            && head.head().manifest_digest() == preparation.manifest_digest()
        {
            return Ok(RetentionTransitionDisposition::AlreadyCommitted);
        }
        Err(invalid_data(
            "current retention head is not the prepared predecessor; recovery is required",
        ))
    }

    fn write_root_stage(&mut self, root: &AdmittedRetentionRoot<'_>) -> io::Result<()> {
        self.root_stage = Some(FilesystemRetentionStage::create(
            &self.retention,
            pool_name::ROOT_STAGE,
            root.encoded(),
        )?);
        Ok(())
    }

    fn synchronize_root_stage(&mut self) -> io::Result<()> {
        self.root_stage()?.synchronize(&self.retention)
    }

    fn admit_root_namespace(
        &mut self,
        root: &AdmittedRetentionRoot<'_>,
    ) -> io::Result<RetentionNamespaceAdmission> {
        let name = pool_name::namespace(root.root().namespace().digest());
        let admission = match self.roots.create_dir(&name) {
            Ok(()) => RetentionNamespaceAdmission::Created,
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {
                RetentionNamespaceAdmission::Existing
            }
            Err(source) => return Err(source),
        };
        self.namespace = Some(self.roots.open_dir_nofollow(&name)?);
        Ok(admission)
    }

    fn synchronize_roots_after_namespace(&mut self) -> io::Result<()> {
        synchronize_directory(&self.roots)
    }

    fn link_root(&mut self, root: &AdmittedRetentionRoot<'_>) -> io::Result<()> {
        let name = pool_name::root(root.root().generation(), root.digest());
        let namespace = self.namespace()?;
        self.root_stage()?.link(&self.retention, namespace, &name)?;
        self.retained_root = Some(name);
        Ok(())
    }

    fn synchronize_root_namespace(&mut self, _root: &AdmittedRetentionRoot<'_>) -> io::Result<()> {
        synchronize_directory(self.namespace()?)
    }

    fn write_manifest_stage(&mut self, manifest: &CanonicalRetentionManifest) -> io::Result<()> {
        self.manifest_stage = Some(FilesystemRetentionStage::create(
            &self.retention,
            pool_name::MANIFEST_STAGE,
            manifest.encoded(),
        )?);
        Ok(())
    }

    fn synchronize_manifest_stage(&mut self) -> io::Result<()> {
        self.manifest_stage()?.synchronize(&self.retention)
    }

    fn link_manifest(&mut self, manifest: &CanonicalRetentionManifest) -> io::Result<()> {
        let name = self.manifest_name(manifest)?;
        self.manifest_stage()?
            .link(&self.retention, &self.manifests, &name)?;
        self.retained_manifest = Some(name);
        Ok(())
    }

    fn synchronize_manifest_pool(&mut self) -> io::Result<()> {
        synchronize_directory(&self.manifests)
    }

    fn write_head_stage(&mut self, head: &CanonicalRetentionHead) -> io::Result<()> {
        self.head_stage = Some(FilesystemRetentionStage::create(
            &self.retention,
            pool_name::HEAD_STAGE,
            head.encoded(),
        )?);
        Ok(())
    }

    fn synchronize_head_stage(&mut self) -> io::Result<()> {
        self.head_stage()?.synchronize(&self.retention)
    }

    fn replace_head(&mut self) -> io::Result<()> {
        self.take_head_stage()?
            .replace(&self.retention, pool_name::HEAD)
    }

    fn synchronize_retention_namespace(&mut self) -> io::Result<()> {
        synchronize_directory(&self.retention)
    }

    fn remove_root_stage(&mut self) -> io::Result<()> {
        let stage = self.take_root_stage()?;
        let name = self.retained_root_name()?;
        let namespace = self.namespace()?;
        stage.remove(&self.retention, namespace, &name)
    }

    fn remove_manifest_stage(&mut self) -> io::Result<()> {
        let stage = self.take_manifest_stage()?;
        let name = self.retained_manifest_name()?;
        stage.remove(&self.retention, &self.manifests, &name)
    }

    fn synchronize_cleanup(&mut self) -> io::Result<()> {
        synchronize_directory(&self.retention)
    }
}

fn require_no_retained_stage(retention: &Dir) -> io::Result<()> {
    for stage in [
        pool_name::ROOT_STAGE,
        pool_name::MANIFEST_STAGE,
        pool_name::HEAD_STAGE,
    ] {
        match retention.symlink_metadata(stage) {
            Err(source) if source.kind() == io::ErrorKind::NotFound => {}
            Ok(_) => {
                return Err(invalid_data(
                    "retained retention stage requires recovery before publication",
                ));
            }
            Err(source) => return Err(source),
        }
    }
    Ok(())
}

fn read_head(retention: &Dir) -> io::Result<Option<Vec<u8>>> {
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    let mut file = match retention.open_with(pool_name::HEAD, &options) {
        Ok(file) => file,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(source),
    };
    let expected_length = u64::try_from(HEAD_LENGTH)
        .map_err(|_source| invalid_data("retention head length exceeded u64"))?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() != expected_length {
        return Err(invalid_data("retention head kind or length disagreed"));
    }
    let mut bytes = vec![0_u8; HEAD_LENGTH];
    file.read_exact(&mut bytes)?;
    let mut trailing = [0_u8; 1];
    if file.read(&mut trailing)? != 0 {
        return Err(invalid_data("retention head carried trailing bytes"));
    }
    Ok(Some(bytes))
}

impl FilesystemRetentionPublicationAuthority {
    fn manifest_name(&self, manifest: &CanonicalRetentionManifest) -> io::Result<String> {
        let generation = self
            .liveness_generation
            .ok_or_else(|| invalid_data("selected liveness generation was not retained"))?;
        Ok(pool_name::manifest(generation, manifest.digest()))
    }

    fn retained_root_name(&self) -> io::Result<String> {
        self.retained_root
            .clone()
            .ok_or_else(|| invalid_data("retention root pool coordinate was not retained"))
    }

    fn retained_manifest_name(&self) -> io::Result<String> {
        self.retained_manifest
            .clone()
            .ok_or_else(|| invalid_data("retention manifest pool coordinate was not retained"))
    }
}
