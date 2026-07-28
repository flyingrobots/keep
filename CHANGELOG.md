# Changelog

All notable changes to Keep will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project intends to follow [Semantic Versioning](https://semver.org/)
after its public API and format compatibility policies are established.

## [Unreleased]

### Changed

- Documentation corpus selection, pinned tool admission, Markdown and fragment
  checks, workflow linting, Dependabot coverage, and Node lock-graph policy now
  run through bounded Rust `xtask` code; CI and `cargo xtask verify` use that
  boundary, and the seven superseded Python checkers have been removed. The
  boundary rejects duplicate repository JSON fields and unlocked installer
  substitutions, admits only the exact reviewed Node lock artifact, retains
  simultaneous Markdown and link failures, parses documentation workflow
  commands as YAML, rejects guarded or non-string `run` values, preserves
  declarations after Dependabot directory lists, and applies one deadline
  across captured and inherited child execution and output collection.
  Git-backed process fixtures ignore system and global Git configuration and
  preserve non-UTF-8 template paths without lossy conversion. Documentation
  Git inventory and tools start from one retained repository directory handle,
  so transient replacement of the ambient repository path cannot redirect
  validation. Terminal signals now become typed refusals while an external
  repository task is active, so captured and inherited child groups are killed
  and reaped before `xtask` returns.
- Fuzz build and run plans now carry external process deadlines from the
  reviewed campaign policy. Run deadlines use checked addition of the
  exploration budget and process-grace interval before process-group execution.
- ChunkId v1 and CDC profile v1 conformance now run through one bounded Rust
  `cargo xtask conformance-check` command, including the external `b3sum`
  witness, reproducible Gear-table recipe, scalar and streaming FastCDC laws,
  source mutations, and exact boundary corpus; the three superseded Python
  programs have been removed.
- Fuzz policy admission, target reconciliation, bounded campaign execution,
  minimization deadlines, retained-corpus admission, and workflow contract
  tests now run through the repository's Rust `xtask`; the superseded Python
  fuzz scripts have been removed.
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
  opens and verifies repository-root identity after Git inventory and again
  after source scanning, so a persistent root replacement or source path
  replaced with a symlink is refused. The pure Rust boundary also refuses
  `.py`, `.pyw`, and Python shebangs in every executable source candidate,
  including attached `env -S` interpreter strings.
- Git path inventory failures now remain primary when child cleanup, waiting,
  or diagnostic collection also fails; the secondary failure remains typed and
  inspectable. Empty path records and unterminated path bytes produce distinct,
  accurate typed diagnostics.
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

- Public, allocation-free `SegmentHeader` admission and emission for the exact
  `keep.segment-store/v1` 64-byte header, with field-complete typed refusals
  and golden-corpus evidence.
- Public, allocation-free `SegmentRecordHeader` admission and emission for the
  exact 112-byte chunk and flat-layout record grammar, with typed logical
  identities, checked length derivation, and field-complete corruption laws.
- Borrowed `ChecksummedSegmentRecord` and `AdmittedSegmentRecord` states for
  bounded complete-record framing, checksum verification, logical
  content-identity admission, and allocation-free chunk preparation.
- Public, allocation-free `SegmentSeal` admission and emission for the exact
  128-byte immutable-segment terminator, with checked physical coordinates,
  domain-separated digest verification, and seal-checksum corruption laws.
- Borrowed `AdmittedSegment` reading with explicit record and layout resource
  limits, exact nested framing and identity admission, physical-order record
  iteration, trailing-byte refusal, and duplicate-identity index reservation
  bounded by both the configured count and physical record-header capacity.
- Consuming `StagedSegment` transitions and immutable `SealedSegment` receipts
  for exact append-only record writing, streaming seal construction, explicit
  prefix/sealed flush-and-sync order, phase-typed I/O refusals, and a fallibly
  reserved membership index for sublinear duplicate admission.
- Exclusive `FilesystemSegmentStage` creation for the fixed `current.seg`
  staging name, with atomic no-replacement admission, preserved existing
  evidence, zero-origin writing, and no implicit cleanup from `Drop`.
- Rust cargo-fuzz coverage for the public segment header, record header,
  complete record, seal, and complete-segment parser boundaries, seeded from
  the canonical version-1 segment fixtures through `cargo xtask`.
- ADR-0005 and the implementation-independent `keep.segment-store/v1`
  protocol: exact immutable segment, catalog-generation, and publication-head
  grammars; canonical ordering, bounds, and domain-separated checksums;
  one-writer/many-reader publication with explicit flush, synchronization,
  atomic replacement, and directory-synchronization order; stable
  `KEEP-CRASH-001`–`035` transitions; typed recovery classifications; and
  golden physical artifacts. Directory-synchronization crash classes admit
  both the lawful pre-sync and durable namespace states, and recovery admits
  only the exact verified stage/pool digest duplicate created by interrupted
  hard-link publication. Fresh-store initialization is writer-locked,
  idempotent across every partial canonical namespace set, and admitted only
  after root synchronization. Explicit recovery can complete a durable
  fixed-name stage into its immutable pool and durably clear the stage without
  promoting a publication head. Explicit discard receipts now follow
  synchronization of the stage's actual parent: `staging` for segment and
  catalog stages, or the store root for `head.next`. Production storage
  remains assigned to issues #15–#17. The golden corpus now includes a
  generation-2 catalog/head pair whose predecessor field is the exact
  generation-1 catalog digest.
- A deterministic, bounded, license-safe streaming CAS benchmark corpus and
  release-only `cargo xtask benchmark-baseline` workflow covering all required
  ingestion, edit, deduplication, range-read, verification, and input
  partitioning scenarios. The versioned TSV report records exact semantic I/O,
  amplification and reuse ratios, p50/p95/p99 wall latency, process CPU time,
  throughput, allocations, incremental peak live heap, five chunking-profile
  comparisons, compiler/target/Git/host identity bound across execution,
  refusal of ambient code-generation settings and external Cargo
  configuration, single-writer recoverable artifact publication, and an
  explicit refusal to invent regression thresholds before controlled baseline
  history exists.
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
