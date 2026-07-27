//! This module owns ordered execution of the independent conformance oracles.

mod canonical_value;
mod capability_contract;
mod corpus_protocol;
mod error;
mod identity_oracle;
mod invalid_text_oracle;
mod mutation_oracle;
mod mutation_value;
mod scenario_oracle;

use std::path::Path;

use corpus_protocol::Corpus;

pub(crate) use error::GoldenError;

pub(super) fn check(repository_root: &Path) -> Result<(), GoldenError> {
    let corpus = Corpus::new(repository_root.join("conformance/golden-file-worldline/v1"));
    let fixtures = identity_oracle::check_identities(&corpus)?;
    invalid_text_oracle::check(&corpus)?;
    mutation_oracle::check(&corpus, &fixtures)?;
    scenario_oracle::check_steps(&corpus, &fixtures)?;
    capability_contract::check(&corpus)?;
    Ok(())
}
