//! This module owns the source-bound historical benchmark-report seed.

use super::{FuzzSeedError, Seed};

const REPORT: &[u8] =
    include_bytes!("../../../benchmark/baselines/c529c07-aarch64-apple-darwin.tsv");

pub(super) fn seeds() -> Result<Vec<Seed>, FuzzSeedError> {
    Ok(vec![Seed::new(
        "benchmark_report",
        "canonical-v1",
        REPORT.to_vec(),
    )?])
}
