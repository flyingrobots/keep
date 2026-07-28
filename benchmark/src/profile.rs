//! Named benchmark-only chunking profiles and exact partitions.

use crate::ProfileError;

/// Chunking profiles compared by the streaming CAS benchmark.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChunkingProfile {
    /// Benchmark-only Keep `FastCDC` candidate with 4/16/64 KiB bounds.
    KeepFastCdcSmall,
    /// Keep's registered production 16/64/256 KiB profile.
    KeepFastCdcRegistered,
    /// Benchmark-only Keep `FastCDC` candidate with 64/256/1,024 KiB bounds.
    KeepFastCdcLarge,
    /// Fixed 64 KiB shift-sensitive baseline.
    Fixed64KiB,
    /// Pinned git-cas 64/256/1,024 KiB Buzhash defaults.
    GitCasDefault,
}

/// Exact ordered chunk identities and exclusive end coordinates.
pub struct ChunkPartition {
    pub(super) logical_bytes: usize,
    pub(super) ends: Vec<usize>,
    pub(super) identities: Vec<keep::ChunkId>,
}

impl ChunkingProfile {
    /// Returns the stable benchmark profile name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::KeepFastCdcSmall => "keep-fastcdc-4-16-64",
            Self::KeepFastCdcRegistered => "keep-fastcdc-16-64-256",
            Self::KeepFastCdcLarge => "keep-fastcdc-64-256-1024",
            Self::Fixed64KiB => "fixed-64",
            Self::GitCasDefault => "git-cas-buzhash-64-256-1024",
        }
    }

    /// Returns the source coordinate for this comparison law.
    #[must_use]
    pub const fn provenance(self) -> &'static str {
        match self {
            Self::KeepFastCdcSmall | Self::KeepFastCdcRegistered | Self::KeepFastCdcLarge => {
                "keep.fastcdc-gear64/v1"
            }
            Self::Fixed64KiB => "benchmark.fixed-size/v1",
            Self::GitCasDefault => "git-cas@432c5d9effb12c9f66536f1386791bb4421f3cea",
        }
    }

    /// Returns minimum, target, and maximum chunk sizes in KiB.
    #[must_use]
    pub const fn bounds_kib(self) -> (u32, u32, u32) {
        match self {
            Self::KeepFastCdcSmall => (4, 16, 64),
            Self::KeepFastCdcRegistered => (16, 64, 256),
            Self::KeepFastCdcLarge | Self::GitCasDefault => (64, 256, 1_024),
            Self::Fixed64KiB => (64, 64, 64),
        }
    }

    /// Partitions exact bytes under this benchmark profile.
    ///
    /// The two candidate Keep profiles are benchmark-only and are not admitted
    /// storage profiles. This operation does not mutate Keep's registry.
    ///
    /// # Errors
    ///
    /// Returns [`ProfileError`] for bounded allocation, coordinate, table,
    /// or chunk-identity failures.
    pub fn partition(self, source: &[u8]) -> Result<ChunkPartition, ProfileError> {
        let ends = match self {
            Self::KeepFastCdcSmall | Self::KeepFastCdcRegistered | Self::KeepFastCdcLarge => {
                crate::keep_profile::partition(self, source)?
            }
            Self::Fixed64KiB => crate::fixed_profile::partition(source)?,
            Self::GitCasDefault => crate::git_cas_profile::partition(source)?,
        };
        ChunkPartition::from_ends(source, ends)
    }
}

#[cfg(test)]
#[path = "profile_tests.rs"]
mod tests;
