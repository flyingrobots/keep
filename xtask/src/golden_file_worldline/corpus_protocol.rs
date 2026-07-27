//! This module owns bounded TSV framing, row admission, and corpus file paths.

use std::collections::BTreeMap;
#[cfg(feature = "repository-tasks")]
use std::fs::File;
#[cfg(feature = "repository-tasks")]
use std::io::{Read, Take};
use std::path::{Path, PathBuf};

#[cfg(feature = "repository-tasks")]
use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
#[cfg(feature = "repository-tasks")]
use cap_std::ambient_authority;
#[cfg(feature = "repository-tasks")]
use cap_std::fs::{Dir, OpenOptions};

use super::GoldenError;
use xtask::protocol_admission::{FramedLinesError, framed_lines, posix_relative_path, tab_fields};

pub(super) const MAX_SOURCE_BYTES: usize = 1_048_576;
pub(super) const MAX_MUTATION_VALUE_BYTES: usize = 64;
pub(super) const U16_MAX: u64 = 65_535;

const MAX_TABLE_BYTES: usize = 1_048_576;

#[cfg(feature = "repository-tasks")]
pub(super) struct Corpus {
    directory: Dir,
    root: PathBuf,
}

#[cfg(feature = "repository-tasks")]
impl Corpus {
    pub(super) fn open(root: PathBuf) -> Result<Self, GoldenError> {
        let directory = Dir::open_ambient_dir(&root, ambient_authority())
            .map_err(|source| GoldenError::io("open corpus root", &root, source))?;
        Ok(Self { directory, root })
    }

    pub(super) fn rows(
        &self,
        table: &'static str,
        schema: &str,
        columns: &[&'static str],
    ) -> Result<Vec<TableRow>, GoldenError> {
        let raw = self
            .open_table(Path::new(table), table)?
            .bounded_bytes(MAX_TABLE_BYTES, table)?;
        let lines = protocol_lines_from_bytes(Path::new(table), &raw)?;
        table_rows(table, schema, columns, lines)
    }

    pub(super) fn source_file(&self, parameter: &str) -> Result<CorpusFile, GoldenError> {
        let relative = protocol_source_path(parameter)?;
        self.open_source(&relative, parameter)
    }

    fn open_table(&self, relative: &Path, label: &str) -> Result<CorpusFile, GoldenError> {
        self.open_file(
            relative,
            label,
            "open corpus table",
            &nofollow_read_options(),
        )
    }

    fn open_source(&self, relative: &Path, label: &str) -> Result<CorpusFile, GoldenError> {
        self.open_file(
            relative,
            label,
            "open corpus source",
            &nonblocking_read_options(),
        )
    }

    fn open_file(
        &self,
        relative: &Path,
        label: &str,
        action: &'static str,
        options: &OpenOptions,
    ) -> Result<CorpusFile, GoldenError> {
        let path = self.root.join(relative);
        let file = self
            .directory
            .open_with(relative, options)
            .map(cap_std::fs::File::into_std)
            .map_err(|source| GoldenError::io(action, &path, source))?;
        let metadata = file
            .metadata()
            .map_err(|source| GoldenError::io("inspect corpus entry", &path, source))?;
        if !metadata.is_file() {
            return Err(GoldenError::violation(format!(
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

#[cfg(feature = "repository-tasks")]
pub(super) struct CorpusFile {
    expected: u64,
    file: File,
    path: PathBuf,
}

#[cfg(feature = "repository-tasks")]
impl CorpusFile {
    pub(super) const fn len(&self) -> u64 {
        self.expected
    }

    pub(super) fn bounded_bytes(self, maximum: usize, label: &str) -> Result<Vec<u8>, GoldenError> {
        bounded_reader_bytes(self.file, self.expected, &self.path, maximum, label)
    }
}

#[cfg(feature = "repository-tasks")]
fn nonblocking_read_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.read(true).nonblock(true);
    options
}

#[cfg(feature = "repository-tasks")]
fn nofollow_read_options() -> OpenOptions {
    let mut options = nonblocking_read_options();
    options.follow(FollowSymlinks::No);
    options
}

fn protocol_source_path(parameter: &str) -> Result<PathBuf, GoldenError> {
    posix_relative_path(parameter)
        .map_err(|_| GoldenError::violation(format!("unsafe source path: {parameter}")))
}

pub(super) fn table_rows(
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
        let values = tab_fields(line, columns.len()).map_err(|_| {
            GoldenError::violation(format!("{table}:{line_number}: malformed field count"))
        })?;
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

#[cfg(feature = "repository-tasks")]
fn bounded_reader_bytes(
    file: impl Read,
    expected: u64,
    path: &Path,
    maximum: usize,
    label: &str,
) -> Result<Vec<u8>, GoldenError> {
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
    let mut bounded: Take<_> = file.take(read_limit);
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

pub(super) fn protocol_lines_from_bytes(
    path: &Path,
    raw: &[u8],
) -> Result<Vec<String>, GoldenError> {
    let table = display_name(path);
    match framed_lines(raw, MAX_TABLE_BYTES) {
        Ok(lines) => Ok(lines),
        Err(FramedLinesError::Utf8(source)) => Err(GoldenError::Utf8 {
            path: path.to_path_buf(),
            source,
        }),
        Err(FramedLinesError::BlankLine) => Err(GoldenError::violation(format!(
            "{table}: protocol contains a blank line"
        ))),
        Err(FramedLinesError::ExceedsMaximum { maximum }) => Err(GoldenError::violation(format!(
            "{table}: file exceeds {maximum} bytes"
        ))),
        Err(FramedLinesError::FinalLfOnly) => Err(GoldenError::violation(format!(
            "{table}: protocol must use final-LF-only framing"
        ))),
    }
}

fn display_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map_or_else(|| path.display().to_string(), str::to_owned)
}

#[cfg(all(test, feature = "repository-tasks"))]
#[path = "corpus_protocol/tests.rs"]
mod tests;
