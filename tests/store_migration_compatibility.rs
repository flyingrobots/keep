//! Public migration record admission preserves precise version and mandatory-flag refusals.

#[allow(
    dead_code,
    reason = "shared immutable record fixtures include vectors used by other codec laws"
)]
#[path = "store_migration_receipt/fixture.rs"]
mod fixture;
mod support;

use keep::{
    AdmittedStoreFormatMarker as Marker, AdmittedStoreMigrationIntent as Intent,
    AdmittedStoreMigrationReceipt as Receipt, StoreFormatMarkerDecodeError as MarkerError,
    StoreMigrationIntentDecodeError as IntentError,
    StoreMigrationReceiptDecodeError as ReceiptError,
};
use std::error::Error;

// Size: small. Oracle: all migration records use version 2 at bytes 16..18 (v2 format specification).
// Delete only if version 2 is retired or stronger public codec laws subsume it.
#[test]
fn unsupported_migration_versions_refuse_with_exact_coordinates() -> Result<(), Box<dyn Error>> {
    let marker = fixture::marker_bytes()?;
    let intent = fixture::intent_bytes()?;
    let admitted_marker = Marker::decode(&marker)?;
    let admitted_intent = Intent::decode(&intent)?;
    for version in [0_u16, 1, 3, u16::MAX] {
        let field = version.to_be_bytes();
        assert_eq!(
            Marker::decode(&changed(marker.clone(), 16, &field)?).err(),
            Some(MarkerError::UnsupportedVersion {
                expected: 2,
                observed: version
            })
        );
        assert_eq!(
            Intent::decode(&changed(intent.clone(), 16, &field)?).err(),
            Some(IntentError::UnsupportedVersion {
                expected: 2,
                observed: version
            })
        );
        assert_eq!(
            Receipt::decode(
                &changed(fixture::receipt_bytes()?, 16, &field)?,
                &admitted_intent,
                &admitted_marker
            )
            .err(),
            Some(ReceiptError::UnsupportedVersion {
                expected: 2,
                observed: version
            })
        );
    }
    Ok(())
}

// Size: small. Oracle: every bit of the v2 flags word at bytes 20..24 is mandatory and unsupported.
// Delete only when specific flag bits gain documented compatible semantics.
#[test]
fn every_unknown_mandatory_flag_refuses_without_downgrade() -> Result<(), Box<dyn Error>> {
    let marker = fixture::marker_bytes()?;
    let intent = fixture::intent_bytes()?;
    let admitted_marker = Marker::decode(&marker)?;
    let admitted_intent = Intent::decode(&intent)?;
    for bit in 0..32 {
        let flags = 1_u32.checked_shl(bit).ok_or("flag shift overflow")?;
        let field = flags.to_be_bytes();
        assert_eq!(
            Marker::decode(&changed(marker.clone(), 20, &field)?).err(),
            Some(MarkerError::UnsupportedFlags { observed: flags })
        );
        assert_eq!(
            Intent::decode(&changed(intent.clone(), 20, &field)?).err(),
            Some(IntentError::UnsupportedFlags { observed: flags })
        );
        assert_eq!(
            Receipt::decode(
                &changed(fixture::receipt_bytes()?, 20, &field)?,
                &admitted_intent,
                &admitted_marker
            )
            .err(),
            Some(ReceiptError::UnsupportedFlags { observed: flags })
        );
    }
    Ok(())
}

fn changed(mut bytes: Vec<u8>, start: usize, field: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    let end = start
        .checked_add(field.len())
        .ok_or("field range overflow")?;
    bytes
        .get_mut(start..end)
        .ok_or("field outside fixture")?
        .copy_from_slice(field);
    Ok(bytes)
}
