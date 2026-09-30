//! Decoding one mutated record through its public entry point and naming
//! the first refusal as `<record>.<variant>` with its verification stage.

use std::fmt::Debug;
use std::io;

use keep::{
    AdmittedGcRetirementIntent, AdmittedGcRetirementReceipt, AdmittedRecoveryDispositionReceipt,
    AdmittedRetentionManifest, AdmittedRetentionRoot, AdmittedSegment, AdmittedStoreFormatMarker,
    AdmittedStoreMigrationIntent, AdmittedStoreMigrationReceipt, CatalogAdmissionError,
    CatalogDecodeError, ChecksummedCatalog, ChecksummedPublicationHead, ChecksummedRetentionHead,
    LayoutEntryLimit, SegmentReadError, SegmentReadPolicy, SegmentRecordAdmissionError,
    SegmentRecordDecodeError, SegmentRecordLimit,
};

use super::fixtures::fixture;
use super::ledger::Format;
use crate::support::{decode_hex, invalid_corpus};

/// One classified refusal.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Refusal {
    pub(crate) outcome: String,
    pub(crate) stage: &'static str,
}

const fn policy() -> SegmentReadPolicy {
    SegmentReadPolicy::new(SegmentRecordLimit::MAXIMUM, LayoutEntryLimit::MAXIMUM)
}

fn canonical(format: Format, name: &str) -> Result<Vec<u8>, io::Error> {
    let hex = fixture(format, name)?;
    decode_hex(hex.strip_suffix('\n').unwrap_or(hex))
}

/// The variant name of a `Debug`-rendered error, in kebab case.
fn variant<E: Debug>(error: &E) -> String {
    let rendered = format!("{error:?}");
    let name = rendered
        .split(|character: char| !character.is_ascii_alphanumeric())
        .next()
        .unwrap_or_default();
    let mut kebab = String::new();
    for (index, character) in name.chars().enumerate() {
        if character.is_ascii_uppercase() && index != 0 {
            kebab.push('-');
        }
        kebab.push(character.to_ascii_lowercase());
    }
    kebab
}

fn refusal(record: &str, variant_name: &str) -> Refusal {
    Refusal {
        outcome: format!("{record}.{variant_name}"),
        stage: stage_of(record, variant_name),
    }
}

/// The verification stage each registered refusal establishes.
///
/// `checksum` refusals are a record's own integrity trailer; `identity`
/// refusals are content that does not hash to its declared identity;
/// `binding` refusals are a contradiction with another record or a
/// registered definition; everything else is canonical framing.
fn stage_of(record: &str, variant_name: &str) -> &'static str {
    let checksum = matches!(
        (record, variant_name),
        (_, "checksum-mismatch")
            | (
                "segment-seal",
                "seal-checksum-mismatch" | "segment-digest-mismatch"
            )
            | ("catalog", "digest-mismatch")
            | (
                "retention-root",
                "root-digest-mismatch" | "anchor-set-digest-mismatch"
            )
            | (
                "retention-manifest",
                "manifest-digest-mismatch" | "entry-set-digest-mismatch"
            )
            | (
                "gc-intent",
                "intent-digest-mismatch" | "candidate-set-digest-mismatch"
            )
    );
    if checksum {
        return "checksum";
    }
    if matches!(
        (record, variant_name),
        ("segment-record", "chunk-identity-mismatch")
    ) {
        return "identity";
    }
    let binding = matches!(record, "catalog-binding" | "head-binding")
        || matches!(
            (record, variant_name),
            ("format-marker", "definition-digest-mismatch")
                | (
                    "migration-intent",
                    "definition-digest-mismatch" | "store-identifier-mismatch"
                )
                | (
                    "migration-receipt",
                    "intent-digest-mismatch" | "store-identifier-mismatch"
                )
                | ("migration-receipt", "format-marker-digest-mismatch")
                | ("retention-root", "profile")
                | ("gc-intent", "profile")
                | ("gc-receipt", _)
                | ("disposition", "empty-retention-digest-mismatch")
        ) && !matches!(
            variant_name,
            "wrong-length"
                | "invalid-magic"
                | "unsupported-version"
                | "invalid-record-length"
                | "unsupported-flags"
                | "non-zero-reserved"
                | "zero-generation"
        );
    if binding { "binding" } else { "framing" }
}

fn admitted(case: &str) -> io::Error {
    invalid_corpus(match case.is_empty() {
        true => "mutation was admitted",
        false => "mutation was unexpectedly admitted",
    })
}

