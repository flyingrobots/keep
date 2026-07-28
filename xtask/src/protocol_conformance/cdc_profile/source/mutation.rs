//! This module owns CDC source edit admission and deterministic application.

use std::collections::BTreeMap;

use xtask::protocol_admission::EmptyHex;

use super::{MAX_SOURCE_BYTES, admit_aggregate, require_declared};
use crate::protocol_conformance::ConformanceError;
use crate::protocol_conformance::canonical::{case_name, decimal, lower_hex};
use crate::protocol_conformance::corpus::{Corpus, TablePolicy, TableRow};

const MAX_EDIT_BYTES: usize = 4_096;
const COLUMNS: [&str; 7] = [
    "case",
    "base_case",
    "operation",
    "offset",
    "span_length",
    "value_hex",
    "logical_length",
];
const POLICY: TablePolicy = TablePolicy::new("keep.cdc-mutations/v1", &COLUMNS, 1_048_576, 256);
pub(super) const CASES: [&str; 4] = [
    "early-delete",
    "early-insert",
    "early-xor",
    "target-long-transition",
];

#[derive(Clone, Copy)]
struct MutationPlan<'a> {
    name: &'a str,
    operation: &'a str,
    base: &'a [u8],
    offset: usize,
    end: usize,
    value: &'a [u8],
}

pub(super) fn load(
    corpus: &Corpus,
    values: &mut BTreeMap<String, Vec<u8>>,
    aggregate: &mut usize,
) -> Result<(), ConformanceError> {
    for row in corpus.rows("mutations.tsv", POLICY)? {
        let name = case_name(row.field("case")?, "mutations.tsv")?;
        if values.contains_key(name) {
            return Err(ConformanceError::violation(format!(
                "mutations.tsv: duplicate case {name:?}"
            )));
        }
        if !CASES.contains(&name) {
            return Err(ConformanceError::violation(format!(
                "mutations.tsv: case is outside the required exact set: {name:?}"
            )));
        }
        let content = reconstruct(values, &row)?;
        admit_aggregate(aggregate, content.len())?;
        values.insert(name.to_owned(), content);
    }
    Ok(())
}

fn reconstruct(
    values: &BTreeMap<String, Vec<u8>>,
    row: &TableRow,
) -> Result<Vec<u8>, ConformanceError> {
    let name = row.field("case")?;
    let base_name = case_name(row.field("base_case")?, "mutations.tsv")?;
    let base = values.get(base_name).ok_or_else(|| {
        ConformanceError::violation(format!("{name}: mutation base is absent: {base_name}"))
    })?;
    let offset = decimal(row.field("offset")?, &format!("{name} offset"), base.len())?;
    let span = decimal(
        row.field("span_length")?,
        &format!("{name} span"),
        MAX_EDIT_BYTES,
    )?;
    let declared = decimal(
        row.field("logical_length")?,
        &format!("{name} length"),
        MAX_SOURCE_BYTES,
    )?;
    let value = mutation_value(row, name)?;
    let end = offset
        .checked_add(span)
        .ok_or_else(|| ConformanceError::violation(format!("{name}: edit offset overflow")))?;
    if end > base.len() {
        return Err(ConformanceError::violation(format!(
            "{name}: edit is outside its bounded base"
        )));
    }
    let content = apply(MutationPlan {
        name,
        operation: row.field("operation")?,
        base,
        offset,
        end,
        value: &value,
    })?;
    require_declared(name, declared, content)
}

fn mutation_value(row: &TableRow, name: &str) -> Result<Vec<u8>, ConformanceError> {
    if row.field("value_hex")? == "-" {
        Ok(Vec::new())
    } else {
        lower_hex(
            row.field("value_hex")?,
            &format!("{name} value"),
            MAX_EDIT_BYTES,
            EmptyHex::Refuse,
        )
    }
}

