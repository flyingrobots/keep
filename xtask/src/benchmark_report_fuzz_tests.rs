//! Laws for the production report fuzz entry point and its fixed corpus seed.

use crate::{BenchmarkReportAdmission, admit_benchmark_report};

const SEED: &[u8] = include_bytes!("../../benchmark/baselines/c529c07-aarch64-apple-darwin.tsv");

#[test]
fn historical_seed_reaches_full_production_admission() {
    assert_eq!(
        admit_benchmark_report(SEED),
        BenchmarkReportAdmission::Admitted
    );
    assert_eq!(
        admit_benchmark_report(b"not a report\n"),
        BenchmarkReportAdmission::Refused
    );
}

#[test]
fn corpus_byte_mutations_produce_repeatable_admission_and_refusal() {
    for (position, original) in SEED.iter().enumerate().step_by(17) {
        let mut mutated = SEED.to_vec();
        if let Some(value) = mutated.get_mut(position) {
            *value = original ^ 0x80;
        }
        assert_eq!(
            admit_benchmark_report(&mutated),
            BenchmarkReportAdmission::Refused
        );
        assert_eq!(
            admit_benchmark_report(&mutated),
            BenchmarkReportAdmission::Refused
        );
    }
}
