//! Canonical verification receipt laws: the golden fixtures match a
//! handwritten oracle and the production encoder, every real report and
//! refusal projects and round-trips, every structural field has one exact
//! first refusal, and a refusal never decodes as a report.

#![expect(
    missing_docs,
    reason = "the oracle and matrix helpers are reached only from this test crate"
)]

pub mod support;

#[path = "verification_receipt/matrix.rs"]
pub mod matrix;
#[path = "verification_receipt/oracle.rs"]
pub mod oracle;

use std::error::Error;
use std::io::Cursor;

use keep::{
    AdmittedRetentionManifest, BlobId, CanonicalVerificationReceipt, ChecksummedCatalog,
    GcRetentionState, LayoutEntryLimit, LayoutId, ReceiptCorruption, ReceiptRefusal,
    ReceiptSubject, ReceiptVerificationDepth, ReferenceStore, ReferenceStoreCapacity,
    VERIFICATION_CONTRACT_VERSION, VerificationDepth, VerificationError, VerificationOutcome,
    VerificationReceipt, VerificationReceiptDecodeError as DecodeError,
    VerificationReceiptProjectionError, VerificationRefusal, VerificationSubject, VerificationView,
};
use matrix::{CORRUPT, MATRIX, REPORT};
use oracle::{BLOB_ID, LAYOUT_ID, golden_receipts};
use support::{decode_hex, domain_hash, patch};

const FIXTURES: [(&str, &str); 3] = [
    (
        "reference-complete-blob-report.hex",
        include_str!("../conformance/verification-receipt/v1/reference-complete-blob-report.hex"),
    ),
    (
        "durable-corrupt-chunk-refusal.hex",
        include_str!("../conformance/verification-receipt/v1/durable-corrupt-chunk-refusal.hex"),
    ),
    (
        "reference-unsupported-framing-refusal.hex",
        include_str!(
            "../conformance/verification-receipt/v1/reference-unsupported-framing-refusal.hex"
        ),
    ),
];
const V1_CATALOG_GENERATION_TWO: &str =
    include_str!("../conformance/segment-store/v1/one-zero-catalog-generation-two.hex");
const V2_MANIFEST: &str = include_str!("../conformance/segment-store/v2/one-root-manifest.hex");
const CHECKSUM_DOMAIN: &[u8] = b"keep.verification-receipt-checksum/v1\0";
const CHECKSUM_OFFSET: usize = 352;
const SOURCE: &[u8] = b"a receipt names exactly what one verification established";

fn fixture_bytes(name: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let (_, hex) = FIXTURES
        .iter()
        .find(|(fixture, _)| *fixture == name)
        .ok_or("unknown receipt fixture")?;
    Ok(decode_hex(hex.trim_end())?)
}

/// The frozen durable coordinates through their public decoders: the
/// generation-two catalog and the generation-one manifest.
fn durable_view() -> Result<VerificationView, Box<dyn Error>> {
    let catalog_bytes = decode_hex(V1_CATALOG_GENERATION_TWO.trim_end())?;
    let catalog = ChecksummedCatalog::decode(&catalog_bytes)?;
    let manifest_bytes = decode_hex(V2_MANIFEST.trim_end())?;
    let manifest = AdmittedRetentionManifest::decode(&manifest_bytes)?;
    let coordinates = oracle::durable_coordinates()?;
    assert_eq!(catalog.digest().as_bytes(), &coordinates.catalog_digest);
    assert_eq!(manifest.digest().as_bytes(), &coordinates.manifest_digest);
    Ok(VerificationView::Durable {
        catalog_generation: catalog.generation(),
        catalog_digest: catalog.digest(),
        retention: GcRetentionState::Published {
            generation: manifest.manifest().generation(),
            manifest_digest: manifest.digest(),
        },
    })
}

fn reseal(bytes: &mut [u8]) -> Result<(), Box<dyn Error>> {
    let checksum = domain_hash(
        CHECKSUM_DOMAIN,
        bytes.get(..CHECKSUM_OFFSET).ok_or("preimage")?,
    );
    patch(bytes, CHECKSUM_OFFSET, &checksum)?;
    Ok(())
}

#[test]
fn golden_receipts_match_the_oracle_and_decode_from_another_process() -> Result<(), Box<dyn Error>>
{
    for golden in golden_receipts()? {
        let frozen = fixture_bytes(golden.fixture)?;
        assert_eq!(
            frozen, golden.bytes,
            "{}: fixture drifted from the oracle",
            golden.case
        );
        // The fixture was written by another process; admission must be exact.
        let admitted = CanonicalVerificationReceipt::decode(&frozen)?;
        assert_eq!(
            admitted.encoded().as_slice(),
            frozen.as_slice(),
            "{}",
            golden.case
        );
        let reencoded = CanonicalVerificationReceipt::encode(admitted.receipt());
        assert_eq!(
            reencoded, admitted,
            "{}: re-encoding is not canonical",
            golden.case
        );
    }
    Ok(())
}