fn apply(plan: MutationPlan<'_>) -> Result<Vec<u8>, ConformanceError> {
    let MutationPlan {
        name,
        operation,
        base,
        offset,
        end,
        value,
    } = plan;
    let width = end
        .checked_sub(offset)
        .ok_or_else(|| ConformanceError::violation(format!("{name}: mutation span underflow")))?;
    let capacity = base
        .len()
        .checked_add(value.len())
        .ok_or_else(|| ConformanceError::violation(format!("{name}: mutation length overflow")))?
        .min(MAX_SOURCE_BYTES);
    let mut content = Vec::with_capacity(capacity);
    content.extend_from_slice(base.get(..offset).ok_or_else(|| {
        ConformanceError::violation(format!("{name}: mutation prefix is outside its base"))
    })?);
    match operation {
        "insert-v1" if end == offset && !value.is_empty() => content.extend_from_slice(value),
        "delete-v1" if end > offset && value.is_empty() => {}
        "xor-v1" if end > offset && value.len() == width => {
            let span = base.get(offset..end).ok_or_else(|| {
                ConformanceError::violation(format!("{name}: mutation span is outside its base"))
            })?;
            content.extend(span.iter().zip(value).map(|(left, right)| left ^ right));
        }
        _ => {
            return Err(ConformanceError::violation(format!(
                "{name}: malformed mutation {operation:?}"
            )));
        }
    }
    content.extend_from_slice(base.get(end..).ok_or_else(|| {
        ConformanceError::violation(format!("{name}: mutation suffix is outside its base"))
    })?);
    Ok(content)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;

    use super::{ConformanceError, Corpus, MutationPlan, apply, load};
    use crate::test_directory::TestDirectory;

    #[test]
    fn edit_operations_preserve_exact_byte_coordinates() {
        assert!(matches!(
            apply(MutationPlan {
                name: "insert",
                operation: "insert-v1",
                base: b"abcd",
                offset: 2,
                end: 2,
                value: b"XY",
            }),
            Ok(ref bytes) if bytes == b"abXYcd"
        ));
        assert!(matches!(
            apply(MutationPlan {
                name: "delete",
                operation: "delete-v1",
                base: b"abcd",
                offset: 1,
                end: 3,
                value: b"",
            }),
            Ok(ref bytes) if bytes == b"ad"
        ));
        assert!(matches!(
            apply(MutationPlan {
                name: "xor",
                operation: "xor-v1",
                base: b"\x00\x0f",
                offset: 0,
                end: 2,
                value: b"\xff\xf0",
            }),
            Ok(ref bytes) if bytes == b"\xff\xff"
        ));
    }

    #[test]
    fn mutation_grammar_refuses_no_op_edits() {
        assert!(matches!(
            apply(MutationPlan {
                name: "empty",
                operation: "insert-v1",
                base: b"base",
                offset: 1,
                end: 1,
                value: b"",
            }),
            Err(ConformanceError::Violation(ref message))
                if message == "empty: malformed mutation \"insert-v1\""
        ));
    }

    #[test]
    fn mutation_case_set_precedes_mutation_semantics() -> Result<(), ConformanceError> {
        let directory = TestDirectory::create("cdc-mutation-case-set").map_err(|source| {
            ConformanceError::io("create mutation test corpus", "temporary", source)
        })?;
        let root = directory.path().to_owned();
        let path = root.join("mutations.tsv");
        fs::write(
            &path,
            "keep.cdc-mutations/v1\n\
             case\tbase_case\toperation\toffset\tspan_length\tvalue_hex\tlogical_length\n\
             rogue\tedit-base\tunsupported-v1\t0\t0\t-\t4\n",
        )
        .map_err(|source| ConformanceError::io("write mutation test corpus", &path, source))?;
        let corpus = Corpus::open(root.clone())?;
        let mut values = BTreeMap::from([(String::from("edit-base"), b"base".to_vec())]);
        let mut aggregate = 0;

        let result = load(&corpus, &mut values, &mut aggregate);
        assert!(matches!(
            result,
            Err(ConformanceError::Violation(ref message))
                if message == "mutations.tsv: case is outside the required exact set: \"rogue\""
        ));
        directory
            .close()
            .map_err(|source| ConformanceError::io("remove mutation test corpus", &root, source))?;
        Ok(())
    }
}
