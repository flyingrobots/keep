# Dependency Admission: serde 1.0.229 and serde_json 1.0.151

- Status: Accepted for repository-task JSON admission only
- Date: 2026-07-28
- Owner: Keep repository verification
- Upstream:
  [serde-rs/json](https://github.com/serde-rs/json)

## Admitted use

Keep admits the exactly pinned `serde_json` 1.0.151 and `serde` 1.0.229
packages only behind the `xtask` crate's `repository-tasks` feature. They parse
the committed Node documentation-tool manifest and lockfile so the Rust
documentation integrity task can validate their structure and reviewed
dependency versions. A Keep-owned recursive visitor rejects duplicate object
members at every depth before returning a JSON value.

The dependencies are absent from Keep's published library graph, public API,
content identities, durable formats, and production behavior. No
dependency-owned type crosses out of the private repository-task adapter.

## Why a dependency is needed

JSON syntax includes escapes, Unicode, numbers, nested collections, and
duplicate representation details that do not belong in a local partial parser.
The documentation gate needs structural lookup, not canonical identity bytes.
Using a maintained parser keeps malformed input fail-closed without creating a
second JSON implementation inside Keep.

The parsed values are never hashed, persisted, or admitted as Keep domain
types. The repository task reads fixed paths through the bounded,
capability-relative, no-follow file boundary before parsing them.

## Features and resolved graph

Both direct dependencies disable default features and enable only `std`. They
are optional and activated solely by `repository-tasks`.

The active normal dependency graph introduced for this boundary consists of:

- `itoa` 1.0.18;
- `memchr` 2.8.3;
- `serde` and `serde_core` 1.0.229; and
- `zmij` 1.0.23.

Cargo's all-target resolution also retains `serde_derive` 1.0.229 and `syn`
3.0.3. Their procedural-macro dependencies were already present in the
workspace lockfile.

## Safety, licensing, and compatibility

`serde_json` and `serde` declare the MIT OR Apache-2.0 license expression.
Their manifests declare minimum supported Rust versions below Keep's pinned
toolchain.

Keep-owned code invokes only safe APIs. The parser and its transitive
dependencies may contain implementation details outside Keep's `unsafe_code`
lint boundary, so `cargo deny` and RustSec checks remain mandatory
point-in-time evidence.

## Failure and recovery boundaries

Malformed JSON, an oversized file, a non-UTF-8 file, a non-regular file, or a
repository-root replacement produces a typed refusal. The task never repairs,
rewrites, or substitutes repository data. Parsing has no durability or
recovery semantics.

Keep can remove this dependency without changing public or durable behavior by
replacing it with an equally bounded parser that preserves the exact rejection
and structural-validation laws.

Reopen this admission if the direct version, selected feature, resolved graph,
license, MSRV, repository-task-only boundary, or admitted JSON use changes.
