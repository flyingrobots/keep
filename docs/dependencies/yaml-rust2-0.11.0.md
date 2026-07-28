# Dependency Admission: yaml-rust2 0.11.0

- Status: Accepted for repository-task workflow admission only
- Date: 2026-07-28
- Owner: Keep repository verification
- Upstream:
  [Ethiraric/yaml-rust2](https://github.com/Ethiraric/yaml-rust2)

## Admitted use

Keep admits exactly pinned `yaml-rust2` 0.11.0 only behind the `xtask` crate's
`repository-tasks` feature. The documentation-integrity task parses
`.github/workflows/ci.yml`, selects the `documentation` job's actual `run`
fields, and admits only the reviewed command set. Comments, display strings,
unrelated fields, and additional shell commands cannot satisfy that execution
contract.

The dependency is absent from Keep's published library graph, public API,
content identities, durable formats, and production behavior. Its typed parse
error remains inside the private repository-task adapter.

## Why a dependency is needed

YAML includes quoted and block scalars, comments, aliases, nested collections,
and duplicate mapping keys. A substring scan cannot distinguish executable
`run` fields from inert text. A maintained parser keeps the workflow boundary
structural and fail-closed without creating a partial YAML implementation
inside Keep.

The parsed values are never hashed, persisted, or admitted as Keep domain
types. The task reads the fixed workflow path through the bounded,
capability-relative, no-follow repository-file boundary before parsing it.

## Features and resolved graph

The direct dependency disables default features and is optional. It is
activated solely by `repository-tasks`; disabling defaults excludes the
optional non-UTF-8 input support.

The introduced normal dependency graph is:

- `arraydeque` 0.5.1;
- `foldhash` 0.2.0;
- `hashbrown` 0.16.1; and
- `hashlink` 0.11.1.

## Safety, licensing, and compatibility

`yaml-rust2` declares the MIT OR Apache-2.0 license expression and a minimum
supported Rust version of 1.65, below Keep's pinned toolchain. Its 0.11.0 Rust
source contains no `unsafe` block. Keep-owned code invokes only safe APIs.

`cargo deny check licenses bans sources` and `cargo audit` pass with the
resolved graph. These checks remain mandatory point-in-time evidence.

## Failure and recovery boundaries

Malformed YAML, duplicate mapping keys, an absent documentation job, an
unreviewed command, an oversized workflow, a non-UTF-8 workflow, or a replaced
repository root produces a typed refusal. The task never repairs, rewrites, or
substitutes workflow data. Parsing has no durability or recovery semantics.

Keep can remove this dependency without changing public or durable behavior by
replacing it with an equally bounded parser that preserves structural `run`
selection, duplicate-key refusal, and the reviewed-command laws.

Reopen this admission if the direct version, selected features, resolved graph,
license, MSRV, repository-task-only boundary, or admitted YAML use changes.
