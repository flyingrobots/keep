//! Laws for benchmark-only chunking profile comparisons.

use std::error::Error;

use keep::{ChunkId, FastCdc};

use crate::BenchmarkCorpus;

use super::ChunkingProfile;

const PROFILES: [ChunkingProfile; 5] = [
    ChunkingProfile::KeepFastCdcSmall,
    ChunkingProfile::KeepFastCdcRegistered,
    ChunkingProfile::KeepFastCdcLarge,
    ChunkingProfile::Fixed64KiB,
    ChunkingProfile::GitCasDefault,
];

#[test]
fn registered_benchmark_profile_reproduces_production_chunk_identities()
-> Result<(), Box<dyn Error>> {
    let corpus = BenchmarkCorpus::generate()?;
    for member in corpus.members() {
        let benchmark = ChunkingProfile::KeepFastCdcRegistered.partition(member)?;
        assert_eq!(benchmark.identities(), production_identities(member)?);
    }
    Ok(())
}

#[test]
fn compared_profiles_have_explicit_frozen_parameters_and_provenance() {
    assert_eq!(
        PROFILES.map(ChunkingProfile::name),
        [
            "keep-fastcdc-4-16-64",
            "keep-fastcdc-16-64-256",
            "keep-fastcdc-64-256-1024",
            "fixed-64",
            "git-cas-buzhash-64-256-1024",
        ]
    );
    assert_eq!(
        ChunkingProfile::GitCasDefault.provenance(),
        "git-cas@432c5d9effb12c9f66536f1386791bb4421f3cea"
    );
    assert_eq!(
        PROFILES.map(ChunkingProfile::bounds_kib),
        [
            (4, 16, 64),
            (16, 64, 256),
            (64, 256, 1_024),
            (64, 64, 64),
            (64, 256, 1_024),
        ]
    );
}

#[test]
fn every_profile_partitions_every_member_exactly_once() -> Result<(), Box<dyn Error>> {
    let corpus = BenchmarkCorpus::generate()?;
    for profile in PROFILES {
        for member in corpus.members() {
            let partition = profile.partition(member)?;
            assert_eq!(partition.logical_bytes(), member.len());
            assert_eq!(
                partition
                    .identities()
                    .iter()
                    .map(|identity| u64::from(identity.length().get()))
                    .try_fold(0_u64, u64::checked_add),
                u64::try_from(member.len()).ok()
            );
            assert_eq!(partition.ends().last().copied().unwrap_or(0), member.len());
            assert!(partition.ends().windows(2).all(|ends| {
                ends.first()
                    .zip(ends.last())
                    .is_some_and(|(first, last)| first < last)
            }));
            for (range, identity) in partition.ranges().zip(partition.identities()) {
                assert_eq!(
                    ChunkId::hash_bytes(member.get(range).ok_or("chunk range")?)?,
                    *identity
                );
            }
        }
    }
    Ok(())
}

#[test]
fn edit_reuse_counts_are_frozen_for_every_comparison_profile() -> Result<(), Box<dyn Error>> {
    let corpus = BenchmarkCorpus::generate()?;
    let base = corpus.edit_base();
    let variants = [
        corpus.early_insertion(),
        corpus.early_deletion(),
        corpus.near_neighbor(),
    ];
    let [inserted_source, deleted_source, neighbor_source] = variants;
    let mut observed = [(0_usize, 0_usize, 0_usize, 0_usize); 5];
    for (slot, profile) in observed.iter_mut().zip(PROFILES) {
        let base_partition = profile.partition(base)?;
        let inserted = profile.partition(inserted_source)?;
        let deleted = profile.partition(deleted_source)?;
        let neighbor = profile.partition(neighbor_source)?;
        *slot = (
            base_partition.unique_chunk_count()?,
            base_partition.reused_unique_chunk_count(&inserted)?,
            base_partition.reused_unique_chunk_count(&deleted)?,
            base_partition.reused_unique_chunk_count(&neighbor)?,
        );
    }

    assert_eq!(
        observed,
        [
            (86, 81, 85, 85),
            (25, 24, 24, 24),
            (5, 4, 4, 4),
            (32, 0, 0, 31),
            (5, 4, 4, 4),
        ]
    );
    Ok(())
}

fn production_identities(source: &[u8]) -> Result<Vec<ChunkId>, Box<dyn Error>> {
    let mut identities = Vec::new();
    let mut detector = FastCdc::new();
    detector.feed(source, |span| identities.push(span.id()))?;
    if let Some(span) = detector.finish()? {
        identities.push(span.id());
    }
    Ok(identities)
}
