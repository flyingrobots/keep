//! Public storage-profile identity and admission laws.

use keep::{
    RegisteredStorageProfile, StorageProfileAdmissionError, StorageProfileId,
    StorageProfileIdParseError,
};

const FAST_CDC_64K_V1: &str = concat!(
    "keep:storage-profile:v1:blake3-256:",
    "aafa6f05bdc8894306abd41ec6f2b3b76cde995f2598fa3fd547d81fbe1a34eb"
);

#[test]
fn registered_profile_identity_matches_the_frozen_coordinate()
-> Result<(), StorageProfileIdParseError> {
    let parsed = FAST_CDC_64K_V1.parse::<StorageProfileId>()?;
    let registered = RegisteredStorageProfile::FAST_CDC_64K_V1;

    assert_eq!(parsed, registered.id());
    assert_eq!(parsed.to_string(), FAST_CDC_64K_V1);
    assert_eq!(registered.minimum_chunk_length().get(), 16_384);
    assert_eq!(registered.maximum_chunk_length().get(), 262_144);
    Ok(())
}

#[test]
fn canonical_but_unregistered_profile_identity_is_refused_at_admission()
-> Result<(), StorageProfileIdParseError> {
    let unknown = concat!(
        "keep:storage-profile:v1:blake3-256:",
        "00fa6f05bdc8894306abd41ec6f2b3b76cde995f2598fa3fd547d81fbe1a34eb"
    )
    .parse::<StorageProfileId>()?;

    assert_eq!(
        RegisteredStorageProfile::admit(unknown),
        Err(StorageProfileAdmissionError::Unsupported { observed: unknown })
    );
    Ok(())
}

#[test]
fn storage_profile_identity_refuses_noncanonical_digest_text() {
    let uppercase = concat!(
        "keep:storage-profile:v1:blake3-256:",
        "AAfa6f05bdc8894306abd41ec6f2b3b76cde995f2598fa3fd547d81fbe1a34eb"
    );

    assert_eq!(
        uppercase.parse::<StorageProfileId>(),
        Err(StorageProfileIdParseError::NonCanonicalDigestCase)
    );
}
