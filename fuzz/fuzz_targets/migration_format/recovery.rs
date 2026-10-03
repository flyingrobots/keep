//! Bounded recovery-residue fuzz input and specified success/refusal properties.

use keep::{
    AdmittedStoreFormatMarker, AdmittedStoreMigrationIntent, AdmittedStoreMigrationReceipt,
    StoreMigrationEffect, StoreMigrationRecoveryAmbiguity, StoreMigrationRecoveryPlan,
    StoreMigrationResidue, plan_store_migration_recovery,
};

const INTENT_BYTES: usize = 256;
const MAX_RECORD_BYTES: usize = 513;

pub(super) fn exercise(input: &[u8]) -> Option<()> {
    let (expected_bytes, input) = input.split_at_checked(INTENT_BYTES)?;
    let expected = AdmittedStoreMigrationIntent::decode(expected_bytes).ok()?;
    let (&records, input) = input.split_first()?;
    let (&namespace, mut input) = input.split_first()?;
    let residue = StoreMigrationResidue {
        intent_stage: present(records & 1, record(&mut input)?),
        intent: present(records & 2, record(&mut input)?),
        marker_stage: present(records & 4, record(&mut input)?),
        marker: present(records & 8, record(&mut input)?),
        receipt_stage: present(records & 16, record(&mut input)?),
        receipt: present(records & 32, record(&mut input)?),
        reader_fence: namespace & 1 != 0,
        namespace_prefix: [2, 4, 8, 16, 32, 64].map(|bit| namespace & bit != 0),
    };
    let result = plan_store_migration_recovery(&expected, &residue);
    assert_version_one(&residue, &result);
    assert_pre_intent_effects(&residue, &result);
    if matches!(result, Ok(StoreMigrationRecoveryPlan::Complete)) {
        assert_complete(&residue);
    }
    Some(())
}

fn record<'a>(input: &mut &'a [u8]) -> Option<&'a [u8]> {
    let (length, remainder) = input.split_at_checked(2)?;
    let length = usize::from(u16::from_le_bytes(length.try_into().ok()?));
    if length > MAX_RECORD_BYTES {
        return None;
    }
    let (bytes, remainder) = remainder.split_at_checked(length)?;
    *input = remainder;
    Some(bytes)
}

fn present(presence: u8, bytes: &[u8]) -> Option<Vec<u8>> {
    (presence != 0).then(|| bytes.to_vec())
}

type Plan = Result<StoreMigrationRecoveryPlan, StoreMigrationRecoveryAmbiguity>;

fn assert_version_one(residue: &StoreMigrationResidue, result: &Plan) {
    let no_migration_evidence = residue.intent.is_none()
        && residue.intent_stage.is_none()
        && !residue.reader_fence
        && !residue.namespace_prefix.contains(&true)
        && residue.marker.is_none()
        && residue.marker_stage.is_none()
        && residue.receipt.is_none()
        && residue.receipt_stage.is_none();
    assert_eq!(
        matches!(result, Ok(StoreMigrationRecoveryPlan::VersionOne)),
        no_migration_evidence,
        "version-one admission requires absence of all migration evidence"
    );
}

fn assert_pre_intent_effects(residue: &StoreMigrationResidue, result: &Plan) {
    if residue.intent.is_some() {
        return;
    }
    let effect = if residue.reader_fence || residue.namespace_prefix.contains(&true) {
        Some(StoreMigrationEffect::Namespace)
    } else if residue.marker.is_some() || residue.marker_stage.is_some() {
        Some(StoreMigrationEffect::Marker)
    } else if residue.receipt.is_some() || residue.receipt_stage.is_some() {
        Some(StoreMigrationEffect::Receipt)
    } else {
        None
    };
    if let Some(expected) = effect {
        assert!(
            matches!(result, Err(StoreMigrationRecoveryAmbiguity::EffectBeforeIntent { effect }) if *effect == expected),
            "an effect before durable intent must retain its precise refusal: {result:?}"
        );
    }
}

fn assert_complete(residue: &StoreMigrationResidue) {
    assert!(
        residue.reader_fence && residue.namespace_prefix.iter().all(|present| *present),
        "complete migration needs the entire namespace"
    );
    assert!(
        residue.intent_stage.is_none()
            && residue.marker_stage.is_none()
            && residue.receipt_stage.is_none(),
        "complete migration must not leave staged evidence"
    );
    let records = residue
        .intent
        .as_deref()
        .zip(residue.marker.as_deref())
        .zip(residue.receipt.as_deref());
    let admitted = records.and_then(|((intent, marker), receipt)| {
        let intent = AdmittedStoreMigrationIntent::decode(intent).ok()?;
        let marker = AdmittedStoreFormatMarker::decode(marker).ok()?;
        AdmittedStoreMigrationReceipt::decode(receipt, &intent, &marker).ok()
    });
    assert!(
        admitted.is_some(),
        "complete migration needs jointly admitted records"
    );
}