/// Decodes `bytes` as `record` and classifies the first refusal.
///
/// # Errors
///
/// Returns a corpus error when the record is unknown, a context fixture is
/// malformed, or the mutation was admitted.
pub(crate) fn classify(format: Format, record: &str, bytes: &[u8]) -> Result<Refusal, io::Error> {
    match (format, record) {
        (Format::V1, "segment-header" | "segment-record" | "segment-seal" | "segment") => {
            let error = AdmittedSegment::decode(bytes, policy())
                .err()
                .ok_or_else(|| admitted(record))?;
            Ok(segment_refusal(&error))
        }
        (Format::V1, "catalog" | "catalog-entry") => {
            let error = ChecksummedCatalog::decode(bytes)
                .err()
                .ok_or_else(|| admitted(record))?;
            Ok(catalog_refusal(&error))
        }
        (Format::V1, "catalog-binding") => {
            let segment_bytes = canonical(format, "one-zero-segment.hex")?;
            let segment = AdmittedSegment::decode(&segment_bytes, policy()).map_err(corpus)?;
            let segments = [segment];
            let catalog = ChecksummedCatalog::decode(bytes).map_err(corpus)?;
            match catalog.admit(&segments) {
                Ok(_) => Err(admitted(record)),
                Err(CatalogAdmissionError::Catalog { source }) => Ok(catalog_refusal(&source)),
                Err(error) => Ok(refusal("catalog-binding", &variant(&error))),
            }
        }
        (Format::V1, "publication-head") => {
            let error = ChecksummedPublicationHead::decode(bytes)
                .err()
                .ok_or_else(|| admitted(record))?;
            Ok(refusal("publication-head", &variant(&error)))
        }
        (Format::V1, "head-binding") => {
            let segment_bytes = canonical(format, "one-zero-segment.hex")?;
            let segment = AdmittedSegment::decode(&segment_bytes, policy()).map_err(corpus)?;
            let segments = [segment];
            let catalog_bytes = canonical(format, "one-zero-catalog.hex")?;
            let catalog = ChecksummedCatalog::decode(&catalog_bytes)
                .map_err(corpus)?
                .admit(&segments)
                .map_err(corpus)?;
            let head = ChecksummedPublicationHead::decode(bytes).map_err(corpus)?;
            let error = head.admit(catalog).err().ok_or_else(|| admitted(record))?;
            Ok(refusal("head-binding", &variant(&error)))
        }
        (Format::V2, "format-marker") => simple(record, AdmittedStoreFormatMarker::decode(bytes)),
        (Format::V2, "migration-intent") => {
            simple(record, AdmittedStoreMigrationIntent::decode(bytes))
        }
        (Format::V2, "migration-receipt") => {
            let intent_bytes = canonical(format, "migration-intent.hex")?;
            let marker_bytes = canonical(format, "format-marker.hex")?;
            let intent = AdmittedStoreMigrationIntent::decode(&intent_bytes).map_err(corpus)?;
            let marker = AdmittedStoreFormatMarker::decode(&marker_bytes).map_err(corpus)?;
            simple(
                record,
                AdmittedStoreMigrationReceipt::decode(bytes, &intent, &marker),
            )
        }
        (Format::V2, "retention-root") => simple(record, AdmittedRetentionRoot::decode(bytes)),
        (Format::V2, "retention-manifest") => {
            simple(record, AdmittedRetentionManifest::decode(bytes))
        }
        (Format::V2, "retention-head") => simple(record, ChecksummedRetentionHead::decode(bytes)),
        (Format::V2, "gc-intent") => simple(record, AdmittedGcRetirementIntent::decode(bytes)),
        (Format::V2, "gc-receipt") => {
            let intent_bytes = canonical(format, "one-candidate-gc-intent.hex")?;
            let intent = AdmittedGcRetirementIntent::decode(&intent_bytes).map_err(corpus)?;
            simple(record, AdmittedGcRetirementReceipt::decode(bytes, &intent))
        }
        (Format::V2, "disposition") => {
            simple(record, AdmittedRecoveryDispositionReceipt::decode(bytes))
        }
        _ => Err(invalid_corpus("mutation ledger names an unknown record")),
    }
}

fn simple<T, E: Debug>(record: &str, result: Result<T, E>) -> Result<Refusal, io::Error> {
    let error = result.err().ok_or_else(|| admitted(record))?;
    Ok(refusal(record, &variant(&error)))
}

fn corpus<E: Debug>(error: E) -> io::Error {
    io::Error::other(format!("canonical context fixture refused: {error:?}"))
}

fn segment_refusal(error: &SegmentReadError) -> Refusal {
    match error {
        SegmentReadError::Header { source } => refusal("segment-header", &variant(source)),
        SegmentReadError::Seal { source } => refusal("segment-seal", &variant(source)),
        SegmentReadError::RecordHeader { source, .. } => {
            refusal("segment-record", &variant(source))
        }
        SegmentReadError::RecordDecode { source, .. } => match source {
            SegmentRecordDecodeError::Header { source } => {
                refusal("segment-record", &variant(source))
            }
            other => refusal("segment-record", &variant(other)),
        },
        SegmentReadError::RecordAdmission { source, .. } => match source {
            SegmentRecordAdmissionError::Header { source } => {
                refusal("segment-record", &variant(source))
            }
            other => refusal("segment-record", &variant(other)),
        },
        other => refusal("segment", &variant(other)),
    }
}

fn catalog_refusal(error: &CatalogDecodeError) -> Refusal {
    match error {
        CatalogDecodeError::Entry { source, .. } => refusal("catalog-entry", &variant(source)),
        other => refusal("catalog", &variant(other)),
    }
}
