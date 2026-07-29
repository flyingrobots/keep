//! Repository verification and pure protocol-admission surfaces.

#![deny(warnings)]
#![forbid(unsafe_code)]

#[cfg(feature = "golden-protocol-fuzz")]
extern crate self as xtask;

#[cfg(feature = "golden-protocol-fuzz")]
mod diagnostic;

#[cfg(feature = "golden-protocol-fuzz")]
#[allow(
    clippy::redundant_pub_crate,
    reason = "the library facade deliberately hides the parser implementation"
)]
mod golden_protocol_fuzz;

#[cfg(feature = "repository-json-fuzz")]
#[allow(
    clippy::redundant_pub_crate,
    reason = "the library facade deliberately hides the repository parser implementation"
)]
#[path = "documentation_integrity/node_toolchain/unique_json.rs"]
mod repository_json;

pub mod protocol_admission;

#[cfg(test)]
#[allow(
    clippy::redundant_pub_crate,
    reason = "scoped test directories are shared by parser test modules"
)]
mod test_directory;

/// Whether one bounded Golden File Worldline production parser admitted input.
#[cfg(feature = "golden-protocol-fuzz")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoldenProtocolAdmission {
    /// The selected production parser admitted the input.
    Admitted,
    /// The selected production parser refused the input.
    Refused,
}

/// Exercise one bounded production parser from the Golden File Worldline corpus.
///
/// Every selector value maps deterministically to a table, scalar,
/// invalid-identity, or mutation parser. This entry point performs no I/O and
/// exists so fuzzing exercises the exact parser implementations used by the
/// repository checker.
#[cfg(feature = "golden-protocol-fuzz")]
#[must_use]
pub fn admit_golden_protocol(selector: u8, input: &[u8]) -> GoldenProtocolAdmission {
    if golden_protocol_fuzz::admit(selector, input).is_ok() {
        GoldenProtocolAdmission::Admitted
    } else {
        GoldenProtocolAdmission::Refused
    }
}

/// Whether bounded duplicate-refusing repository JSON admission accepted input.
#[cfg(feature = "repository-json-fuzz")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepositoryJsonAdmission {
    /// UTF-8 JSON within the byte and nesting limits was admitted.
    Admitted,
    /// Encoding, syntax, size, nesting, or duplicate members were refused.
    Refused,
}

/// Exercises the exact duplicate-refusing repository JSON parser.
///
/// Input above one mebibyte is refused before UTF-8 decoding or recursive
/// parsing. No serializer-owned value escapes this fuzz-only boundary.
#[cfg(feature = "repository-json-fuzz")]
#[must_use]
pub fn admit_repository_json(input: &[u8]) -> RepositoryJsonAdmission {
    const MAXIMUM_BYTES: usize = 1_048_576;
    if input.len() > MAXIMUM_BYTES {
        return RepositoryJsonAdmission::Refused;
    }
    let Ok(raw) = std::str::from_utf8(input) else {
        return RepositoryJsonAdmission::Refused;
    };
    if repository_json::parse(raw).is_ok() {
        RepositoryJsonAdmission::Admitted
    } else {
        RepositoryJsonAdmission::Refused
    }
}