#[test]
fn the_golden_report_and_refusals_state_exactly_their_frozen_coordinates()
-> Result<(), Box<dyn Error>> {
    let blob = BlobId::parse_binary(&BLOB_ID)?;
    let layout = LayoutId::parse_binary(&LAYOUT_ID)?;

    let report = CanonicalVerificationReceipt::decode(&fixture_bytes(
        "reference-complete-blob-report.hex",
    )?)?;
    assert_eq!(report.receipt().view(), VerificationView::Reference);
    assert_eq!(
        report.receipt().outcome(),
        VerificationOutcome::Established {
            subject: ReceiptSubject::Blob(blob),
            depth: ReceiptVerificationDepth::CompleteBlobIdentity,
            layout,
            target: blob,
            chunks_verified: 1,
        }
    );

    let corrupt =
        CanonicalVerificationReceipt::decode(&fixture_bytes("durable-corrupt-chunk-refusal.hex")?)?;
    assert_eq!(corrupt.receipt().view(), durable_view()?);
    assert_eq!(
        corrupt.receipt().outcome(),
        VerificationOutcome::Refused(ReceiptRefusal::Corrupt {
            subject: ReceiptSubject::Layout(layout),
            stage: ReceiptVerificationDepth::ChunkIdentity,
            evidence: ReceiptCorruption::ChunkIdentity { layout, index: 0 },
        })
    );

    let unsupported = CanonicalVerificationReceipt::decode(&fixture_bytes(
        "reference-unsupported-framing-refusal.hex",
    )?)?;
    assert_eq!(
        unsupported.receipt().outcome(),
        VerificationOutcome::Refused(ReceiptRefusal::Unsupported {
            subject: ReceiptSubject::Blob(blob),
            requested: ReceiptVerificationDepth::Framing,
            supported_minimum: ReceiptVerificationDepth::ChunkIdentity,
            supported_maximum: ReceiptVerificationDepth::CompleteBlobIdentity,
        })
    );
    assert_eq!(VERIFICATION_CONTRACT_VERSION, 1);
    Ok(())
}

#[test]
fn every_reference_store_outcome_projects_and_round_trips() -> Result<(), Box<dyn Error>> {
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let mut source = Cursor::new(SOURCE);
    let published = store
        .stage(&mut source, LayoutEntryLimit::MAXIMUM)?
        .commit(&mut store)?;
    let subject = VerificationSubject::Blob {
        identity: published.target(),
    };
    let mut receipts = Vec::new();
    for depth in ReceiptVerificationDepth::ALL.iter().copied() {
        let receipt = match store.verify(subject, depth.into()) {
            Ok(report) => VerificationReceipt::from_report(&report, VerificationView::Reference)?,
            Err(error @ VerificationError::Refused { .. }) => {
                VerificationReceipt::from_error(&error, VerificationView::Reference)?
            }
            Err(other) => return Err(other.into()),
        };
        receipts.push(receipt);
    }
    let mut staged = Cursor::new(b"staged but never committed");
    let absent = store
        .stage(&mut staged, LayoutEntryLimit::MAXIMUM)?
        .target();
    match store.verify(
        VerificationSubject::Blob { identity: absent },
        VerificationDepth::ChunkIdentity,
    ) {
        Err(
            error @ VerificationError::Refused {
                refusal: VerificationRefusal::Missing { .. },
                ..
            },
        ) => {
            receipts.push(VerificationReceipt::from_error(
                &error,
                VerificationView::Reference,
            )?);
        }
        other => return Err(format!("absent blob was not missing: {other:?}").into()),
    }
    for receipt in receipts {
        let canonical = CanonicalVerificationReceipt::encode(&receipt);
        let decoded = CanonicalVerificationReceipt::decode(canonical.encoded())?;
        assert_eq!(decoded.receipt(), &receipt);
        assert_eq!(decoded.encoded(), canonical.encoded());
    }
    Ok(())
}

#[test]
fn every_structural_field_has_one_exact_first_refusal() -> Result<(), Box<dyn Error>> {
    for mutation in MATRIX {
        let mut bytes = fixture_bytes(mutation.fixture)?;
        if mutation.offset == usize::MAX {
            bytes.truncate(383);
        } else {
            patch(&mut bytes, mutation.offset, mutation.value)?;
        }
        if mutation.reseal {
            reseal(&mut bytes)?;
        }
        let error = CanonicalVerificationReceipt::decode(&bytes)
            .err()
            .ok_or_else(|| format!("mutated {} was admitted", mutation.field))?;
        assert!(
            (mutation.refuses)(&error),
            "{}: unexpected first refusal {error:?}",
            mutation.field
        );
    }
    Ok(())
}

