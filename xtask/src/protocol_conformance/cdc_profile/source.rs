//! This module owns bounded CDC source recipes and mutation reconstruction.

mod mutation;

use std::collections::{BTreeMap, BTreeSet};

use crate::protocol_conformance::ConformanceError;
use crate::protocol_conformance::canonical::{case_name, decimal, exact_hex};
use crate::protocol_conformance::corpus::{Corpus, TablePolicy, TableRow};

const MAX_INPUT_BYTES: usize = 4_096;
const MAX_SOURCE_BYTES: usize = 2_097_152;
const SOURCE_CASES: [&str; 20] = [
    "alternating-long",
    "edit-base",
    "empty",
    "ff-long",
    "max-exact",
    "max-minus-one",
    "max-plus-one",
    "min-exact",
    "min-minus-one",
    "min-plus-one",
    "natural-cut-runt",
    "probe-byte-carry",
    "random-long",
    "short-mask-match",
    "source-like",
    "target-exact",
    "target-minus-one",
    "target-plus-one",
    "tiny",
    "zeros-long",
];
const SOURCE_COLUMNS: [&str; 5] = ["case", "recipe", "parameter", "count", "logical_length"];
const SOURCE_POLICY: TablePolicy =
    TablePolicy::new("keep.cdc-sources/v1", &SOURCE_COLUMNS, 1_048_576, 256);
const TOTAL_CASES: usize = SOURCE_CASES.len() + mutation::CASES.len();

pub(super) struct Sources {
    values: BTreeMap<String, Vec<u8>>,
}

impl Sources {
    pub(super) fn get(&self, name: &str) -> Result<&[u8], ConformanceError> {
        self.values.get(name).map(Vec::as_slice).ok_or_else(|| {
            ConformanceError::violation(format!("CDC source case is absent: {name}"))
        })
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = (&str, &[u8])> {
        self.values
            .iter()
            .map(|(name, source)| (name.as_str(), source.as_slice()))
    }

    pub(super) fn names(&self) -> BTreeSet<&str> {
        self.values.keys().map(String::as_str).collect()
    }
}

pub(super) fn load(corpus: &Corpus) -> Result<Sources, ConformanceError> {
    let mut values = BTreeMap::new();
    let mut aggregate = 0_usize;
    for row in corpus.rows("sources.tsv", SOURCE_POLICY)? {
        let name = case_name(row.field("case")?, "sources.tsv")?;
        if values.contains_key(name) {
            return Err(ConformanceError::violation(format!(
                "sources.tsv: duplicate case {name:?}"
            )));
        }
        let content = primitive_source(corpus, &row)?;
        admit_aggregate(&mut aggregate, content.len())?;
        values.insert(name.to_owned(), content);
    }
    require_names(&values, &SOURCE_CASES, "sources.tsv")?;
    mutation::load(corpus, &mut values, &mut aggregate)?;
    require_names(&values, &required_cases(), "CDC corpus")?;
    Ok(Sources { values })
}

fn primitive_source(corpus: &Corpus, row: &TableRow) -> Result<Vec<u8>, ConformanceError> {
    let name = row.field("case")?;
    let count = decimal(
        row.field("count")?,
        &format!("{name} count"),
        MAX_SOURCE_BYTES,
    )?;
    let declared = decimal(
        row.field("logical_length")?,
        &format!("{name} length"),
        MAX_SOURCE_BYTES,
    )?;
    let content = match (row.field("recipe")?, row.field("parameter")?, count) {
        ("empty-v1", "-", 0) => Vec::new(),
        ("repeated-byte-v1", parameter, _) => repeat_pattern(
            &exact_hex(parameter, &format!("{name} byte"), 1)?,
            count,
            name,
        )?,
        ("alternating-v1", parameter, _) => repeat_pattern(
            &exact_hex(parameter, &format!("{name} pattern"), 2)?,
            count,
            name,
        )?,
        ("xorshift64-v1", parameter, _) => xorshift64(parameter, count, name)?,
        ("file-repeat-v1", parameter, _) => {
            let unit = corpus
                .source_file(parameter)?
                .bounded_bytes(MAX_INPUT_BYTES, name)?;
            if unit.is_empty() {
                return Err(ConformanceError::violation(format!(
                    "{name}: repeated file is empty"
                )));
            }
            repeat_pattern(&unit, count, name)?
        }
        (recipe, _, _) => {
            return Err(ConformanceError::violation(format!(
                "{name}: unsupported or malformed recipe {recipe:?}"
            )));
        }
    };
    require_declared(name, declared, content)
}

fn repeat_pattern(pattern: &[u8], count: usize, name: &str) -> Result<Vec<u8>, ConformanceError> {
    let length = pattern
        .len()
        .checked_mul(count)
        .ok_or_else(|| ConformanceError::violation(format!("{name}: repeated length overflow")))?;
    if length > MAX_SOURCE_BYTES {
        return Err(ConformanceError::violation(format!(
            "{name}: repeated pattern exceeds source bound"
        )));
    }
    Ok(pattern.repeat(count))
}

fn xorshift64(parameter: &str, count: usize, name: &str) -> Result<Vec<u8>, ConformanceError> {
    let seed = exact_hex(parameter, &format!("{name} seed"), 8)?;
    let mut state = u64::from_be_bytes(
        seed.try_into()
            .map_err(|_| ConformanceError::violation(format!("{name}: seed width moved")))?,
    );
    if state == 0 {
        return Err(ConformanceError::violation(
            "xorshift64-v1 seed must be nonzero",
        ));
    }
    let mut output = Vec::with_capacity(count);
    for _ in 0..count {
        state ^= state.wrapping_shl(13);
        state ^= state >> 7;
        state ^= state.wrapping_shl(17);
        output.push(state.to_le_bytes()[0]);
    }
    Ok(output)
}

pub(super) fn require_declared(
    name: &str,
    declared: usize,
    content: Vec<u8>,
) -> Result<Vec<u8>, ConformanceError> {
    if content.len() != declared {
        return Err(ConformanceError::violation(format!(
            "{name}: declared {declared} bytes, generated {}",
            content.len()
        )));
    }
    Ok(content)
}

pub(super) fn admit_aggregate(aggregate: &mut usize, added: usize) -> Result<(), ConformanceError> {
    *aggregate = aggregate
        .checked_add(added)
        .ok_or_else(|| ConformanceError::violation("source corpus byte count overflow"))?;
    let maximum = TOTAL_CASES
        .checked_mul(MAX_SOURCE_BYTES)
        .ok_or_else(|| ConformanceError::violation("source corpus bound overflow"))?;
    if *aggregate > maximum {
        return Err(ConformanceError::violation(
            "source corpus exceeds its aggregate byte bound",
        ));
    }
    Ok(())
}

fn required_cases() -> Vec<&'static str> {
    SOURCE_CASES.into_iter().chain(mutation::CASES).collect()
}

fn require_names(
    values: &BTreeMap<String, Vec<u8>>,
    required: &[&str],
    table: &str,
) -> Result<(), ConformanceError> {
    let observed = values.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let required = required.iter().copied().collect::<BTreeSet<_>>();
    if observed == required {
        Ok(())
    } else {
        Err(ConformanceError::violation(format!(
            "{table}: case set differs from the required exact set"
        )))
    }
}
