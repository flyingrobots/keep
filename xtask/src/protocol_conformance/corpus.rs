//! This module owns bounded, capability-relative conformance corpus admission.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Take};
use std::path::{Path, PathBuf};

use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions};
use xtask::protocol_admission::{FramedLinesError, framed_lines, posix_relative_path, tab_fields};

use super::ConformanceError;

pub(super) struct Corpus {
    directory: Dir,
    root: PathBuf,
}

#[derive(Clone, Copy)]
pub(super) struct TablePolicy {
    schema: &'static str,
    columns: &'static [&'static str],
    maximum_bytes: usize,
    maximum_rows: usize,
}

impl TablePolicy {
    pub(super) const fn new(
        schema: &'static str,
        columns: &'static [&'static str],
        maximum_bytes: usize,
        maximum_rows: usize,
    ) -> Self {
        Self {
            schema,
            columns,
            maximum_bytes,
            maximum_rows,
        }
    }
}

impl Corpus {
    pub(super) fn open(root: PathBuf) -> Result<Self, ConformanceError> {
        let directory = Dir::open_ambient_dir(&root, ambient_authority())
            .map_err(|source| ConformanceError::io("open corpus root", &root, source))?;
        Ok(Self { directory, root })
    }

    pub(super) fn rows(
        &self,
        table: &'static str,
        policy: TablePolicy,
    ) -> Result<Vec<TableRow>, ConformanceError> {
        let raw = self
            .open_file(Path::new(table), table)?
            .bounded_bytes(policy.maximum_bytes, table)?;
        let lines = protocol_lines(Path::new(table), &raw, policy.maximum_bytes)?;
        parse_rows(
            table,
            policy.schema,
            policy.columns,
            lines,
            policy.maximum_rows,
        )
    }

    pub(super) fn source_file(&self, parameter: &str) -> Result<CorpusFile, ConformanceError> {
        let relative = posix_relative_path(parameter).map_err(|source| ConformanceError::Path {
            parameter: parameter.to_owned(),
            source,
        })?;
        self.open_file(&relative, parameter)
    }

    fn open_file(&self, relative: &Path, label: &str) -> Result<CorpusFile, ConformanceError> {
        let path = self.root.join(relative);
        let file = self
            .directory
            .open_with(relative, &nofollow_read_options())
            .map(cap_std::fs::File::into_std)
            .map_err(|source| ConformanceError::io("open corpus entry", &path, source))?;
        let metadata = file
            .metadata()
            .map_err(|source| ConformanceError::io("inspect corpus entry", &path, source))?;
        if !metadata.is_file() {
            return Err(ConformanceError::violation(format!(
                "corpus entry is not a regular file: {label}"
            )));
        }
        Ok(CorpusFile {
            expected: metadata.len(),
            file,
            path,
        })
    }
}

pub(super) struct CorpusFile {
    expected: u64,
    file: File,
    path: PathBuf,
}

impl CorpusFile {
    pub(super) fn bounded_bytes(
        self,
        maximum: usize,
        label: &str,
    ) -> Result<Vec<u8>, ConformanceError> {
        let maximum_u64 = u64::try_from(maximum).map_err(|source| {
            ConformanceError::violation(format!(
                "{label}: platform bound cannot be represented: {source}"
            ))
        })?;
        if self.expected > maximum_u64 {
            return Err(ConformanceError::violation(format!(
                "{label}: file exceeds {maximum} bytes"
            )));
        }
        let limit = maximum_u64
            .checked_add(1)
            .ok_or_else(|| ConformanceError::violation(format!("{label}: read bound overflow")))?;
        read_bounded(self.file, self.expected, &self.path, limit, label)
    }
}

pub(super) struct TableRow {
    fields: BTreeMap<&'static str, String>,
}

impl TableRow {
    pub(super) fn field(&self, name: &'static str) -> Result<&str, ConformanceError> {
        self.fields.get(name).map(String::as_str).ok_or_else(|| {
            ConformanceError::violation(format!("checker schema omitted required field {name}"))
        })
    }
}

fn nofollow_read_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.read(true).nonblock(true).follow(FollowSymlinks::No);
    options
}

fn read_bounded(
    file: impl Read,
    expected: u64,
    path: &Path,
    limit: u64,
    label: &str,
) -> Result<Vec<u8>, ConformanceError> {
    let mut content = Vec::new();
    let mut bounded: Take<_> = file.take(limit);
    bounded
        .read_to_end(&mut content)
        .map_err(|source| ConformanceError::io("read corpus entry", path, source))?;
    let observed = u64::try_from(content.len()).map_err(|source| {
        ConformanceError::violation(format!(
            "{label}: observed size cannot be represented: {source}"
        ))
    })?;
    if observed != expected || observed >= limit {
        return Err(ConformanceError::violation(format!(
            "{label}: file size changed or exceeded its bound while reading"
        )));
    }
    Ok(content)
}

fn protocol_lines(
    path: &Path,
    raw: &[u8],
    maximum: usize,
) -> Result<Vec<String>, ConformanceError> {
    match framed_lines(raw, maximum) {
        Ok(lines) => Ok(lines),
        Err(FramedLinesError::Utf8(source)) => Err(ConformanceError::Utf8 {
            path: path.to_path_buf(),
            source,
        }),
        Err(FramedLinesError::BlankLine) => Err(ConformanceError::violation(format!(
            "{}: protocol contains a blank line",
            path.display()
        ))),
        Err(FramedLinesError::ExceedsMaximum { maximum }) => Err(ConformanceError::violation(
            format!("{}: file exceeds {maximum} bytes", path.display()),
        )),
        Err(FramedLinesError::FinalLfOnly) => Err(ConformanceError::violation(format!(
            "{}: protocol must use final-LF-only framing",
            path.display()
        ))),
    }
}

fn parse_rows(
    table: &'static str,
    schema: &str,
    columns: &[&'static str],
    lines: Vec<String>,
    maximum_rows: usize,
) -> Result<Vec<TableRow>, ConformanceError> {
    let mut lines = lines.into_iter();
    if lines.next().as_deref() != Some(schema) {
        return Err(ConformanceError::violation(format!(
            "{table}: unsupported schema or empty table"
        )));
    }
    let observed_columns = lines.next().ok_or_else(|| {
        ConformanceError::violation(format!("{table}: unsupported schema or empty table"))
    })?;
    if observed_columns.split('\t').ne(columns.iter().copied()) {
        return Err(ConformanceError::violation(format!(
            "{table}: unexpected columns"
        )));
    }
    let rows = lines
        .enumerate()
        .map(|(offset, line)| parse_row(table, offset, &line, columns))
        .collect::<Result<Vec<_>, _>>()?;
    if rows.is_empty() || rows.len() > maximum_rows {
        return Err(ConformanceError::violation(format!(
            "{table}: row count is outside its bound"
        )));
    }
    Ok(rows)
}

fn parse_row(
    table: &'static str,
    offset: usize,
    line: &str,
    columns: &[&'static str],
) -> Result<TableRow, ConformanceError> {
    let line_number = offset
        .checked_add(3)
        .ok_or_else(|| ConformanceError::violation(format!("{table}: row number overflow")))?;
    let values = tab_fields(line, columns.len()).map_err(|_| {
        ConformanceError::violation(format!("{table}:{line_number}: malformed field count"))
    })?;
    if values.contains(&"") {
        return Err(ConformanceError::violation(format!(
            "{table}:{line_number}: empty field"
        )));
    }
    let fields = columns
        .iter()
        .copied()
        .zip(values.into_iter().map(str::to_owned))
        .collect();
    Ok(TableRow { fields })
}
