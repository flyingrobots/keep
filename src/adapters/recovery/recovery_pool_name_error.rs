//! This module owns immutable-pool name parsing failures.

use std::error::Error;
use std::fmt;

/// Why an immutable-pool entry name is noncanonical.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryPoolNameError {
    /// The complete name has the wrong width.
    WrongLength {
        /// Required byte width.
        expected: usize,
        /// Observed byte width.
        observed: usize,
    },
    /// The artifact suffix is not exact.
    WrongSuffix,
    /// The catalog generation separator is not `-`.
    WrongSeparator,
    /// The catalog generation contains uppercase hexadecimal.
    UppercaseGeneration,
    /// The catalog generation contains a non-hexadecimal byte.
    InvalidGenerationAlphabet,
    /// Catalog generation zero is forbidden.
    ZeroGeneration,
    /// The digest has the wrong width.
    DigestLength {
        /// Required byte width.
        expected: usize,
        /// Observed byte width.
        observed: usize,
    },
    /// The digest contains uppercase hexadecimal.
    UppercaseDigest,
    /// The digest contains a non-hexadecimal byte.
    InvalidDigestAlphabet,
}

impl fmt::Display for RecoveryPoolNameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongLength { expected, observed } => {
                write!(formatter, "pool name length {observed} is not {expected}")
            }
            Self::WrongSuffix => formatter.write_str("pool name suffix is noncanonical"),
            Self::WrongSeparator => {
                formatter.write_str("catalog pool generation separator is not '-'")
            }
            Self::UppercaseGeneration => {
                formatter.write_str("catalog pool generation uses uppercase hexadecimal")
            }
            Self::InvalidGenerationAlphabet => {
                formatter.write_str("catalog pool generation is not hexadecimal")
            }
            Self::ZeroGeneration => formatter.write_str("catalog pool generation is zero"),
            Self::DigestLength { expected, observed } => {
                write!(formatter, "pool digest length {observed} is not {expected}")
            }
            Self::UppercaseDigest => formatter.write_str("pool digest uses uppercase hexadecimal"),
            Self::InvalidDigestAlphabet => {
                formatter.write_str("pool digest is not lowercase hexadecimal")
            }
        }
    }
}

impl Error for RecoveryPoolNameError {}
