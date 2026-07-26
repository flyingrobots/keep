use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Read, Take};
use std::path::{Path, PathBuf};

use super::GoldenError;

pub(super) const MAX_SOURCE_BYTES: usize = 1_048_576;
pub(super) const MAX_MUTATION_VALUE_BYTES: usize = 64;
pub(super) const U16_MAX: u64 = 65_535;

const MAX_TABLE_BYTES: usize = 1_048_576;

pub(super) struct Corpus {
    root: PathBuf,
}

impl Corpus {
    pub(super) const fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub(super) fn rows(
        &self,
        table: &'static str,
        schema: &str,
        columns: &[&'static str],
    ) -> Result<Vec<TableRow>, GoldenError> {
        let path = self.root.join(table);
        let lines = protocol_lines(&path)?;
        table_rows(table, schema, columns, lines)
    }

    pub(super) fn source_path(&self, parameter: &str) -> Result<PathBuf, GoldenError> {
        let relative = protocol_source_path(parameter)?;
        let root = fs::canonicalize(&self.root)
            .map_err(|source| GoldenError::io("canonicalize corpus root", &self.root, source))?;
        let unresolved = self.root.join(&relative);
        let path = fs::canonicalize(&unresolved)
            .map_err(|source| GoldenError::io("canonicalize corpus source", &unresolved, source))?;
        let metadata = fs::metadata(&path)
            .map_err(|source| GoldenError::io("inspect source", &path, source))?;
        if path == root || !path.starts_with(&root) || !metadata.is_file() {
            return Err(GoldenError::violation(format!(
                "source is outside the corpus or is not a file: {parameter}"
            )));
        }
        Ok(path)
    }
}

fn protocol_source_path(parameter: &str) -> Result<PathBuf, GoldenError> {
    if parameter.is_empty()
        || parameter.contains('\\')
        || parameter.contains(':')
        || parameter.contains('\0')
    {
        return Err(GoldenError::violation(format!(
            "unsafe source path: {parameter}"
        )));
    }
    let mut relative = PathBuf::new();
    for segment in parameter.split('/') {
        if segment.is_empty() || matches!(segment, "." | "..") {
            return Err(GoldenError::violation(format!(
                "unsafe source path: {parameter}"
            )));
        }
        relative.push(segment);
    }
    Ok(relative)
}

fn table_rows(
    table: &str,
    schema: &str,
    columns: &[&'static str],
    lines: Vec<String>,
) -> Result<Vec<TableRow>, GoldenError> {
    let mut lines = lines.into_iter();
    if lines.next().as_deref() != Some(schema) {
        return Err(GoldenError::violation(format!(
            "{table}: unsupported schema or empty table"
        )));
    }
    let observed_columns = lines.next().ok_or_else(|| {
        GoldenError::violation(format!("{table}: unsupported schema or empty table"))
    })?;
    if observed_columns.split('\t').ne(columns.iter().copied()) {
        return Err(GoldenError::violation(format!(
            "{table}: unexpected columns"
        )));
    }
    let rows = lines
        .enumerate()
        .map(|(offset, line)| {
            let line_number = offset
                .checked_add(3)
                .ok_or_else(|| GoldenError::violation(format!("{table}: row number overflow")))?;
            TableRow::parse(table, line_number, &line, columns)
        })
        .collect::<Result<Vec<_>, _>>()?;
    if rows.is_empty() {
        Err(GoldenError::violation(format!(
            "{table}: table has no data rows"
        )))
    } else {
        Ok(rows)
    }
}

pub(super) struct TableRow {
    fields: BTreeMap<&'static str, String>,
}

impl TableRow {
    fn parse(
        table: &str,
        line_number: usize,
        line: &str,
        columns: &[&'static str],
    ) -> Result<Self, GoldenError> {
        let values = line.split('\t').collect::<Vec<_>>();
        if values.len() != columns.len() {
            return Err(GoldenError::violation(format!(
                "{table}:{line_number}: malformed field count"
            )));
        }
        let fields = columns
            .iter()
            .copied()
            .zip(values.into_iter().map(str::to_owned))
            .collect();
        Ok(Self { fields })
    }

    pub(super) fn field(&self, name: &'static str) -> Result<&str, GoldenError> {
        self.fields.get(name).map(String::as_str).ok_or_else(|| {
            GoldenError::violation(format!("checker schema omitted required field {name}"))
        })
    }
}

pub(super) fn bounded_file_bytes(
    path: &Path,
    maximum: usize,
    label: &str,
) -> Result<Vec<u8>, GoldenError> {
    let file = File::open(path).map_err(|source| GoldenError::io("open", path, source))?;
    let expected = file
        .metadata()
        .map_err(|source| GoldenError::io("inspect", path, source))?
        .len();
    let maximum_u64 = u64::try_from(maximum).map_err(|source| {
        GoldenError::violation(format!(
            "{label}: platform bound cannot be represented: {source}"
        ))
    })?;
    if expected > maximum_u64 {
        return Err(GoldenError::violation(format!(
            "{label}: file exceeds {maximum} bytes"
        )));
    }
    let read_limit = maximum_u64
        .checked_add(1)
        .ok_or_else(|| GoldenError::violation(format!("{label}: read bound overflow")))?;
    let mut content = Vec::new();
    let mut bounded: Take<File> = file.take(read_limit);
    bounded
        .read_to_end(&mut content)
        .map_err(|source| GoldenError::io("read", path, source))?;
    let observed = u64::try_from(content.len()).map_err(|source| {
        GoldenError::violation(format!(
            "{label}: observed size cannot be represented: {source}"
        ))
    })?;
    if observed != expected || observed > maximum_u64 {
        return Err(GoldenError::violation(format!(
            "{label}: file size changed while reading"
        )));
    }
    Ok(content)
}

fn protocol_lines(path: &Path) -> Result<Vec<String>, GoldenError> {
    let raw = bounded_file_bytes(path, MAX_TABLE_BYTES, &display_name(path))?;
    let table = display_name(path);
    if raw.is_empty() || !raw.ends_with(b"\n") || raw.contains(&b'\r') {
        return Err(GoldenError::violation(format!(
            "{table}: protocol must use final-LF-only framing"
        )));
    }
    let text = String::from_utf8(raw).map_err(|source| GoldenError::Utf8 {
        path: path.to_path_buf(),
        source,
    })?;
    let framed = text.strip_suffix('\n').ok_or_else(|| {
        GoldenError::violation(format!("{table}: protocol must use final-LF-only framing"))
    })?;
    let lines = framed.split('\n').map(str::to_owned).collect::<Vec<_>>();
    if lines.iter().any(String::is_empty) {
        return Err(GoldenError::violation(format!(
            "{table}: protocol contains a blank line"
        )));
    }
    Ok(lines)
}

fn display_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map_or_else(|| path.display().to_string(), str::to_owned)
}

#[cfg(test)]
mod tests;
