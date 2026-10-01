//! One structural mutation per verification receipt field and the exact
//! first refusal each must reach.

use keep::{VerificationReceiptDecodeError as DecodeError, VerificationReceiptField as Field};

/// One structural mutation and the exact first refusal it must reach.
pub struct Mutation {
    pub field: &'static str,
    pub fixture: &'static str,
    pub offset: usize,
    pub value: &'static [u8],
    pub reseal: bool,
    pub refuses: fn(&DecodeError) -> bool,
}

pub const REPORT: &str = "reference-complete-blob-report.hex";
pub const CORRUPT: &str = "durable-corrupt-chunk-refusal.hex";
pub const UNSUPPORTED: &str = "reference-unsupported-framing-refusal.hex";

pub const MATRIX: &[Mutation] = &[
    Mutation {
        field: "magic",
        fixture: REPORT,
        offset: 0,
        value: &[0x4a],
        reseal: false,
        refuses: |e| matches!(e, DecodeError::InvalidMagic { .. }),
    },
    Mutation {
        field: "version",
        fixture: REPORT,
        offset: 16,
        value: &[0, 2],
        reseal: false,
        refuses: |e| matches!(e, DecodeError::UnsupportedVersion { observed: 2, .. }),
    },
    Mutation {
        field: "record length",
        fixture: REPORT,
        offset: 18,
        value: &[1, 0],
        reseal: false,
        refuses: |e| matches!(e, DecodeError::InvalidRecordLength { observed: 256, .. }),
    },
    Mutation {
        field: "flags",
        fixture: REPORT,
        offset: 20,
        value: &[0, 0, 0, 1],
        reseal: false,
        refuses: |e| matches!(e, DecodeError::UnsupportedFlags { observed: 1 }),
    },
    Mutation {
        field: "contract",
        fixture: REPORT,
        offset: 24,
        value: &[0, 0, 0, 2],
        reseal: false,
        refuses: |e| matches!(e, DecodeError::UnsupportedContract { observed: 2, .. }),
    },
    Mutation {
        field: "checksum",
        fixture: REPORT,
        offset: 352,
        value: &[0xff],
        reseal: false,
        refuses: |e| matches!(e, DecodeError::ChecksumMismatch { .. }),
    },
    Mutation {
        field: "covered byte",
        fixture: REPORT,
        offset: 316,
        value: &[9],
        reseal: false,
        refuses: |e| matches!(e, DecodeError::ChecksumMismatch { .. }),
    },
    Mutation {
        field: "outcome",
        fixture: REPORT,
        offset: 28,
        value: &[0, 3],
        reseal: true,
        refuses: |e| {
            matches!(
                e,
                DecodeError::UnregisteredCode {
                    field: Field::Outcome,
                    observed: 3
                }
            )
        },
    },
    Mutation {
        field: "depth",
        fixture: REPORT,
        offset: 30,
        value: &[0, 8],
        reseal: true,
        refuses: |e| {
            matches!(
                e,
                DecodeError::UnregisteredCode {
                    field: Field::Depth,
                    observed: 8
                }
            )
        },
    },
    Mutation {
        field: "subject kind",
        fixture: REPORT,
        offset: 32,
        value: &[0, 3],
        reseal: true,
        refuses: |e| {
            matches!(
                e,
                DecodeError::UnregisteredCode {
                    field: Field::SubjectKind,
                    ..
                }
            )
        },
    },
    Mutation {
        field: "view kind",
        fixture: REPORT,
        offset: 34,
        value: &[0, 0],
        reseal: true,
        refuses: |e| {
            matches!(
                e,
                DecodeError::UnregisteredCode {
                    field: Field::ViewKind,
                    ..
                }
            )
        },
    },
    Mutation {
        field: "refusal class",
        fixture: REPORT,
        offset: 36,
        value: &[0, 5],
        reseal: true,
        refuses: |e| {
            matches!(
                e,
                DecodeError::UnregisteredCode {
                    field: Field::RefusalClass,
                    ..
                }
            )
        },
    },
    Mutation {
        field: "evidence kind",
        fixture: REPORT,
        offset: 38,
        value: &[0, 5],
        reseal: true,
        refuses: |e| {
            matches!(
                e,
                DecodeError::UnregisteredCode {
                    field: Field::EvidenceKind,
                    ..
                }
            )
        },
    },
    Mutation {
        field: "layout present",
        fixture: REPORT,
        offset: 40,
        value: &[0, 2],
        reseal: true,
        refuses: |e| {
            matches!(
                e,
                DecodeError::UnregisteredCode {
                    field: Field::LayoutPresent,
                    ..
                }
            )
        },
    },
    Mutation {
        field: "target present",
        fixture: REPORT,
        offset: 46,
        value: &[0, 2],
        reseal: true,
        refuses: |e| {
            matches!(
                e,
                DecodeError::UnregisteredCode {
                    field: Field::TargetPresent,
                    ..
                }
            )
        },
    },
    Mutation {
        field: "reserved",
        fixture: REPORT,
        offset: 340,
        value: &[1],
        reseal: true,
        refuses: |e| {
            matches!(
                e,
                DecodeError::NonZero {
                    field: Field::Reserved
                }
            )
        },
    },
    Mutation {
        field: "subject slot magic",
        fixture: REPORT,
        offset: 48,
        value: &[0x4a],
        reseal: true,
        refuses: |e| matches!(e, DecodeError::BlobId { .. }),
    },
    Mutation {
        field: "subject slot padding",
        fixture: REPORT,
        offset: 107,
        value: &[1],
        reseal: true,
        refuses: |e| matches!(e, DecodeError::Semantic { .. }),
    },
    Mutation {
        field: "layout slot magic",
        fixture: REPORT,
        offset: 108,
        value: &[0x4a],
        reseal: true,
        refuses: |e| matches!(e, DecodeError::LayoutId { .. }),
    },
    Mutation {
        field: "target disagrees with subject",
        fixture: REPORT,
        offset: 227,
        value: &[1],
        reseal: true,
        refuses: |e| matches!(e, DecodeError::BlobId { .. } | DecodeError::Semantic { .. }),
    },
    Mutation {
        field: "report with refusal class",
        fixture: REPORT,
        offset: 36,
        value: &[0, 1],
        reseal: true,
        refuses: |e| matches!(e, DecodeError::Semantic { .. }),
    },
    Mutation {
        field: "report without layout",
        fixture: REPORT,
        offset: 40,
        value: &[0, 0],
        reseal: true,
        refuses: |e| matches!(e, DecodeError::Semantic { .. }),
    },
    Mutation {
        field: "reference view with generation",
        fixture: REPORT,
        offset: 235,
        value: &[1],
        reseal: true,
        refuses: |e| matches!(e, DecodeError::Semantic { .. }),
    },
    Mutation {
        field: "durable view zero generation",
        fixture: CORRUPT,
        offset: 228,
        value: &[0, 0, 0, 0, 0, 0, 0, 0],
        reseal: true,
        refuses: |e| {
            matches!(
                e,
                DecodeError::NonZero {
                    field: Field::CatalogGeneration
                }
            )
        },
    },
    Mutation {
        field: "empty retention with manifest",
        fixture: CORRUPT,
        offset: 268,
        value: &[0, 0, 0, 0, 0, 0, 0, 0],
        reseal: true,
        refuses: |e| matches!(e, DecodeError::Semantic { .. }),
    },
    Mutation {
        field: "refusal verified chunks",
        fixture: CORRUPT,
        offset: 323,
        value: &[1],
        reseal: true,
        refuses: |e| matches!(e, DecodeError::Semantic { .. }),
    },
    Mutation {
        field: "refusal with target",
        fixture: CORRUPT,
        offset: 46,
        value: &[0, 1],
        reseal: true,
        refuses: |e| matches!(e, DecodeError::Semantic { .. }),
    },
    Mutation {
        field: "corruption without layout",
        fixture: CORRUPT,
        offset: 40,
        value: &[0, 0],
        reseal: true,
        refuses: |e| matches!(e, DecodeError::Semantic { .. }),
    },
    Mutation {
        field: "corruption without kind",
        fixture: CORRUPT,
        offset: 38,
        value: &[0, 0],
        reseal: true,
        refuses: |e| matches!(e, DecodeError::Semantic { .. }),
    },
    Mutation {
        field: "unindexed corruption with index",
        fixture: CORRUPT,
        offset: 38,
        value: &[0, 2],
        reseal: false,
        refuses: |e| matches!(e, DecodeError::ChecksumMismatch { .. }),
    },
    Mutation {
        field: "supported range on a corruption",
        fixture: CORRUPT,
        offset: 42,
        value: &[0, 1],
        reseal: true,
        refuses: |e| matches!(e, DecodeError::Semantic { .. }),
    },
    Mutation {
        field: "unsupported minimum unregistered",
        fixture: UNSUPPORTED,
        offset: 42,
        value: &[0, 9],
        reseal: true,
        refuses: |e| {
            matches!(
                e,
                DecodeError::UnregisteredCode {
                    field: Field::SupportedMinimum,
                    ..
                }
            )
        },
    },
    Mutation {
        field: "unsupported maximum unregistered",
        fixture: UNSUPPORTED,
        offset: 44,
        value: &[0, 0],
        reseal: true,
        refuses: |e| {
            matches!(
                e,
                DecodeError::UnregisteredCode {
                    field: Field::SupportedMaximum,
                    ..
                }
            )
        },
    },
    Mutation {
        field: "unsupported range unordered",
        fixture: UNSUPPORTED,
        offset: 42,
        value: &[0, 6],
        reseal: true,
        refuses: |e| matches!(e, DecodeError::Semantic { .. }),
    },
    Mutation {
        field: "unsupported depth inside range",
        fixture: UNSUPPORTED,
        offset: 30,
        value: &[0, 4],
        reseal: true,
        refuses: |e| matches!(e, DecodeError::Semantic { .. }),
    },
    Mutation {
        field: "unsupported with layout",
        fixture: UNSUPPORTED,
        offset: 40,
        value: &[0, 1],
        reseal: true,
        refuses: |e| matches!(e, DecodeError::LayoutId { .. }),
    },
    Mutation {
        field: "truncated",
        fixture: REPORT,
        offset: usize::MAX,
        value: &[],
        reseal: false,
        refuses: |e| matches!(e, DecodeError::WrongLength { observed: 383, .. }),
    },
];
