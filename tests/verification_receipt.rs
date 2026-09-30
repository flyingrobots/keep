//! Canonical verification receipt laws: the golden fixtures match a
//! handwritten oracle and the production encoder, every real report and
//! refusal projects and round-trips, every structural field has one exact
//! first refusal, and a refusal never decodes as a report.

pub mod support;

#[path = "verification_receipt/matrix.rs"]
mod matrix;
#[path = "verification_receipt/oracle.rs"]
mod oracle;

use std::error::Error;
use std::io::Cursor;

use keep::{
    AdmittedRetentionManifest, BlobId, CanonicalVerificationReceipt, ChecksummedCatalog,
    GcRetentionState, LayoutEntryLimit, LayoutId, ReceiptCorruption, ReceiptRefusal,
    ReferenceStore, ReferenceStoreCapacity, VERIFICATION_CONTRACT_VERSION, VerificationDepth,
    VerificationError, VerificationOutcome, VerificationReceipt,
    VerificationReceiptDecodeError as DecodeError, VerificationRefusal, VerificationSubject,
    VerificationView,
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
            subject: VerificationSubject::Blob(blob),
            depth: VerificationDepth::CompleteBlobIdentity,
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
            subject: VerificationSubject::Layout(layout),
            stage: VerificationDepth::ChunkIdentity,
            evidence: ReceiptCorruption::ChunkIdentity { layout, index: 0 },
        })
    );

    let unsupported = CanonicalVerificationReceipt::decode(&fixture_bytes(
        "reference-unsupported-framing-refusal.hex",
    )?)?;
    assert_eq!(
        unsupported.receipt().outcome(),
        VerificationOutcome::Refused(ReceiptRefusal::Unsupported {
            subject: VerificationSubject::Blob(blob),
            requested: VerificationDepth::Framing,
            supported_minimum: VerificationDepth::ChunkIdentity,
            supported_maximum: VerificationDepth::CompleteBlobIdentity,
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
    let subject = VerificationSubject::Blob(published.target());
    let mut receipts = Vec::new();
    for depth in VerificationDepth::ALL {
        let receipt = match store.verify(subject, depth) {
            Ok(report) => VerificationReceipt::from_report(&report, VerificationView::Reference),
            Err(VerificationError::Refused(refusal)) => {
                VerificationReceipt::from_refusal(refusal.as_ref(), VerificationView::Reference)
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
        VerificationSubject::Blob(absent),
        VerificationDepth::ChunkIdentity,
    ) {
        Err(VerificationError::Refused(refusal))
            if matches!(*refusal, VerificationRefusal::Missing { .. }) =>
        {
            receipts.push(VerificationReceipt::from_refusal(
                refusal.as_ref(),
                durable_view()?,
            ));
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
