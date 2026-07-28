//! Repository-owned verification and maintenance commands.

#![deny(warnings)]
#![forbid(unsafe_code)]

#[allow(
    clippy::redundant_pub_crate,
    reason = "the command dispatcher owns this private repository task"
)]
mod benchmark_baseline;
mod diagnostic;
#[allow(
    clippy::redundant_pub_crate,
    reason = "the command and task-error boundaries are sibling consumers"
)]
mod fuzz_campaign;
#[allow(
    clippy::redundant_pub_crate,
    reason = "the parent command dispatcher is the only consumer"
)]
mod fuzz_seed_corpus;
#[allow(
    clippy::redundant_pub_crate,
    reason = "the parent command dispatcher is the only consumer"
)]
mod golden_file_worldline;
#[allow(
    clippy::redundant_pub_crate,
    reason = "bounded process output is shared by sibling adapters"
)]
mod process_output;
#[allow(
    clippy::redundant_pub_crate,
    reason = "the command and task-error boundaries are sibling consumers"
)]
mod protocol_conformance;
#[allow(
    clippy::redundant_pub_crate,
    reason = "the parent command dispatcher is the only consumer"
)]
mod source_structure;
#[allow(
    clippy::redundant_pub_crate,
    reason = "the parent command dispatcher is the only consumer"
)]
mod task_error;
#[cfg(test)]
#[allow(
    clippy::redundant_pub_crate,
    reason = "scoped test directories are shared by sibling test modules"
)]
mod test_directory;

use std::env;
use std::ffi::OsString;
use std::io;
use std::path::Path;

use task_error::TaskError;

fn main() -> Result<(), TaskError> {
    run(env::args_os().skip(1))
}

fn run(mut arguments: impl Iterator<Item = OsString>) -> Result<(), TaskError> {
    let command = arguments
        .next()
        .ok_or(TaskError::Usage)?
        .into_string()
        .map_err(|_| TaskError::InvalidCommandEncoding)?;
    let repository_root = repository_root()?;
    if command == "fuzz" {
        let stdout = io::stdout();
        let mut output = stdout.lock();
        fuzz_campaign::run(repository_root, arguments, &mut output)?;
        return Ok(());
    }
    refuse_extra(&mut arguments)?;
    match command.as_str() {
        "benchmark-baseline" => {
            benchmark_baseline::run(repository_root)?;
        }
        "cdc-profile-conformance-check" => {
            protocol_conformance::check_cdc_profile(repository_root)?;
        }
        "chunk-id-conformance-check" => {
            protocol_conformance::check_chunk_identity(repository_root)?;
        }
        "conformance-check" => {
            protocol_conformance::check(repository_root)?;
        }
        "prepare-fuzz-corpus" => {
            fuzz_seed_corpus::prepare(repository_root)?;
        }
        "golden-file-worldline-check" => {
            golden_file_worldline::check(repository_root)?;
        }
        "source-structure-check" => {
            source_structure::check(repository_root)?;
        }
        "verify" => {
            golden_file_worldline::check(repository_root)?;
            protocol_conformance::check(repository_root)?;
            source_structure::check(repository_root)?;
        }
        _ => return Err(TaskError::UnknownCommand(command)),
    }
    Ok(())
}

fn refuse_extra(arguments: &mut impl Iterator<Item = OsString>) -> Result<(), TaskError> {
    if let Some(extra) = arguments.next() {
        let extra = extra
            .into_string()
            .map_err(|_| TaskError::InvalidExtraArgumentEncoding)?;
        return Err(TaskError::UnexpectedArgument(extra));
    }
    Ok(())
}

fn repository_root() -> Result<&'static Path, TaskError> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or(TaskError::RepositoryRoot)
}
