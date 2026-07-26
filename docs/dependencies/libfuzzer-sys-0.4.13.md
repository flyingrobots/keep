# Dependency Admission: libfuzzer-sys 0.4.13

- Status: Accepted for non-published fuzz tooling
- Date: 2026-07-19
- Owner: Keep conformance testing
- Upstream: [rust-fuzz/libfuzzer](https://github.com/rust-fuzz/libfuzzer)

## Admitted use

Keep admits the exactly pinned `libfuzzer-sys` 0.4.13 package only in the
separate, non-published `keep-fuzz` workspace. It supplies LLVM libFuzzer's
runtime and the `fuzz_target!` integration used to continuously exercise the
text parser, binary decoder, and streaming identity state machine.

It is not linked into the `keep` library, appears in no Keep public API or
durable format, and cannot participate in content identity.

## Why a dependency is required

The standard library has no coverage-guided fuzzing engine. A local wrapper
would still need an external mutation engine, sanitizer integration, process
protocol, corpus management, and unsafe FFI. Reimplementing those facilities
would exceed 50 lines and create a less-reviewed safety boundary.

## Safety boundary

`libfuzzer-sys` wraps LLVM's C++ libFuzzer runtime. Its crate contains unsafe
FFI declarations, pointer conversion, and callback glue, and its build script
compiles native runtime code through `cc`. That unsafe code is admitted only in
the fuzz executable workspace. Keep-owned production crates remain
`unsafe_code = "forbid"`, and fuzz inputs cross into Keep exclusively through
safe public byte-slice and string APIs.

The package is licensed as `(MIT OR Apache-2.0) AND NCSA`; the fuzz-specific
dependency policy contains an exact package/version exception. No global NCSA
or MIT license allowance follows from this admission.

## Version, features, and transitive graph

The manifest pins `libfuzzer-sys` exactly at 0.4.13, disables default features,
and explicitly enables only `link_libfuzzer`. The package does not declare a
Cargo `rust-version`. Keep compiles and lints the fuzz workspace with the
repository's pinned Rust 1.96.0 toolchain as executable compatibility
evidence.

Its direct graph adds:

- `arbitrary` 1.4.2;
- build dependency `cc` 1.3.0 and that package's locked build graph.

The dedicated fuzz lockfile, policy check, advisory scan, formatting check,
Clippy gate, compile gate, and runtime smoke gate run independently from the
production workspace. Any resolution change must repeat this review.

## Runtime execution

The runtime gates read the exact `cargo-fuzz` version, dated nightly, and
resource bounds from `fuzz/campaign.env`. They install the runner from
crates.io with its published lockfile. The nightly remains isolated from
Keep's stable Rust 1.96.0 production and MSRV gates. The runner is CI tooling;
it is not part of either Keep lockfile, public API, or durable format.

`cargo-fuzz` is licensed as `MIT OR Apache-2.0`. Its installer graph is fixed
by its published lockfile rather than either Keep lockfile, so Keep's
`cargo deny` and `cargo audit` jobs do not inspect that tool graph. The exact
version, locked installation, ephemeral runner, and read-only repository
permission bound that separate supply-chain surface; an upgrade requires
renewed review.

Every target declared by `cargo fuzz list` must exactly match a checked-in
harness. The pull-request smoke profile uses a short startup budget; the
scheduled profile uses a larger bounded exploration budget. Both share exact
per-input timeout, input-length, and RSS limits. Before mutation starts, a
bounded preparation script materializes canonical parser encodings plus
minimum, natural-boundary, probe-carry, hard-maximum, and multi-chunk CDC
witnesses. Neither profile claims exhaustive state-space coverage.

GitHub-hosted runners execute the campaigns with only read-only GitHub
contents permission. `fuzz/campaign.env` defines finite retention periods for
pull-request failures, scheduled failures, and minimized corpus evidence. The
evolving cache is disposable, non-authoritative acceleration and may be absent
or evicted without changing the campaign's validity. Restored and retained
corpora must remain regular-file trees beneath recognized target directories
and within the file, byte, and input limits in the campaign policy. A
confirmed failure must become a committed deterministic regression test.

## Maintenance and exit strategy

The resolved package identifies the rust-fuzz project as its maintainer and
the official `rust-fuzz/libfuzzer` repository as its source. The locked crate
builds under Keep's current toolchain and target; this is point-in-time
evidence, not a maintenance guarantee.

Because no dependency type or format escapes the fuzz workspace, Keep can
replace this tool with another coverage-guided engine, move fuzz targets to an
external harness, or remove the integration without changing the library API
or any identity. A replacement must preserve the successful-parse canonical
round-trip and partition-invariance properties.

## Review triggers

Reopen this admission when any of these changes:

- package version, enabled features, or transitive graph;
- compiler, sanitizer, host, or target requirements;
- `cargo-fuzz` version, pinned nightly, or campaign resource bounds;
- unsafe or native-code boundary;
- license or advisory posture;
- a dependency-owned type crosses into production code;
- fuzz execution moves into a privileged or production environment.
