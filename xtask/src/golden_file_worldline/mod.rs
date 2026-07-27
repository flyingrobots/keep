//! This module owns ordered execution of the independent conformance oracles.

mod b3sum_oracle;
mod canonical_value;
mod capability_contract;
mod corpus_protocol;
mod digest_port;
mod error;
mod identity_oracle;
mod invalid_text_oracle;
mod mutation_oracle;
mod mutation_value;
mod scenario_oracle;

use std::path::Path;

use b3sum_oracle::B3sumOracle;
use corpus_protocol::Corpus;

pub(crate) use error::GoldenError;

pub(super) fn check(repository_root: &Path) -> Result<(), GoldenError> {
    let corpus = Corpus::open(repository_root.join("conformance/golden-file-worldline/v1"))?;
    let oracle = B3sumOracle;
    let fixtures = identity_oracle::check_identities(&corpus, &oracle)?;
    invalid_text_oracle::check(&corpus)?;
    mutation_oracle::check(&corpus, &fixtures, &oracle)?;
    scenario_oracle::check_steps(&corpus, &fixtures)?;
    capability_contract::check(&corpus)?;
    Ok(())
}
