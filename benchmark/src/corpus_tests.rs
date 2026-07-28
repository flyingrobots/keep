//! Laws for the generated benchmark corpus.

use std::error::Error;

use keep::BlobId;

use super::BenchmarkCorpus;

#[test]
fn generated_corpus_is_bounded_reproducible_and_scenario_complete() -> Result<(), Box<dyn Error>> {
    let first = BenchmarkCorpus::generate()?;
    let second = BenchmarkCorpus::generate()?;

    assert_eq!(first.total_bytes(), second.total_bytes());
    assert!(first.total_bytes() <= BenchmarkCorpus::TOTAL_BYTE_LIMIT);
    assert_eq!(first.identities(), second.identities());
    assert_eq!(
        first.member_names(),
        [
            "large-text",
            "large-binary",
            "edit-base",
            "early-insertion",
            "early-deletion",
            "near-neighbor",
            "zero-dedup",
        ]
    );
    assert_eq!(first.large_text().len(), 1_048_576);
    assert_eq!(first.large_binary().len(), 1_048_576);
    assert_eq!(first.edit_base().len(), 2_097_152);
    assert_eq!(first.early_insertion().len(), 2_101_248);
    assert_eq!(first.early_deletion().len(), 2_093_056);
    assert_eq!(first.near_neighbor().len(), 2_097_152);
    assert_eq!(first.zero_dedup().len(), 2_097_152);
    assert_eq!(first.tiny_blobs().len(), 256);
    assert!(first.tiny_blobs().iter().all(|blob| blob.len() <= 256));

    for (identity, member) in first.identities().iter().zip(first.members()) {
        assert_eq!(*identity, BlobId::hash_bytes(member)?);
    }
    Ok(())
}

#[test]
fn generated_edits_have_exact_early_and_near_neighbor_semantics() -> Result<(), Box<dyn Error>> {
    let corpus = BenchmarkCorpus::generate()?;
    let base = corpus.edit_base();

    assert_eq!(
        corpus
            .early_insertion()
            .get(..4_096)
            .ok_or("insertion prefix")?,
        base.get(..4_096).ok_or("base insertion prefix")?
    );
    assert_eq!(
        corpus
            .early_insertion()
            .get(8_192..)
            .ok_or("insertion suffix")?,
        base.get(4_096..).ok_or("base insertion suffix")?
    );
    assert_eq!(
        corpus
            .early_deletion()
            .get(..4_096)
            .ok_or("deletion prefix")?,
        base.get(..4_096).ok_or("base deletion prefix")?
    );
    assert_eq!(
        corpus
            .early_deletion()
            .get(4_096..)
            .ok_or("deletion suffix")?,
        base.get(8_192..).ok_or("base deletion suffix")?
    );
    assert_eq!(
        corpus
            .near_neighbor()
            .get(..8_192)
            .ok_or("neighbor prefix")?,
        base.get(..8_192).ok_or("base neighbor prefix")?
    );
    assert_eq!(
        corpus
            .near_neighbor()
            .get(12_288..)
            .ok_or("neighbor suffix")?,
        base.get(12_288..).ok_or("base neighbor suffix")?
    );
    assert_ne!(
        corpus
            .near_neighbor()
            .get(8_192..12_288)
            .ok_or("neighbor change")?,
        base.get(8_192..12_288).ok_or("base neighbor change")?
    );
    Ok(())
}

#[test]
fn generated_members_match_frozen_logical_identity_witnesses() -> Result<(), Box<dyn Error>> {
    let corpus = BenchmarkCorpus::generate()?;
    let expected = [
        concat!(
            "keep:blob:v1:blake3-256:1048576:",
            "ce9299ba09e28adc20eb12881344636393b207af13857e1b1c0fd1c1b6179d62"
        ),
        concat!(
            "keep:blob:v1:blake3-256:1048576:",
            "9088c68986d0d4fac38cded718d2f6a0e7e62de07b6b88b9a3cd26e9ecd068e3"
        ),
        concat!(
            "keep:blob:v1:blake3-256:2097152:",
            "87a01fb55b86d2f45dd84a8971ba5ef9c24c3c7cd4999f767e9ddd96d437436b"
        ),
        concat!(
            "keep:blob:v1:blake3-256:2101248:",
            "ebb0bb45f64668c92509c6a781cc35e98136c00bb30b2ea68267ad8681b8a233"
        ),
        concat!(
            "keep:blob:v1:blake3-256:2093056:",
            "1a36ce7e2ffeea866576e1c3bb6145c4a09c0ce48e3a087aa55b8a26b273ac94"
        ),
        concat!(
            "keep:blob:v1:blake3-256:2097152:",
            "8dbc9def23a5e04e31cd2330b4a6ed031c92e89c650b82c5c2eb135a23d418a4"
        ),
        concat!(
            "keep:blob:v1:blake3-256:2097152:",
            "aa2f0d2e8f06da8fe0c34496287efa1f7cdedc949462bf003e3930fabf3a5eed"
        ),
    ];

    for ((identity, witness), name) in corpus
        .identities()
        .iter()
        .zip(expected)
        .zip(corpus.member_names())
    {
        assert_eq!(format!("{identity}"), witness, "{name}");
    }
    Ok(())
}
