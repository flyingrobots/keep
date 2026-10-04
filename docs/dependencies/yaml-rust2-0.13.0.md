# Dependency Admission: yaml-rust2 0.13.0

- Status: Accepted for repository-task YAML admission only
- Updated: 2026-10-03
- Owner: Keep repository verification
- Upstream: [Ethiraric/yaml-rust2](https://github.com/Ethiraric/yaml-rust2)

## Admitted use

Keep admits exactly pinned `yaml-rust2` 0.13.0 only behind the `xtask` crate's `repository-tasks` feature. The documentation-integrity task parses the CI workflow and Dependabot configuration through `YamlLoader::load_from_str`; fuzz-campaign workflow tests also consume that parser.

Workflow admission selects the documentation job's actual executable fields and admits only the reviewed steps, permissions, triggers and tool setup. Dependabot admission checks update scopes against the repository's tracked manifests. Comments, display strings and unrelated fields cannot substitute for those structural contracts.

The dependency is absent from Keep's published library graph, public API, content identities, durable formats and production storage behavior. Its typed parse error remains inside the private repository-task adapter.

## Why a dependency is needed

YAML includes quoted and block scalars, comments, aliases, nested collections and duplicate mapping keys. A maintained parser keeps admission structural without introducing a partial YAML implementation inside Keep.

Parsed values are never hashed, persisted or admitted as Keep domain types. The task reads fixed policy paths through the bounded, capability-relative, no-follow repository-file boundary before parsing.

## Features and resolved graph

The direct dependency disables default features and is optional. Only `repository-tasks` activates it; non-UTF-8 decoding through the upstream `encoding` feature is excluded.

The resolved normal dependency graph includes:

- `arraydeque` 0.5.1;
- `foldhash` 0.2.0;
- `hashbrown` 0.17.1; and
- `hashlink` 0.12.2.

## Upgrade and compatibility

PR #103 updates the previous 0.11.0 admission. Upstream raises its minimum supported Rust version to 1.85.0, below Keep's pinned 1.96.0, and updates hashlink and hashbrown. The MIT OR Apache-2.0 license expression is unchanged.

The published 0.13.0 source changes its active scanner's `map_or(false, ...)` expression to `is_some_and(...)` and corrects the `Marker::index` documentation to bytes. Its short-input decoder progress fix is inside the disabled `encoding` module; Keep does not claim that fix as an exercised runtime improvement.

The 0.13.0 Rust source contains no `unsafe` block. Keep-owned code continues to invoke safe APIs, and dependency-owned YAML types remain private to tooling.

Existing workflow, duplicate-key, Dependabot and CLI admission laws are exercised in debug and release, without editing their expectations. These are bounded tool-contract checks; they do not establish equivalence over every possible YAML input or strengthen Keep's storage claims.

The reviewed lockfile, license/source policy checks and security advisory checks remain required point-in-time admission evidence. Earlier green checks on 0.11.0 do not certify this graph.

## Failure and recovery boundaries

Malformed YAML, duplicate mapping keys, missing policy fields, unreviewed commands, oversized input, non-UTF-8 input and replaced repository roots remain typed refusal cases covered by the existing admission contracts. The task does not repair or rewrite workflow data. Parsing has no storage durability or recovery semantics.

Keep can remove this dependency without changing public or durable behavior by replacing it with a parser that preserves structural selection, duplicate-key refusal, the reviewed-command laws and the manifest-coverage contract.

Reopen this admission if the direct version, selected features, resolved graph, license, MSRV, repository-task-only boundary or admitted YAML use changes.
