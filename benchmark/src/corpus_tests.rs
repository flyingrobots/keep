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
    assert_eq!(first.edit_base().len(), 524_288);
    assert_eq!(first.early_insertion().len(), 528_384);
    assert_eq!(first.early_deletion().len(), 520_192);
    assert_eq!(first.near_neighbor().len(), 524_288);
    assert_eq!(first.zero_dedup().len(), 524_288);
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
            "ae1cd25c858113e4be9aaa5a896b039200a5efbd86121eac05c360cca86db40e"
        ),
        concat!(
            "keep:blob:v1:blake3-256:1048576:",
            "9088c68986d0d4fac38cded718d2f6a0e7e62de07b6b88b9a3cd26e9ecd068e3"
        ),
        concat!(
            "keep:blob:v1:blake3-256:524288:",
            "3e5191ad2325e530a58fd7268eef8b27cd20cbf9b40c85cca1620143cc81343a"
        ),
        concat!(
            "keep:blob:v1:blake3-256:528384:",
            "177a0811abb4bdc29e91038497dadac68d8bc67c606a3117f85d5dcf73f46a11"
        ),
        concat!(
            "keep:blob:v1:blake3-256:520192:",
            "c6ac8ef4aa685ccfcd358b74a227ecba6e09b399b094747d6e729d54fc539713"
        ),
        concat!(
            "keep:blob:v1:blake3-256:524288:",
            "47ddd8c58f0f9d1bb51b801779365aa4518768943895ceca71d6c22670e7ff5a"
        ),
        concat!(
            "keep:blob:v1:blake3-256:524288:",
            "44445b7fb1b1ad9f3326f0cab7ed278c0012c028f531d9b0f1c5deb9422a56a6"
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
