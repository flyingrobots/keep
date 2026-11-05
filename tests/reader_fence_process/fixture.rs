//! An initialized golden store for public reader-fence process laws.

use super::populated_store::{self, FirstCatalogRecords, StoreAdmission};
use super::sandbox::TestDirectory;
use keep::{FilesystemCatalogPublisher, FilesystemStoreAdmission};
use std::error::Error;

/// The shared builder's route outside the production platform profile.
const UNCHECKED: StoreAdmission = StoreAdmission {
    initialize: |root| {
        Ok(
            FilesystemStoreAdmission::initialize_unchecked_for_repository_tasks(root)?
                .into_admission(),
        )
    },
    reopen: |root| Ok(FilesystemStoreAdmission::reopen_unchecked_for_repository_tasks(root)?),
};

/// Creates the one-zero bundle store and keeps its writer authority.
///
/// Initialization and the first publication run through the public writer.
/// Writer authority stays continuous while the caller arranges collection: a
/// concurrent spawn may inherit lock descriptions until exec, so dropping and
/// immediately reacquiring cannot assume all holders have disappeared.
pub(super) fn initialized(
    name: &str,
) -> Result<(TestDirectory, FilesystemCatalogPublisher), Box<dyn Error>> {
    let sandbox = TestDirectory::create(name)?;
    let (_created, writer) = populated_store::create_holding_authority(
        sandbox.path(),
        UNCHECKED,
        &[&[0]],
        FirstCatalogRecords::Complete,
    )?;
    populated_store::require_first_catalog(
        sandbox.path(),
        &decode(include_str!(
            "../../conformance/segment-store/v1/one-zero-bundle-segment.hex"
        ))?,
        &decode(include_str!(
            "../../conformance/segment-store/v1/one-zero-bundle-catalog.hex"
        ))?,
        &decode(include_str!(
            "../../conformance/segment-store/v1/one-zero-bundle-head.hex"
        ))?,
    )?;
    Ok((sandbox, writer))
}

fn decode(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    hex.trim_end()
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| Ok(u8::from_str_radix(std::str::from_utf8(pair)?, 16)?))
        .collect()
}
