//! A migrated golden store for public reader-fence process laws.

use super::sandbox::TestDirectory;
use keep::{
    FilesystemStoreMigrationAuthority, RepositoryInitializationStorage, SegmentReadPolicy,
    execute_store_migration, initialize_store,
};
use std::{error::Error, fs};

pub(super) fn migrated(
    name: &str,
) -> Result<(TestDirectory, FilesystemStoreMigrationAuthority), Box<dyn Error>> {
    let sandbox = TestDirectory::create(name)?;
    let mut storage = RepositoryInitializationStorage::admit_unchecked(sandbox.path())?;
    let _initialized = initialize_store(&mut storage)?;
    for (name, hex) in [
        (
            "segments/221f6745cd8a5221c9a87c3707593608479282b54a4a74d0e753fd76f70e8db2.seg",
            include_str!("../../conformance/segment-store/v1/one-zero-bundle-segment.hex"),
        ),
        (
            "catalogs/0000000000000001-0b7cad1b6de663d34beacbc214db7497f2e36ab6b08dfbd5febbc8d06a418811.cat",
            include_str!("../../conformance/segment-store/v1/one-zero-bundle-catalog.hex"),
        ),
        (
            "HEAD",
            include_str!("../../conformance/segment-store/v1/one-zero-bundle-head.hex"),
        ),
    ] {
        fs::write(sandbox.path().join(name), decode(hex)?)?;
    }
    let mut migration = FilesystemStoreMigrationAuthority::open_unchecked_for_repository_tasks(
        storage.into_writer_lock()?,
        SegmentReadPolicy::MAXIMUM,
    )?;
    let intent = migration.observe_intent()?;
    let _receipt = execute_store_migration(&mut migration, &intent)?;
    // Keep writer authority continuous while the caller arranges collection.
    // A concurrent spawn may inherit lock descriptions until exec, so dropping
    // and immediately reacquiring cannot assume all holders have disappeared.
    Ok((sandbox, migration))
}

fn decode(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    hex.trim_end()
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| Ok(u8::from_str_radix(std::str::from_utf8(pair)?, 16)?))
        .collect()
}
