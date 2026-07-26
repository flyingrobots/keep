# Dependency Admission: Divan 0.1.21

- Status: Accepted for benchmark targets only
- Date: 2026-07-25
- Owner: Keep performance evidence
- Upstream: [nvzqz/divan](https://github.com/nvzqz/divan)

## Admitted use

Keep admits the exactly pinned `divan` 0.1.21 package only as a development
dependency. The `streaming_cdc` benchmark measures whole-slice and one-byte-feed
throughput for bounded deterministic inputs.

Divan is absent from Keep's published library graph, public API, identity
preimages, durable formats, and production behavior. Default features are
disabled because Keep does not need wrapped command-line help.

## Why it is required

Streaming CDC is performance-sensitive, and ADR-0003 requires throughput,
allocation, and peak-memory evidence before an optimization claim. Stable Rust
does not provide the built-in benchmark harness available behind nightly
features. A local timing harness would duplicate warmup, sampling, statistics,
and optimizer barriers while producing weaker regression evidence.

Divan provides those benchmark mechanics while benchmark code continues to
exercise Keep's safe public `FastCdc` API.

## Dependency and safety boundary

The exact package supports Rust 1.80 and declares `MIT OR Apache-2.0`; Keep
selects Apache-2.0. Its locked graph is development-only and remains subject to
the repository's dependency-policy and advisory gates.

Divan and its graph may use platform timing, process, or unsafe internals. Keep
passes caller-owned byte slices and receives benchmark statistics; no
dependency-owned type crosses a production boundary.

## Maintenance and exit

The benchmark is evidence, not part of content identity or compatibility.
Keep may replace Divan with another stable benchmark harness without moving an
API or format, provided whole-feed and one-byte-feed scenarios remain directly
comparable.

Reopen this admission if the version, selected features, resolved graph,
license, supported targets, or benchmark scenarios change.
