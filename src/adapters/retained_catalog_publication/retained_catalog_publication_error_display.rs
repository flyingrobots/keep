//! This module owns diagnostic text for retained-closure publication refusals.

use std::fmt;

use super::RetainedCatalogPublicationError;

impl fmt::Display for RetainedCatalogPublicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Catalog { .. } => formatter.write_str("retained catalog publication refused"),
            Self::RetentionStorage { phase, .. } | Self::HeadDecode { phase, .. } => {
                write!(formatter, "{phase} failed")
            }
            Self::RootStorage { entry, .. } => write!(
                formatter,
                "retained root read failed at root generation {}",
                entry.root_generation().get()
            ),
            Self::ManifestLength { expected, observed } => write!(
                formatter,
                "retention manifest length must be {}, observed {observed}",
                expected.get()
            ),
            Self::ManifestDecode { .. } => {
                formatter.write_str("retention manifest admission failed")
            }
            Self::ManifestDigest { .. } => {
                formatter.write_str("retention manifest digest differs from the head")
            }
            Self::ManifestGeneration { expected, observed } => write!(
                formatter,
                "retention manifest generation must be {}, observed {}",
                expected.get(),
                observed.get()
            ),
            Self::ManifestPredecessor { .. } => {
                formatter.write_str("retention manifest predecessor differs from the head")
            }
            Self::RootDecode { .. } => formatter.write_str("retained root admission failed"),
            Self::RootBinding { .. } => {
                formatter.write_str("retained root differs from its manifest entry")
            }
            Self::Closure { .. } => {
                formatter.write_str("candidate catalog does not preserve a retained closure")
            }
            Self::RetentionChanged { .. } => {
                formatter.write_str("retention head changed before catalog head replacement")
            }
        }
    }
}
