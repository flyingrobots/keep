//! This boundary module owns the registered recovery-disposition
//! enumerations frozen in `definition.tsv`.

macro_rules! registered_enum {
    (
        $(#[$doc:meta])* $name:ident { $($(#[$variant_doc:meta])* $variant:ident = $code:literal / $identifier:literal),+ $(,)? }
    ) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub enum $name {
            $($(#[$variant_doc])* $variant,)+
        }

        impl $name {
            /// Every registered value in code order.
            pub const ALL: &'static [Self] = &[$(Self::$variant,)+];

            /// Returns the registered wire code.
            #[must_use]
            pub const fn code(self) -> u16 {
                match self {
                    $(Self::$variant => $code,)+
                }
            }

            /// Returns the registered identifier from `definition.tsv`.
            #[must_use]
            pub const fn identifier(self) -> &'static str {
                match self {
                    $(Self::$variant => $identifier,)+
                }
            }

            /// Admits one registered wire code; every other code refuses.
            #[must_use]
            pub const fn from_code(code: u16) -> Option<Self> {
                match code {
                    $($code => Some(Self::$variant),)+
                    _ => None,
                }
            }
        }
    };
}

registered_enum! {
    /// The kind of physical artifact one disposition names.
    RecoveryArtifactKind {
        /// An immutable segment in `segments/`.
        Segment = 1 / "segment",
        /// An immutable catalog generation in `catalogs/`.
        Catalog = 2 / "catalog",
        /// An immutable retention root in `retention/roots/<namespace>/`.
        RetentionRoot = 3 / "retention-root",
        /// An immutable retention manifest in `retention/manifests/`.
        RetentionManifest = 4 / "retention-manifest",
        /// A retention head stage, `retention/head.next`.
        RetentionHead = 5 / "retention-head",
    }
}

registered_enum! {
    /// The decision one disposition records.
    RecoveryDispositionDecision {
        /// Complete the interrupted publication and keep the artifact.
        Finalize = 1 / "finalize",
        /// Release the artifact from recovery protection so GC may plan it.
        Retire = 2 / "retire",
    }
}

registered_enum! {
    /// The recovery classification the artifact was admitted under.
    RecoveryClassification {
        /// A complete, verified artifact linked into its pool that no head,
        /// catalog, or manifest names.
        CompleteOrphan = 1 / "complete-orphan",
        /// A complete, verified fixed stage not yet linked into its pool.
        CompleteStage = 2 / "complete-stage",
        /// A complete artifact whose generation a later publication
        /// superseded before it became visible.
        StaleGeneration = 3 / "stale-generation",
    }
}
