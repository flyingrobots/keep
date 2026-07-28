# Changelog

All notable changes to Keep will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project intends to follow [Semantic Versioning](https://semver.org/)
after its public API and format compatibility policies are established.

## [Unreleased]

### Changed

- Reconstruction output-accounting errors now expose typed `BlobLength`
  coordinates consistently.
- Public streaming CAS behavior is now checked after every operation in all
  216 exhaustive three-step sequences over admission, reads, dropped staging,
  empty blobs, idempotence, and claimed-content mismatch.
- Flat-layout decoding now validates cardinality before entry materialization
  and admits decoded coordinates through the domain's ordered one-pass
  validator, so a later zero length cannot hide an earlier entry failure.
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

- Validated half-open `ByteRange` coordinates and allocation-free range
  planning, plus exact synchronous reference-store range reads that load only
  overlapping chunks, authenticate each selected complete chunk before
  slicing, reauthenticate before output, and return a receipt whose deliberately
  narrow verification scope excludes the complete blob, unrequested chunks,
  and storage-profile boundaries. Caller-supplied layouts and records must
  resolve to a committed target-layout binding before chunk lookup.
- Expected-`BlobId` staging with typed complete-stream mismatch refusal; the
  Golden File Worldline scenario and every claimed-content mutation now run
  through public stage, commit, and reconstruct APIs instead of only the
  private test model.
- Publication now calculates and validates the final materialized-byte count
  before changing visible reference-store state, so intervening capacity
  exhaustion cannot expose a partial commit.
- Publication refuses staged work whose destination lacks a required chunk,
  preventing cross-store commits from exposing incomplete layouts.
- Bounded canonical layout-record reconstruction with typed pre-output refusal
  for malformed records, zero-progress and over-reporting writers, output I/O
  failures, conflicting stored chunks, corrupted chunk content, and ordinary
  publication attempts that would silently repair missing committed chunks or
  missing, incomplete, or wrong-target committed layout indexes.
- Exact synchronous reference-store reconstruction that authenticates every
  chunk, the registered storage-profile boundaries, and the complete named
  blob before output, reverifies chunks during emission, completes short
  writes, retries interruptions, and reports missing or mismatched content
  with typed expected and observed identities.
- Capacity-bounded streaming ingestion into a non-durable in-memory reference
  adapter, with exact blob, chunk, and layout identity calculation, typed
  source and capacity refusals, streaming enforcement of the caller's layout
  entry cap, identity-based chunk deduplication, and an explicit
  staged-to-visible commit transition.
- An independent field-by-field flat-layout fixture oracle that verifies every
  fixed offset, checksum, and `LayoutId` before cross-checking the production
  encoder.
- Validated flat-layout admission with explicit entry caps and an exact
  canonical version-1 encoder backed by every frozen record and `LayoutId`
  witness.
- Bounded flat-layout decoding through explicit parse, validate, and admit
  stages with deterministic first-failure errors for every frozen structural
  mutation and optional final expected-`LayoutId` verification.
- Generated flat-layout canonicality properties and a continuous
  `layout_record` decoder fuzz target seeded through the Rust `xtask` with all
  four frozen binary records.
- Canonical `StorageProfileId` text coordinates and explicit admission of the
  frozen `fastcdc-64k-v1` profile through `RegisteredStorageProfile`.
- Canonical binary and text `LayoutId` coordinates with typed plan-length and
  digest mismatch reporting backed by every coordinate refusal vector.
- The canonical `keep.flat-chunks/v1` durable layout specification, typed
  `LayoutId` grammar, checked flat-plan bounds, domain-separated checksum,
  exact golden records, field-complete `LayoutId` refusal tables and
  cardinality-before-aggregate first-failure plan mutation ledger, and
  verified storage-profile boundary replay law.
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
