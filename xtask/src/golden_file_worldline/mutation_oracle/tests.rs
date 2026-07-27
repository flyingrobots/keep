//! This module owns mutation-width and case-admission regression evidence.

use std::collections::BTreeMap;
use std::fs;

use super::{
    Corpus, GoldenError, IdentityFixture, apply_fixed_width, check, copy_fixed_width, expected_text,
};
use crate::golden_file_worldline::digest_port::IdentityDigestOracle;
use crate::golden_file_worldline::identity_oracle::digest;
use crate::test_directory::TestDirectory;

struct InProcessOracle;

impl IdentityDigestOracle for InProcessOracle {
    fn identity_digest(&self, payload: &[u8]) -> Result<[u8; 32], GoldenError> {
        Ok(*digest(payload)?.as_bytes())
    }
}

#[test]
fn fixed_width_mutations_require_the_declared_value_width() {
    for value in ["", "02", "000203"] {
        let mut changed = [0_u8; 2];
        let result = apply_fixed_width(&mut changed, "set-u16-be", 0, value, "set-version");
        assert!(matches!(
            result,
            Err(GoldenError::Violation(ref message))
                if message == "set-version: mutation value must be exactly 2 bytes"
        ));
    }
}

#[test]
fn fixed_width_mutations_admit_the_declared_value_width() {
    let mut changed = [0_u8; 2];
    let result = apply_fixed_width(&mut changed, "set-u16-be", 0, "0002", "set-version");
    assert!(result.is_ok());
    assert_eq!(changed, [0_u8, 2]);
}

#[test]
fn fixed_width_mutations_refuse_unknown_operations() {
    for operation in ["set-u32-be", "set-u64-le", "set-u8-typo"] {
        let mut changed = [0_u8; 4];
        let result = apply_fixed_width(&mut changed, operation, 0, "01", "set-value");
        assert!(matches!(
            result,
            Err(GoldenError::Violation(ref message))
                if message == &format!(
                    "set-value: unknown fixed-width mutation operation {operation:?}"
                )
        ));
    }
}

#[test]
fn fixed_width_copy_refuses_a_width_mismatch() {
    let mut destination = [0_u8; 2];
    let result = copy_fixed_width(&mut destination, &[1_u8], "set-value");
    assert!(matches!(
        result,
        Err(GoldenError::Violation(ref message))
            if message == "set-value: mutation value width does not match its destination"
    ));
}

#[test]
fn duplicate_mutation_precedes_malformed_mutation_semantics() -> Result<(), GoldenError> {
    let directory = TestDirectory::create("duplicate-mutation")
        .map_err(|source| GoldenError::io("create mutation test corpus", "temporary", source))?;
    let root = directory.path().to_owned();
    let first = include_str!("../../../../conformance/golden-file-worldline/v1/mutations.tsv")
        .lines()
        .nth(2)
        .ok_or_else(|| GoldenError::violation("mutation fixture row is absent"))?;
    let malformed = first.replacen("\tcontent\t", "\tinvalid\t", 1);
    let table = format!(
        "# keep.golden-file-worldline.mutations/v1\n\
         case\ttarget_kind\ttarget_case\toperation\toffset\tvalue_hex\texpected_outcome\n\
         {first}\n{malformed}\n"
    );
    let path = root.join("mutations.tsv");
    fs::write(&path, table)
        .map_err(|source| GoldenError::io("write mutation test corpus", &path, source))?;
    let content = vec![0_u8];
    let length = u64::try_from(content.len())
        .map_err(|source| GoldenError::violation(format!("test length is invalid: {source}")))?;
    let fixture = IdentityFixture {
        canonical_text: expected_text(length, &digest(&content)?),
        content,
        canonical_binary: Vec::new(),
    };
    let fixtures = BTreeMap::from([(String::from("state-a"), fixture)]);

    let result = check(&Corpus::open(root.clone())?, &fixtures, &InProcessOracle);
    let refused = matches!(
        result,
        Err(GoldenError::Violation(ref message))
            if message == "mutations.tsv: duplicate identifier \"content-first-bit\""
    );
    assert!(refused);
    directory
        .close()
        .map_err(|source| GoldenError::io("remove mutation test corpus", &root, source))?;
    Ok(())
}