#[test]
fn a_refusal_never_decodes_as_a_report_and_a_report_never_as_a_refusal()
-> Result<(), Box<dyn Error>> {
    // Flipping only the outcome kind leaves every other field contradicting
    // it, so neither direction is admitted.
    let mut refusal = fixture_bytes(CORRUPT)?;
    patch(&mut refusal, 28, &[0, 1])?;
    reseal(&mut refusal)?;
    assert!(matches!(
        CanonicalVerificationReceipt::decode(&refusal),
        Err(DecodeError::Semantic { .. })
    ));
    let mut report = fixture_bytes(REPORT)?;
    patch(&mut report, 28, &[0, 2])?;
    reseal(&mut report)?;
    assert!(matches!(
        CanonicalVerificationReceipt::decode(&report),
        Err(DecodeError::Semantic { .. })
    ));
    Ok(())
}

/// Size: small. Oracle: reference verification establishes no durable view coordinates.
#[test]
fn a_reference_report_cannot_be_relabeled_as_durable() -> Result<(), Box<dyn Error>> {
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let published = store
        .stage(&mut Cursor::new(SOURCE), LayoutEntryLimit::MAXIMUM)?
        .commit(&mut store)?;
    let report = store.verify(
        VerificationSubject::Blob {
            identity: published.target(),
        },
        VerificationDepth::CompleteBlobIdentity,
    )?;
    assert_eq!(
        VerificationReceipt::from_report(&report, durable_view()?),
        Err(VerificationReceiptProjectionError::ViewMismatch)
    );
    let receipt = VerificationReceipt::from_report(&report, VerificationView::Reference)?;
    assert_eq!(receipt.view(), VerificationView::Reference);
    Ok(())
}

/// Size: small. Oracle: absence in the reference view proves nothing about a durable catalog.
#[test]
fn a_reference_refusal_cannot_be_relabeled_as_durable() -> Result<(), Box<dyn Error>> {
    let store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let absent = BlobId::hash_bytes(SOURCE)?;
    let error = store
        .verify(
            VerificationSubject::Blob { identity: absent },
            VerificationDepth::CompleteBlobIdentity,
        )
        .err()
        .ok_or("absent blob was reported present")?;
    assert_eq!(
        VerificationReceipt::from_error(&error, durable_view()?),
        Err(VerificationReceiptProjectionError::ViewMismatch)
    );
    Ok(())
}

/// Size: small. Oracle: v1 registers exactly depth codes 1 through 7; `SnapshotBinding` has no slot.
#[test]
fn snapshot_binding_is_refused_instead_of_encoded_as_another_depth() -> Result<(), Box<dyn Error>> {
    let store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let absent = BlobId::hash_bytes(SOURCE)?;
    let error = store
        .verify(
            VerificationSubject::Blob { identity: absent },
            VerificationDepth::SnapshotBinding,
        )
        .err()
        .ok_or("reference view established snapshot binding")?;
    assert_eq!(
        VerificationReceipt::from_error(&error, VerificationView::Reference),
        Err(VerificationReceiptProjectionError::Depth(
            VerificationDepth::SnapshotBinding
        ))
    );
    Ok(())
}

/// Size: small. Oracle: the independent one-zero conformance record, not this encoder.
#[test]
fn live_reference_verification_reproduces_the_frozen_report() -> Result<(), Box<dyn Error>> {
    let mut store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let published = store
        .stage(&mut Cursor::new([0_u8]), LayoutEntryLimit::MAXIMUM)?
        .commit(&mut store)?;
    let report = store.verify(
        VerificationSubject::Blob {
            identity: published.target(),
        },
        VerificationDepth::CompleteBlobIdentity,
    )?;
    let receipt = VerificationReceipt::from_report(&report, VerificationView::Reference)?;
    assert_eq!(
        CanonicalVerificationReceipt::encode(&receipt)
            .encoded()
            .as_slice(),
        fixture_bytes("reference-complete-blob-report.hex")?
    );
    Ok(())
}

/// Size: small. Oracle: the independent unsupported-framing conformance record.
#[test]
fn live_reference_refusal_reproduces_the_frozen_supported_interval() -> Result<(), Box<dyn Error>> {
    let store = ReferenceStore::new(ReferenceStoreCapacity::new(1_048_576));
    let target = BlobId::parse_binary(&BLOB_ID)?;
    let error = store
        .verify(
            VerificationSubject::Blob { identity: target },
            VerificationDepth::Framing,
        )
        .err()
        .ok_or("reference framing unexpectedly supported")?;
    let receipt = VerificationReceipt::from_error(&error, VerificationView::Reference)?;
    assert_eq!(
        CanonicalVerificationReceipt::encode(&receipt)
            .encoded()
            .as_slice(),
        fixture_bytes("reference-unsupported-framing-refusal.hex")?
    );
    Ok(())
}
