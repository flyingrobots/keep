//! This module owns identity-case admission-order regression evidence.

use std::env;
use std::fs;
use std::process;

use super::{Corpus, GoldenError, check_identities, digest, verify_independent_digest};
use crate::golden_file_worldline::digest_port::IdentityDigestOracle;

struct InProcessOracle;

impl IdentityDigestOracle for InProcessOracle {
    fn identity_digest(&self, payload: &[u8]) -> Result<[u8; 32], GoldenError> {
        Ok(*digest(payload)?.as_bytes())
    }
}

#[test]
fn independent_digest_mismatch_is_refused() {
    let in_process = blake3::hash(b"in-process");
    let result = verify_independent_digest(&in_process, [0_u8; 32]);
    assert!(matches!(
        result,
        Err(GoldenError::Violation(ref message))
            if message == "independent BLAKE3 digest mismatch"
    ));
}

#[test]
fn duplicate_identity_precedes_malformed_identity_semantics() -> Result<(), GoldenError> {
    let root = env::temp_dir().join(format!("keep-duplicate-identity-{}", process::id()));
    fs::create_dir(&root)
        .map_err(|source| GoldenError::io("create identity test corpus", &root, source))?;
    let first = include_str!("../../../../conformance/golden-file-worldline/v1/identities.tsv")
        .lines()
        .nth(2)
        .ok_or_else(|| GoldenError::violation("identity fixture row is absent"))?;
    let malformed = first.replacen("\t1\t0\t", "\tbad\t0\t", 1);
    let table = format!(
        "# keep.golden-file-worldline.identities/v1\n\
         case\tsource_kind\tsource_parameter\trepetitions\tlogical_length\tcanonical_text\tcanonical_binary_hex\n\
         {first}\n{malformed}\n"
    );
    let path = root.join("identities.tsv");
    fs::write(&path, table)
        .map_err(|source| GoldenError::io("write identity test corpus", &path, source))?;

    let result = check_identities(&Corpus::open(root.clone())?, &InProcessOracle);
    let refused = matches!(
        result,
        Err(GoldenError::Violation(ref message))
            if message == "identities.tsv: duplicate identifier \"empty\""
    );
    fs::remove_dir_all(&root)
        .map_err(|source| GoldenError::io("remove identity test corpus", &root, source))?;

    assert!(refused);
    Ok(())
}
