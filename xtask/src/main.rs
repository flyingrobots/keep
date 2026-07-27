//! Repository-owned verification and maintenance commands.

#![deny(warnings)]
#![forbid(unsafe_code)]

mod diagnostic;
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
    reason = "the parent command dispatcher is the only consumer"
)]
mod source_structure;
#[allow(
    clippy::redundant_pub_crate,
    reason = "the parent command dispatcher is the only consumer"
)]
mod task_error;

use std::env;
use std::ffi::OsString;
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
    if let Some(extra) = arguments.next() {
        let extra = extra
            .into_string()
            .map_err(|_| TaskError::InvalidExtraArgumentEncoding)?;
        return Err(TaskError::UnexpectedArgument(extra));
    }
    let repository_root = repository_root()?;
    match command.as_str() {
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
            source_structure::check(repository_root)?;
        }
        _ => return Err(TaskError::UnknownCommand(command)),
    }
    Ok(())
}

fn repository_root() -> Result<&'static Path, TaskError> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or(TaskError::RepositoryRoot)
}
