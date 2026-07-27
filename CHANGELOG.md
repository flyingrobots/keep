# Changelog

All notable changes to Keep will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project intends to follow [Semantic Versioning](https://semver.org/)
after its public API and format compatibility policies are established.

## [Unreleased]

### Changed

- Golden File Worldline verification now runs through a dependency-isolated
  Rust `xtask`, cross-checks every identity-bearing digest against external
  `b3sum`, and CI refuses Rust, Python, or shell source modules that exceed the
  documented 500-physical-line hard maximum, including test modules.
- Repository source verification now uses capability-relative, no-follow file
  opens and verifies repository-root identity after Git inventory, so a
  persistent root replacement or source path replaced with a symlink is
  refused before source bytes are read.
- The repository `cargo xtask` alias and Rust command contract are now
  explicitly silent on success and emit one typed `Error:` diagnostic with
  exit status 1 on refusal; untrusted control characters are escaped so the
  diagnostic remains one physical line.
- Golden protocol framing, field, hexadecimal, path, mutation-operation, and
  fixed-width value decoders now share a bounded fuzz surface backed by
  precise table-driven malformed-corpus refusals.
- Deterministic fuzz seed materialization now uses a capability-bound Rust
  `xtask`, syncs and atomically publishes derived seed files without mutating
  hard-link targets, recovers interrupted fixed-name staging files, cleans
  failed stages, and gives `golden_protocol` seeds that reach every table and
  semantic parser.
- Golden File Worldline source paths now have a named version-1 lexical
  profile with typed lexical refusal reasons and an explicit portability
  rationale, and both tables and named sources refuse final-component
  symlinks.
- Filesystem-backed `xtask` tests now use collision-resistant scoped
  directories with explicit cleanup instead of PID-only paths.
- Scheduled and manually dispatched fuzz campaigns now exercise every
  registered target under centralized bounded policy, retain failures, and
  preserve minimized evolving corpora as non-authoritative test state.
- CI now runs every registered fuzz target with pinned `cargo-fuzz` and
  nightly versions, deterministic deep-state seeds, bounded per-target
  resources, and retained failure artifacts.
- Moved the canonical text and binary `BlobId` codecs out of the `blob`
  identity module into a new `adapters` boundary layer, per
  [ADR-0004](docs/adr/0004-hexagonal-boundary-architecture.md). `blob` now
  owns only identity calculation; encoding and decoding live at the
  boundary. No public API or format change.
- `BlobId`'s `Debug` output changed from `BlobId(<canonical text>)` to
  `BlobId { logical_length: ..., digest: [..] }` so core's `Debug` impl no
  longer depends on the adapter-owned `Display` impl. `Debug` output carries
  no stability contract; this is not a format change.

### Added

- Validated flat-layout admission with explicit entry caps and an exact
  canonical version-1 encoder backed by every frozen record and `LayoutId`
  witness.
- Bounded flat-layout decoding through explicit parse, validate, and admit
  stages with deterministic first-failure errors for every frozen structural
  mutation and optional final expected-`LayoutId` verification.
- Canonical `StorageProfileId` text coordinates and explicit admission of the
  frozen `fastcdc-64k-v1` profile through `RegisteredStorageProfile`.
- Canonical binary and text `LayoutId` coordinates with typed plan-length and
  digest mismatch reporting backed by every coordinate refusal vector.
- The canonical `keep.flat-chunks/v1` durable layout specification, typed
  `LayoutId` grammar, checked flat-plan bounds, domain-separated checksum,
  exact golden records, field-complete `LayoutId` refusal tables and
  cardinality-before-aggregate first-failure plan mutation ledger, and
  verified storage-profile boundary replay law. Ingestion and verified
  reconstruction remain outside the current implementation.
- Canonical version-1 `ChunkId` calculation in a domain distinct from
  `BlobId`, with independent golden vectors.
- A constant-memory `FastCdc` detector for `fastcdc-64k-v1` that preserves
  boundaries and chunk identities across arbitrary feed partitioning, batches
  contiguous identity-hash updates, and enters an explicit failed state after
  a typed refusal.
- Typed `ChunkLength`, `ChunkOffset`, and `ChunkSpan` values, corpus-driven
  property and adversarial tests, measured allocation and throughput evidence,
  and a fail-closed streaming CDC fuzz target.
- Canonical version-1 `BlobId` calculation over exact logical bytes using a
  one-pass, length-committing BLAKE3-256 preimage.
- Strict, allocation-bounded text and fixed-width binary `BlobId` codecs with
  typed refusal for malformed and unsupported encodings.
- The implementation-independent Golden File Worldline v1 conformance corpus,
  independent vector checker, mutation cases, and bounded reference model.
- A versioned Gear64/FastCDC content-defined chunking profile, canonical
  `StorageProfileId`, and language-neutral golden boundary corpus.
- Initial repository foundation.
