//! Field-complete corruption ledgers over every durable segment-store record:
//! each frozen mutation reaches exactly its named first refusal at its named
//! verification stage, through the public decoders.

pub mod support;

#[path = "segment_store_mutations/classify.rs"]
mod classify;
#[path = "segment_store_mutations/fixtures.rs"]
mod fixtures;
#[path = "segment_store_mutations/ledger.rs"]
mod ledger;
#[path = "segment_store_mutations/recipes.rs"]
mod recipes;

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;

use keep::VerificationDepth;

use classify::classify;
use ledger::{Format, MutationCase, mutation_cases};

const STAGES: [&str; 4] = ["framing", "checksum", "identity", "binding"];
const V1_RECORDS: [&str; 8] = [
    "segment-header",
    "segment-record",
    "segment-seal",
    "segment",
    "catalog",
    "catalog-entry",
    "catalog-binding",
    "publication-head",
];
const V2_RECORDS: [&str; 9] = [
    "format-marker",
    "migration-intent",
    "migration-receipt",
    "retention-root",
    "retention-manifest",
    "retention-head",
    "gc-intent",
    "gc-receipt",
    "disposition",
];

fn mutated(case: &MutationCase) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = case.mutated_bytes()?;
    match case.checksum_posture {
        "preserve-v1" => {}
        "recompute-v1" => recipes::recompute(case.record, &mut bytes)?,
        "recompute-trailer-v1" => recipes::recompute_trailer(case.record, &mut bytes)?,
        "recompute-checksum-v1" => recipes::recompute_checksum_only(case.record, &mut bytes)?,
        _ => return Err(format!("{}: unknown checksum posture", case.case).into()),
    }
    Ok(bytes)
}

#[test]
fn every_frozen_mutation_reaches_its_exact_first_refusal() -> Result<(), Box<dyn Error>> {
    let mut seen = BTreeSet::new();
    let mut outcome_stages: BTreeMap<&str, &str> = BTreeMap::new();
    let mut differences = Vec::new();
    for case in mutation_cases()? {
        assert!(
            seen.insert((case.format, case.case)),
            "{}: duplicate case",
            case.case
        );
        assert!(
            STAGES.contains(&case.stage),
            "{}: unregistered stage",
            case.case
        );
        let bytes = mutated(&case)?;
        match classify(case.format, case.record, &bytes) {
            Ok(refusal)
                if refusal.outcome == case.expected_outcome && refusal.stage == case.stage => {}
            Ok(refusal) => differences.push(format!(
                "{}: expected {} ({}), observed {} ({})",
                case.case, case.expected_outcome, case.stage, refusal.outcome, refusal.stage
            )),
            Err(error) => differences.push(format!("{}: {error}", case.case)),
        }
        if let Some(previous) = outcome_stages.insert(case.expected_outcome, case.stage) {
            assert_eq!(
                previous, case.stage,
                "{}: one outcome, two stages",
                case.case
            );
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
    Ok(())
}

#[test]
fn every_durable_record_has_a_ledger_and_every_row_cites_a_requirement()
-> Result<(), Box<dyn Error>> {
    let cases = mutation_cases()?;
    for (format, records) in [
        (Format::V1, V1_RECORDS.as_slice()),
        (Format::V2, V2_RECORDS.as_slice()),
    ] {
        for record in records {
            let rows = cases
                .iter()
                .filter(|case| case.format == format && case.record == *record)
                .count();
            assert!(rows >= 3, "{record}: fewer than three mutations");
        }
    }
    for case in &cases {
        assert!(
            case.requirement.starts_with("KEEP-") && case.requirement.len() > 10,
            "{}: malformed requirement",
            case.case
        );
        assert!(
            case.expected_outcome
                .starts_with(&format!("{}.", case.record))
                || case.record == "segment-record"
                || case.record == "segment"
                || case.record == "segment-seal"
                || case.record == "catalog-entry"
                || case.record == "catalog",
            "{}: outcome names another record",
            case.case
        );
    }
    Ok(())
}

#[test]
fn ledger_stages_order_like_verification_depths() {
    // `framing` and `checksum` are the two structural depths a durable view
    // establishes first, in that order; `identity` follows them; `binding`
    // is a cross-record contradiction established at the record's role.
    assert!(VerificationDepth::Framing < VerificationDepth::Checksum);
    assert!(VerificationDepth::Checksum < VerificationDepth::ChunkIdentity);
    assert_eq!(STAGES, ["framing", "checksum", "identity", "binding"]);
}
