# Keep Roadmap

This page has one job: to list every feature Keep has, is building, or
intends, and to break the unfinished ones into tasks precise enough to
start. It is a plan. It is not evidence. The authoritative status of every
requirement, with the test that proves it, remains the requirement ledgers:
[`segment-store-v1/requirements.md`](docs/formats/segment-store-v1/requirements.md),
[`segment-store-v2/requirements.md`](docs/formats/segment-store-v2/requirements.md),
and
[`authenticated-reconstruction/requirements.md`](docs/invariants/authenticated-reconstruction/requirements.md).
Where this page and a ledger disagree, the ledger wins.

Snapshot: `main` at `f49cff7`, 2026-09-30. Twenty issues open, thirty-two
closed, one non-dependency pull request open (#99).

## How to read this page

Every feature has a status word:

| Status | Meaning |
| --- | --- |
| Done | Shipped on `main` with executable evidence named in a ledger, the changelog, or a test file |
| Partial | Some tasks shipped; the remaining tasks are listed with an owner |
| In review | An open pull request delivers it; not on `main` yet |
| Planned | Owned by a GitHub issue; no executable evidence |
| Proposed | Suggested here; no issue, no decision record, no owner |
| Out of scope | Refused or deferred by a decision record; listed so nobody re-proposes it by accident |

A feature is a capability a user can name. A task is one unit of work that
delivers part of a feature. Each unfinished task carries the full field set:
requirements, acceptance criteria, scope, user stories, interface, contract
schema, test plan, definition of done, complexity, documentation, and
dependencies. Completed tasks carry a checkbox and a pointer to their
evidence instead, because the ledgers already own that detail.

User stories take four perspectives:

- **Human**: an operator or maintainer running Keep by hand.
- **API user**: a Rust program linking the `keep` crate.
- **MCP user**: a person driving Keep through a Model Context Protocol server
  that wraps the crate. No such server exists today; the stories say what one
  would need. See F-42.
- **Agent**: an autonomous coding or operations agent acting through the API
  or an MCP server without a human in the loop.

Keep's core exposes no command-line interface by design
(`docs/Rust Standards.md` §5.2). Where a task lists an interface, it names
the Rust API and the operation an out-of-core CLI or MCP adapter would
expose over it.

Complexity uses four sizes: **S** (one pull request under the 400-line
target), **M** (two to four pull requests), **L** (a milestone slice of
several weeks with its own crash or corruption evidence), **XL** (spans
milestones or needs a new decision record first).

Task identifiers are `T-<feature>.<n>`. They exist only on this page.
Requirement identifiers (`KEEP-RETENTION-007`), crash points
(`KEEP-CRASH-036`), issues (`#21`), and ADRs (`ADR-0009`) are the durable
names; use those in code, tests, and commits.

## Checklist

### Foundations (M1 and M2)

- [x] [F-01 Core law and fail-closed contract](#f-01-core-law-and-fail-closed-contract) — Done
- [x] [F-02 BlobId exact logical identity](#f-02-blobid-exact-logical-identity) — Done
- [x] [F-03 Identity layers and RepresentationId](#f-03-identity-layers-and-representationid) — Done as a model; representation codec reserved
- [x] [F-04 Deterministic chunking and ChunkId](#f-04-deterministic-chunking-and-chunkid) — Done
- [x] [F-05 Flat chunk layout v1 and LayoutId](#f-05-flat-chunk-layout-v1-and-layoutid) — Done
- [x] [F-06 Reference store](#f-06-reference-store) — Done; two open defects
- [x] [F-07 Authenticated reconstruction and exact range reads](#f-07-authenticated-reconstruction-and-exact-range-reads) — Done for the reference store
- [x] [F-08 Conformance corpora and the Golden File Worldline](#f-08-conformance-corpora-and-the-golden-file-worldline) — Done
- [x] [F-09 Streaming CAS benchmark baseline](#f-09-streaming-cas-benchmark-baseline) — Done; thresholds unconfigured
- [x] [F-10 Hexagonal boundary architecture](#f-10-hexagonal-boundary-architecture) — Done

### Durable segment store version 1 (M3)

- [x] [F-11 Immutable segment format and verified I/O](#f-11-immutable-segment-format-and-verified-io) — Done
- [x] [F-12 Catalog generations and writer-locked publication](#f-12-catalog-generations-and-writer-locked-publication) — Done
- [x] [F-13 Store initialization, recovery, and the crash matrix](#f-13-store-initialization-recovery-and-the-crash-matrix) — Done
- [ ] [F-14 Segment store v1 living documentation refresh](#f-14-segment-store-v1-living-documentation-refresh) — Planned (#69)

### Retention and version 2 (M4)

- [x] [F-15 Retention roots, release, and GC liveness model](#f-15-retention-roots-release-and-gc-liveness-model) — Done (ADR-0009)
- [x] [F-16 Version-2 format records and codecs](#f-16-version-2-format-records-and-codecs) — Done
- [ ] [F-17 One-way migration from version 1 to version 2](#f-17-one-way-migration-from-version-1-to-version-2) — Partial (#97 and residual #19)
- [ ] [F-18 Retention publication](#f-18-retention-publication) — Partial; recovery in review (PR #99)
- [ ] [F-19 Reader fence and immutable version-2 snapshots](#f-19-reader-fence-and-immutable-version-2-snapshots) — In review (PR #99)
- [ ] [F-20 Model-based retention transition evidence](#f-20-model-based-retention-transition-evidence) — In review (PR #99)
- [ ] [F-21 Precise verification reports and corruption refusal](#f-21-precise-verification-reports-and-corruption-refusal) — Planned (#20)
- [ ] [F-22 Garbage collection, compaction, and recovery dispositions](#f-22-garbage-collection-compaction-and-recovery-dispositions) — Planned (#21)
- [ ] [F-23 Durable authenticated reads and refusal receipts](#f-23-durable-authenticated-reads-and-refusal-receipts) — Planned; no open Keep issue
- [ ] [F-24 Bounded production ingestion through the durable store](#f-24-bounded-production-ingestion-through-the-durable-store) — Planned (#82, #74, #72)

### Integration (M5)

- [x] [F-25 Echo adapter and transaction boundary](#f-25-echo-adapter-and-transaction-boundary) — Done in the Echo repository
- [ ] [F-26 Graft Golden File Worldline end to end](#f-26-graft-golden-file-worldline-end-to-end) — Planned (#24)
- [ ] [F-27 git-cas import posture](#f-27-git-cas-import-posture) — Planned (#25)

### Encrypted representations and lifecycle surfaces (M6)

- [ ] [F-28 Authenticated encrypted representations](#f-28-authenticated-encrypted-representations) — Planned (#86, #83)
- [ ] [F-29 Retention-derived lifecycle surfaces](#f-29-retention-derived-lifecycle-surfaces) — Planned (#85)
- [ ] [F-30 Opaque asset, page, and bundle handles](#f-30-opaque-asset-page-and-bundle-handles) — Planned (#89)
- [ ] [F-31 Private named vault and mutable root sets](#f-31-private-named-vault-and-mutable-root-sets) — Planned (#92)
- [ ] [F-32 Managed cache sets](#f-32-managed-cache-sets) — Planned (#90)
- [ ] [F-33 Expiry-safe replay sets](#f-33-expiry-safe-replay-sets) — Planned (#87)
- [ ] [F-34 Portable bindings](#f-34-portable-bindings) — Planned (#91)

### Exploration

- [ ] [F-35 Read-only FUSE projection](#f-35-read-only-fuse-projection) — Planned (#66)
- [ ] [F-36 Transactional write-enabled projection](#f-36-transactional-write-enabled-projection) — Planned (#73)

### Repository, process, and documentation

- [x] [F-37 Repository verification tooling and CI gates](#f-37-repository-verification-tooling-and-ci-gates) — Done
- [ ] [F-38 Documentation status drift](#f-38-documentation-status-drift) — Proposed; small

### Proposed, without an owner

- [ ] [F-39 Compression representation codec](#f-39-compression-representation-codec) — Proposed
- [ ] [F-40 Additional chunking profiles and hierarchical layouts](#f-40-additional-chunking-profiles-and-hierarchical-layouts) — Proposed
- [ ] [F-41 Platform adapters beyond Linux ext4](#f-41-platform-adapters-beyond-linux-ext4) — Proposed
- [ ] [F-42 Operator surfaces](#f-42-operator-surfaces) — Proposed (CLI and MCP adapters)
- [ ] [F-43 Public release and API stability](#f-43-public-release-and-api-stability) — Proposed
- [ ] [F-44 Multi-writer, replication, and remote tiers](#f-44-multi-writer-replication-and-remote-tiers) — Out of scope

### Assurance beyond empirical testing

- [ ] [F-45 Formal verification of the durable protocols](#f-45-formal-verification-of-the-durable-protocols) — Proposed
- [ ] [F-46 Condition coverage and mutation analysis](#f-46-condition-coverage-and-mutation-analysis) — Proposed

## Dependency map

Edges are "needs", read left to right. Only edges between unfinished
features are listed; finished prerequisites are implied.

- F-17 partial-prefix migration recovery needs F-17 T-17.1 (#97).
- F-18 recovery and F-19 and F-20 ship together in PR #99; F-18 orphan
  disposition then needs F-22.
- F-21 (#20) is the most-cited blocker: F-22, F-23, F-24, F-27, F-28, F-29,
  F-30, F-31, F-33, F-34 all name it.
- F-22 (#21) needs F-19, F-21, and the remainder of F-18; it is needed by
  F-26, F-29, F-31, F-32, F-33.
- F-23 needs F-19 and F-22 (a pinned view must survive collection).
- F-24 (#82) needs F-18, F-21, and the F-06 defects #74 and #72.
- F-26 (#24) needs F-22 and F-25.
- F-28 implementation (#83) needs the F-28 ADR (#86), F-21, #74, #72.
- F-29 (#85) needs F-18, F-21, F-22; it is needed by F-30, F-31, F-32, F-33.
- F-30 (#89) needs F-24 and F-29; F-31 (#92) needs F-30; F-32 (#90) needs
  F-31.
- F-34 (#91) needs F-21, F-26, and stable handles from F-30.
- F-35 (#66) needs F-19 and F-22 before any mount is exposed; F-36 needs
  F-35.
- F-42 needs F-23 and F-24 before an adapter has a durable path to wrap.
- F-43 needs every Planned feature in M4, plus F-24 and F-28, before a
  format-compatibility policy can be promised.

## Features

### F-01 Core law and fail-closed contract

**Status:** Done. Governs every other feature.

For a given content identity, Keep must return exactly the bytes named by
that identity, or refuse. Keep refuses, before mutating anything, a disk
that returns a corrupted block, a process killed mid-update that leaves
finished-looking state, and a byte-identical file substituted at the same
path. The core holds no clock, no caller identity, no paths, and no
application policy.

- [x] T-01.1 State the law and its limits — `README.md`,
  `docs/adr/0001-exact-logical-byte-identity.md`,
  `docs/invariants/authenticated-reconstruction/README.md`.
- [x] T-01.2 Make every refusal a typed value, never a string — every
  boundary error enum carries `expected` and `observed` fields and a
  preserved `source`; `unwrap_used`, `expect_used`, and `panic` are denied
  workspace-wide.
- [x] T-01.3 Keep application semantics out of the core — `KEEP-STORE-016`;
  Echo, Git, Graft, WARP, and CLI types never enter `src/`.

### F-02 BlobId exact logical identity

**Status:** Done (ADR-0001, issues #2 and #6, M1).

`BlobId` is BLAKE3-256 over a typed, domain-separated preimage
(`KEEP:BLOB:DATA`, version, algorithm, the bytes, and a trailing `u64`
length) with a strict 59-byte binary form and a strict text form
`keep:blob:v1:blake3-256:<length>:<digest>`. It never moves under
rechunking, repacking, compression, encryption, key rotation, migration,
compaction, or catalog rebuild.

- [x] T-02.1 Decide the identity contract — ADR-0001.
- [x] T-02.2 Implement `BlobId`, `BlobHasher`, `BlobLength`, and both codecs
  with typed parse failures — `src/blob/`, `src/adapters/`; corpus
  `conformance/golden-file-worldline/v1/`.
- [x] T-02.3 One-pass unknown-length streaming identity — `BlobHasher`
  keeps constant state; the length suffix makes a single pass sufficient.

Deferred by the ADR without an owner: tree-hash parallelism "when a future
measured path warrants it". Any faster path must reproduce every
independent vector first.

### F-03 Identity layers and RepresentationId

**Status:** Done as a model (ADR-0002, issue #3). The `RepresentationId`
codec is reserved and unassigned.

Five concepts stay distinct: `BlobId` (logical bytes), `LayoutId`
(reconstruction plan), `RepresentationId` (one stored encoding, including
compression and encryption), physical location (mutable catalog evidence),
and retention reference (liveness evidence). The transition-law table says
which identifiers may change under rechunk, repack, re-encrypt, tier copy,
compaction, and catalog rebuild.

- [x] T-03.1 Decide the layers and their transition laws — ADR-0002.
- [x] T-03.2 Assign layout codec 1 — `keep.flat-chunks/v1`, see F-05.
- [ ] T-03.3 Assign a representation codec — reserved envelope
  `KEEP:REPR:ID`; first assignment belongs to F-28 (encryption) or F-39
  (compression), whichever lands first. The catalog, retention, and GC
  formats must already carry a representation coordinate before either
  ships; see T-28.2.

### F-04 Deterministic chunking and ChunkId

**Status:** Done (ADR-0003, issues #7 and #8, M2).

Boundary algorithm `keep.fastcdc-gear64/v1`; the only registered profile is
`fastcdc-64k-v1` (minimum 16 KiB, target 64 KiB, maximum 256 KiB, NC2
normalization, seed 0). Boundaries are source-partition invariant; the
detector retains at most 4 KiB of state and allocates nothing on the heap.
`ChunkId` names one exact nonempty chunk under its own domain
`KEEP:CHUNK:DATA`.

- [x] T-04.1 Decide the algorithm and profile record — ADR-0003; 96-byte
  profile record; `StorageProfileId`.
- [x] T-04.2 Implement `FastCdc` and `ChunkId` — `src/chunk/`;
  `tests/streaming_cdc.rs`, `tests/streaming_cdc_memory.rs`,
  `fuzz/fuzz_targets/fast_cdc.rs`, `benches/streaming_cdc.rs`.
- [x] T-04.3 Language-neutral corpora — `conformance/cdc-profile/v1/`,
  `conformance/chunk-id/v1/`; `cargo xtask conformance-check`.

Open design note without an owner: the `FastCdc::feed` callback has no
fallible storage sink; F-24 supplies that boundary.

### F-05 Flat chunk layout v1 and LayoutId

**Status:** Done (issues #9, #10, #11, #13, M2). Two ledger rows are
specified by design only.

`keep.flat-chunks/v1` maps one `BlobId` to an ordered, bounded, contiguous
sequence of `ChunkId` entries under one registered `StorageProfileId`.
Decoded, validated, admitted, and verified-reconstruction states are
distinct types. Depth 1, at most 1,048,576 entries, at most 256 GiB per
plan.

- [x] T-05.1 Specify the format — `docs/formats/flat-chunk-layout-v1/`;
  `KEEP-LAYOUT-001` to `-012`.
- [x] T-05.2 Implement the codec, admission, and fuzz target —
  `AdmittedLayout`, `CanonicalLayoutRecord`, `LayoutDecodePolicy`;
  `fuzz/fuzz_targets/layout_record.rs`; corpus `conformance/layout/v1/`.
- [x] T-05.3 Verified reconstruction replays the profile — `KEEP-LAYOUT-016`.
- [x] T-05.4 Exact range planning — `AdmittedLayout::plan_range`,
  `KEEP-LAYOUT-017`.
- [ ] T-05.5 Name executable evidence for `KEEP-LAYOUT-013` and
  `KEEP-LAYOUT-015`. Both rows read "Specified" with no test named.
  Requirement: a test asserts that no physical coordinate participates in
  `LayoutId` (change a catalog location, identity unchanged) and that a
  codec-1 record with any hierarchical marker refuses with a typed error.
  Acceptance: ledger rows change to Implemented with file names. Scope in:
  two tests and a ledger edit. Scope out: any format change. Interface:
  none. Test plan: golden (existing corpus), edges (reserved bytes,
  unknown codec token), no fuzz needed. Definition of done: ledger and
  tests merged. Complexity: S. Documentation: ledger rows only.
  Dependencies: none.

### F-06 Reference store

**Status:** Done (issue #13). Two open defects, #71 and #74.

`ReferenceStore` is the capacity-bounded, in-memory, non-durable adapter
that proves the stage, commit, and reconstruct laws: `stage` chunks and
hashes without visibility, `StagedBlob::commit` is the explicit
transition, `reconstruct` and `read_range` verify before emitting. Process
death loses everything in it; no API makes a durability claim.

- [x] T-06.1 Bounded streaming ingestion and reconstruction —
  `tests/streaming_cas/`, 216 exhaustive three-step model sequences.
- [x] T-06.2 Chunk deduplication keyed by `ChunkId` as a storage fact, not
  retention — `docs/architecture/reference-store/README.md`.
- [ ] T-06.3 Single-pass authenticated emit (#71, P2).
  - **Requirements:** each selected chunk is hashed exactly once per
    `reconstruct` or `read_range` call; the complete `BlobId`, range
    accounting, and profile-boundary verification stay intact; the failure
    contract (untrusted prefix on failure) is unchanged.
  - **Acceptance criteria:** a counting adapter proves one
    `ChunkId::hash_bytes` call per selected chunk; all existing refusal laws
    pass; no new public type.
  - **Scope:** in — `src/reference/reconstruction.rs`,
    `src/reference/range_read_execution.rs`. Out — a strict two-phase mode;
    if a caller wants proof-before-emit, that is a new explicit API, not a
    default.
  - **User stories:** Human — a maintainer sees full-blob reads cost one
    hash per chunk in the benchmark, not two. API user — `reconstruct` on a
    large blob halves CPU with identical receipts. MCP user — a "read blob"
    tool returns faster with the same guarantee. Agent — an agent reading
    many blobs in a loop is not charged twice for verification.
  - **Interface:** unchanged `ReferenceStore::reconstruct*` and
    `read_*range`.
  - **Contract schema:** none.
  - **Test plan:** golden — existing `reconstruction_laws`; edges —
    single-chunk blob, two-chunk blob, range touching one boundary chunk;
    known failures — every `refusal_laws` case must still stop at the same
    chunk; fuzz — none; stress — the benchmark scenario `whole-blob read`
    must show read amplification at 1 instead of 2.
  - **Definition of done:** regression test merged, benchmark baseline
    regenerated, CHANGELOG entry.
  - **Complexity:** S.
  - **Documentation:** `docs/architecture/reference-store/rationale.md`
    paragraph on two verification passes rewritten.
  - **Dependencies:** none. Blocks nothing, but F-24 inherits the pattern.
- [ ] T-06.4 Bounded-memory staging (#74, P1).
  - **Requirements:** staging holds a bounded window of missing chunks, not
    every missing chunk for the whole blob; the bound is explicit, checked,
    and reported; `StagedBlob` semantics (invisible until commit) hold.
  - **Acceptance criteria:** an instrumented test proves peak staged bytes
    stay under a configured ceiling for a source larger than the ceiling; or
    a written rationale proves materialization is unavoidable for the
    in-memory adapter and the ceiling is enforced as a refusal instead.
  - **Scope:** in — `src/reference/chunk_staging.rs`,
    `src/reference/staged_blob.rs`, `IngestionAllocation`. Out — durable
    staging (F-24), asynchronous ingestion.
  - **User stories:** Human — staging a 4 GiB file does not need 4 GiB of
    RAM. API user — `stage` refuses with a typed capacity error before
    exhausting memory. MCP user — an "ingest file" tool cannot take the
    server down with one large input. Agent — an agent ingesting a corpus
    can predict memory from the documented bound.
  - **Interface:** `ReferenceStore::stage` gains no parameters; the bound
    comes from `ReferenceStoreCapacity` or a new `StagingWindow` value.
  - **Contract schema:** none.
  - **Test plan:** golden — existing ingestion laws; edges — source exactly
    at the window, one byte over, all chunks already present, none present;
    known failures — interrupted source mid-window discards only the
    window; fuzz — none; stress — `tests/streaming_cas_memory.rs` extended
    with a memory ceiling law.
  - **Definition of done:** memory law merged; README example unchanged.
  - **Complexity:** M.
  - **Documentation:** reference-store README "bounded memory" section.
  - **Dependencies:** blocks F-24 (#82) and F-28 (#83).

### F-07 Authenticated reconstruction and exact range reads

**Status:** Done for `ReferenceStore` (`KEEP-RECONSTRUCT-001` to `-008`).
The durable form is F-23.

Every read either establishes its proof scope, emits exactly the supported
bytes, and returns a receipt; or returns an evidenced refusal; or fails
operationally with no content claim. `ReconstructionReceipt` proves the
complete object. `RangeReadReceipt` proves only the requested bytes from
authenticated complete chunks and never satisfies an API that needs the
complete-object receipt.

- [x] T-07.1 State the contract —
  `docs/invariants/authenticated-reconstruction/`.
- [x] T-07.2 Complete-object and exact-layout reads —
  `tests/streaming_cas/reconstruction_laws.rs`.
- [x] T-07.3 Exact range reads load only overlapping chunks —
  `tests/range_read.rs`, `tests/range_read_properties.rs`,
  `tests/range_read_entrypoints.rs`.
- [x] T-07.4 Success, evidenced refusal, and operational failure are
  distinct — `tests/range_read_failures.rs`.

### F-08 Conformance corpora and the Golden File Worldline

**Status:** Done (issues #4, #5, #44, #57, #59).

Language-neutral, checked-in corpora with independent oracles: the Golden
File Worldline (eight semantic laws over states A and B), CDC profile v1,
ChunkId v1, layout v1, segment store v1 (with the `KEEP-CRASH-001` to
`-035` transition table), and segment store v2. Every checker is Rust,
lives in `xtask`, and imports no production code. The external `b3sum`
witness runs under a bounded, deadline-guarded process boundary.

- [x] T-08.1 Worldline scenario and reference model —
  `docs/conformance/golden-file-worldline.md`;
  `cargo xtask golden-file-worldline-check`.
- [x] T-08.2 CDC and ChunkId oracles in Rust — `cargo xtask conformance-check`.
- [x] T-08.3 Segment store v1 and v2 fixture oracles —
  `xtask/tests/segment_store_protocol_contract/`,
  `xtask/tests/retention_store_v2_format_oracle`.
- [x] T-08.4 Bounded external digest execution — ADR-0008.

Rows in `capabilities.tsv` marked `declared-future` (chunk reuse, exact
range I/O, durability, restart recovery, corruption refusal, retention,
compaction, encryption) become executable only when the owning feature
below lands; each such feature's definition of done includes flipping its
row.

### F-09 Streaming CAS benchmark baseline

**Status:** Done (issue #12). Regression thresholds are deliberately
unconfigured.

`cargo xtask benchmark-baseline` runs thirteen scenarios over a 16 MiB
generated corpus, compares the registered profile with benchmark-only
FastCDC sizes, fixed-size chunking, and git-cas Buzhash, and writes a TSV
with environment and commit identity. One baseline exists:
`benchmark/baselines/c529c07-aarch64-apple-darwin.tsv`.

- [x] T-09.1 Corpus, scenarios, metrics, and one baseline —
  `docs/benchmarks/streaming-cas-baseline-v1/README.md`.
- [ ] T-09.2 Regression thresholds from controlled history.
  - **Requirements:** at least five clean optimized baselines on one
    designated runner class before any tolerance is proposed; thresholds
    are per scenario and per metric; a threshold breach is advisory until
    the standard says otherwise.
  - **Acceptance criteria:** a documented runner class, five committed
    baselines, a proposal table with the tolerance and its derivation, and
    an xtask comparison command that reports breaches without failing CI.
  - **Scope:** in — baselines directory, comparison command, README table.
    Out — CI gating, optimization work, new scenarios (see T-09.3).
  - **User stories:** Human — a maintainer sees a p95 regression named in a
    PR check before merging. API user — none directly. MCP user — none.
    Agent — an agent proposing a chunker optimization can cite a threshold
    instead of a feeling.
  - **Interface:** `cargo xtask benchmark-compare <baseline> <candidate>`.
  - **Contract schema:** the existing TSV columns; a `tolerance` column in
    a new `thresholds.tsv`.
  - **Test plan:** golden — comparison over two identical files reports no
    breach; edges — missing scenario, missing column, different commit,
    different runner; fuzz — none; soak — five consecutive runs on the
    runner within the proposed tolerance.
  - **Definition of done:** proposal accepted in a rationale note; command
    merged.
  - **Complexity:** M (mostly waiting for history).
  - **Documentation:** benchmark README "Regression thresholds" section.
  - **Dependencies:** a Linux runner class (F-41 T-41.3) if durable
    scenarios are to count.
- [ ] T-09.3 Durable-store scenarios.
  - **Requirements:** scenarios the Rust standard §18 requires and the
    harness lacks: already-compressed data, recovery scan, root
    publication, GC planning, compaction, post-compaction reads; metrics
    the harness does not record: fsync count, store size on disk.
  - **Acceptance criteria:** each scenario reproducible from a generated
    corpus; results carry the same environment identity as the CAS
    baseline; verification cannot be silently disabled.
  - **Scope:** in — `benchmark/` scenarios and metrics. Out — thresholds.
  - **User stories:** Human — an operator sees how long recovery takes for
    a store of a given size. API user — none. MCP user — none. Agent — an
    agent planning a GC run can estimate its cost.
  - **Interface:** `cargo xtask benchmark-baseline --profile durable`.
  - **Contract schema:** TSV columns extended; documented in the README.
  - **Test plan:** golden — scenario list is pinned; edges — empty store,
    single-segment store; stress — a store at the catalog entry ceiling.
  - **Definition of done:** one committed durable baseline on Linux.
  - **Complexity:** M.
  - **Documentation:** benchmark README.
  - **Dependencies:** F-13 (recovery scan), F-18 (root publication), F-22
    (GC planning, compaction); each scenario lands with its feature.

### F-10 Hexagonal boundary architecture

**Status:** Done (ADR-0004).

The domain core owns laws, validated types, and policy-free orchestration.
Outbound ports name required capabilities (`RetentionPublicationStorage`
names seventeen). Adapters implement ports. Codecs live only at boundaries.
Core and ports import no adapter and no dependency wire type.

- [x] T-10.1 Decide the architecture — ADR-0004.
- [x] T-10.2 Enforce it structurally — `cargo xtask source-structure-check`
  (module size, forbidden filenames, no Python), `unreachable_pub = "deny"`.
- [x] T-10.3 Every durable protocol has a storage port and a fault-injecting
  fake — `tests/*_storage.rs` across catalog, recovery, retention, and
  migration.

### F-11 Immutable segment format and verified I/O

**Status:** Done (ADR-0005, issues #14 and #15, M3).

`keep.segment-store/v1` segments: a 64-byte header, complete typed records
(chunk or flat layout) each with a 112-byte header and a 32-byte checksum,
and a 128-byte seal carrying the physical segment digest. `StagedSegment`
writes only content-admitted records; `SealedSegment` and `ClosedSegment`
are distinct consuming types; `AdmittedSegment` exposes payloads only after
complete framing, checksum, and identity verification.

- [x] T-11.1 Specify the protocol as one inseparable triple of bytes, crash
  states, and recovery — ADR-0005; `docs/formats/segment-store-v1/`.
- [x] T-11.2 Implement codecs, writer, and reader — `KEEP-SEGMENT-001` to
  `-010`; `fuzz/fuzz_targets/segment_format.rs`.
- [x] T-11.3 Deterministic fault injection at every write phase —
  `tests/segment_writer/`, `tests/segment_filesystem_stage.rs`.

### F-12 Catalog generations and writer-locked publication

**Status:** Done (issue #16, M3).

Immutable, generation-numbered catalogs map logical record identity to
physical location without making location part of identity. A 128-byte
`HEAD` names exactly one catalog and is the only file version 1 replaces
in place. One writer holds kernel advisory locks on the store root and
`writer.lock`; readers retain one complete generation and never mix two.

- [x] T-12.1 Catalog and head codecs, ordering, successor proofs —
  `KEEP-CATALOG-001` to `-006`, `-011`.
- [x] T-12.2 Writer exclusion and platform-admitted publication —
  `KEEP-CATALOG-007`, `-008`; `FilesystemWriterLock`,
  `FilesystemCatalogPublisher`, `publish_catalog_generation`.
- [x] T-12.3 Restart snapshot and model agreement — `KEEP-CATALOG-009`,
  `-010`; `FilesystemCatalogSnapshot`.

### F-13 Store initialization, recovery, and the crash matrix

**Status:** Done (issue #17, M3).

Production initialization admits only a writable, non-casefolded Linux
ext4 root on a single local host. Opening is observational; recovery is
explicit and planned against storage ports: inventory, name classification,
stage fingerprint and assessment, then discard, completion, next-head
finalization, or segment resume. `cargo xtask durability-crash-matrix`
kills real writer processes before, during, and after `KEEP-CRASH-001` to
`-035` (105 cases) and asserts the store lands in exactly one documented
lawful state.

- [x] T-13.1 Ordered, idempotent, writer-locked initialization —
  `KEEP-RECOVERY-002` to `-004`; `initialize_store`.
- [x] T-13.2 Recovery inventory, classification, fingerprint, assessment —
  `KEEP-RECOVERY-005` to `-012`.
- [x] T-13.3 Discard, completion, next-head finalization, segment resume —
  `KEEP-RECOVERY-013` to `-020`.
- [x] T-13.4 Process-death crash matrix — `KEEP-RECOVERY-021`;
  `xtask/src/durability_crash_matrix/`; ADR-0006 and ADR-0007 for the
  child-process boundary.

What the matrix does not prove: host power loss, torn media writes, or a
filesystem that violates the admitted atomicity contract. See F-41 T-41.2.

### F-14 Segment store v1 living documentation refresh

**Status:** Planned (#69, P2, M4).

`docs/formats/segment-store-v1/README.md` and `publication.md` still say
initialization, platform admission, and explicit recovery are future work
owned by #17, and that #16 "does not implement admission/recovery". Both
issues are complete on `main`. The v1 pages understate shipped guarantees
and hand version-2 migration a stale source boundary.

- [ ] T-14.1 Reconcile every v1 page with `main`.
  - **Requirements:** every living v1 page describes current behaviour;
    historical scope stays reachable through linked issues, ADRs, and Git
    history; every existing requirement identifier and test name remains
    an evidence anchor; `recovery.md` sentence "Transitive publication-view
    admission and filesystem-streaming semantic classification remain
    unimplemented" is either evidenced or moved to a gap with an owner.
  - **Acceptance criteria:** no v1 page assigns implemented behaviour to a
    future issue; `xtask` written-contract tests that pin protocol phrases
    still pass or are updated in the same PR.
  - **Scope:** in — `docs/formats/segment-store-v1/*`. Out — v1 bytes, v2
    pages, reorganizing unrelated docs.
  - **User stories:** Human — a reader of the v1 format learns what
    recovery does today. API user — a caller finds the recovery entry
    points named on the page that describes their crash states. MCP user —
    none. Agent — an agent implementing an adapter does not re-implement
    recovery that already exists.
  - **Interface:** none.
  - **Contract schema:** none.
  - **Test plan:** `cargo xtask documentation-integrity-check`,
    `cargo xtask documentation-refusal-check`, the
    `xtask/tests/*_contract.rs` phrase pins.
  - **Definition of done:** #69 closed; CHANGELOG "Changed" entry.
  - **Complexity:** S.
  - **Documentation:** this task is documentation.
  - **Dependencies:** none. Should close before the v2 pages become the
    primary format route.

### F-15 Retention roots, release, and GC liveness model

**Status:** Done (ADR-0009, issue #18, M4). This is a decision, not an
implementation; F-16 through F-22 implement it.

Caller-supplied opaque namespaces (1 to 255 bytes, at most 4,096 per store)
each hold a generation-checked root of reconstruction anchors
(`BlobId` plus `LayoutId`). A global manifest binds every namespace to its
root under one liveness generation. Release publishes a successor
generation that omits an anchor; it promises no erasure. Grace is an
explicit anchor in a dedicated namespace, never a clock. GC plans from an
immutable liveness snapshot, retires whole segments only, and holds writer
authority plus an exclusive reader fence.

- [x] T-15.1 Decide namespaces, generations, anchors, closure, release,
  grace, liveness snapshots, GC, dispositions — ADR-0009.
- [x] T-15.2 Reject Git refs, leases, reference counts, unversioned
  tracing, caller-supplied physical closure, and clock-based grace —
  ADR-0009 "Alternatives considered".

Deferred by the ADR: a representation-aware retention policy ("retain every
representation") as a future extension; an ABA-safe successor protocol to
raise the 4,096 namespace ceiling or reclaim tombstones.

### F-16 Version-2 format records and codecs

**Status:** Done (issue #19, PR #78; `KEEP-RETENTION-003` completed on
this roadmap's branch).

`keep.segment-store/v2` adds a 96-byte `FORMAT` marker, 256-byte migration
intent and receipt, root-generation records (192-byte header, 119-byte
anchors, digest, checksum), a global manifest (160-byte header, 72-byte
entries), a 144-byte `retention/HEAD`, and reserved grammars for GC intent,
GC receipt, and recovery disposition receipts. Every record has a
domain-separated digest and checksum, a frozen golden fixture, and a
decoder that refuses every structural fault before admission.

- [x] T-16.1 Freeze the definition and corpus —
  `conformance/segment-store/v2/definition.tsv`; format-definition digest
  `32381f1a…3427`.
- [x] T-16.2 Retention values and codecs — `KEEP-RETENTION-001`, `-002`;
  `RetentionNamespace`, `RootGeneration`, `LivenessGeneration`,
  `RetentionAnchor`, `CanonicalRetentionRoot`, `CanonicalRetentionManifest`,
  `CanonicalRetentionHead`.
- [x] T-16.3 Marker, intent, and receipt codecs — `KEEP-MIGRATION-002`;
  `fuzz/fuzz_targets/migration_format.rs`.
- [x] T-16.4 Seeded `retention_format` fuzz target.
- [x] T-16.5 Complete the corruption matrix (`KEEP-RETENTION-003`) —
  `tests/retention_root_decoding/mutation_laws.rs`,
  `tests/retention_manifest_codec/mutation_laws.rs`,
  `tests/retention_head_codec/mutation_laws.rs`. Original task fields:
  - **Requirements:** every structural field of root, manifest, and head
    has a permanent mutation case with the exact typed refusal it must
    produce; no field is covered only by the fuzz target.
  - **Acceptance criteria:** the ledger row moves from "In progress in
    #19" to Implemented naming the mutation test modules; a deliberately
    weakened decoder fails at least one case per field.
  - **Scope:** in — `tests/retention_root_decoding.rs`,
    `tests/retention_manifest_codec.rs`, `tests/retention_head_codec.rs`
    and a mutation table. Out — format changes.
  - **User stories:** Human — a maintainer can point at the test that
    refuses a flipped bit in a manifest entry. API user — decode errors
    name the field. MCP user — none. Agent — an agent adding a field knows
    the matrix it must extend.
  - **Interface:** none.
  - **Contract schema:** none.
  - **Test plan:** golden — v2 corpus; edges — every reserved byte, every
    length field at bound and bound plus one, unsorted anchors, duplicate
    namespace digests, generation zero and overflow; known failures —
    substituted digest with valid checksum; fuzz — existing target seeded
    from the corpus.
  - **Definition of done:** ledger row Implemented.
  - **Complexity:** S.
  - **Documentation:** ledger row.
  - **Dependencies:** none.

### F-17 One-way migration from version 1 to version 2

**Status:** Partial. The fresh forward path is Done (issue #19, PR #78).
Partial-prefix recovery, the `KEEP-CRASH-053` to `-073` matrix, and
`KEEP-MIGRATION-001`, `-004`, `-005`, `-006`, `-007`, `-008` residue remain,
gated by #97.

A complete version-2 store is entered only by migrating a version-1 store:
admit and recover v1, revalidate head, catalog, pools, root identity, and
writer authority, then execute twenty-one ordered phases that publish
`migration.intent`, create `reader.lock` and the retention, GC, and
recovery directories, publish `FORMAT`, reopen and verify the complete v2
view, and publish `migration.receipt`. Every version-1 byte is preserved.
Direct version-2 initialization is undefined. There is no downgrade.

- [x] T-17.0 Forward migration under writer authority —
  `KEEP-MIGRATION-002`, `-003`; `execute_store_migration`,
  `StoreMigrationPhase::ALL`, `FilesystemStoreMigrationAuthority`,
  `FilesystemStoreMigrationInventoryReader`;
  `FilesystemVersionTwoAdmission::reopen`.
- [ ] T-17.1 Restart-stable root identity coordinate (#97).
  - **Requirements:** the migration intent stops depending on
    `statx.stx_mnt_id`, which changes across unmount, remount, and reboot;
    either the intent format drops the mount coordinate (a format revision
    with a new golden record) or a restart-stable coordinate is defined
    (device plus inode of the root and of `FORMAT`, or filesystem UUID) with
    a specified remount re-admission rule. The `RootIdentityChanged`
    refusal and both comparison sites (`verify_root_identity` before
    mutation, `reopen` on reopen) stay.
  - **Acceptance criteria:** a decision recorded in
    `docs/formats/segment-store-v2/recovery.md` and `requirements.md`; a
    law that reopens a migrated store after a simulated remount (different
    mount id, same device and inode) and observes the decided behaviour;
    `KEEP-MIGRATION-004` evidence no longer depends on a transient
    coordinate; the v2 corpus updated if bytes change.
  - **Scope:** in — intent codec, root identity types
    (`StoreRootMountIdentity`, `StoreRootIdentityCoordinate`), reopen
    admission, corpus. Out — partial-prefix recovery itself (T-17.2);
    non-Linux identity.
  - **User stories:** Human — an operator reboots the host and the store
    reopens without refusing as "root identity changed". API user —
    `FilesystemVersionTwoAdmission::reopen` succeeds after remount and still
    refuses a store copied to another device. MCP user — a "reopen store"
    tool distinguishes "moved" from "rebooted". Agent — an agent that
    migrates a store and later resumes on a fresh boot is not locked out.
  - **Interface:** none new; `FilesystemPlatformAdmissionError::RootIdentityChanged`
    keeps its shape or gains a documented variant.
  - **Contract schema:** `CanonicalStoreMigrationIntent` bytes 0..256 per
    `definition.tsv`; a revision bumps the record version and adds a new
    `migration-intent.hex`.
  - **Test plan:** golden — new or unchanged intent fixture; edges — same
    inode different device, same device different inode, `FORMAT` replaced
    by a byte-identical file at a new inode; known failures — every current
    `RootIdentityChanged` law; fuzz — `migration_format` reseeded; soak —
    none.
  - **Definition of done:** #97 closed; rationale note explains the
    rejected alternative.
  - **Complexity:** M (S if the coordinate is simply dropped).
  - **Documentation:** `recovery.md`, `requirements.md`, `migration-crash.md`,
    v2 corpus README, CHANGELOG.
  - **Dependencies:** blocks T-17.2 and T-17.3.
- [ ] T-17.2 Partial-prefix migration recovery (`KEEP-MIGRATION-004`,
  residual #19 item 7).
  - **Requirements:** the seven-row recovery table in
    `migration-recovery.md` becomes executable: no artifact admits v1;
    intent stage only finalizes or discards the pre-effect stage; durable
    intent continues; intent plus a canonical prefix of v2 names verifies
    each and continues; complete v2 shape without marker writes the marker;
    marker without receipt reopens and publishes the receipt; exact receipt
    cleans any receipt stage and admits. Every ambiguity row (missing
    predecessor, changed v1 coordinate, out-of-order name, wrong kind or
    bytes, conflicting receipt, unknown entry, changed root identity)
    refuses before mutation with a typed value. Continuation is idempotent.
  - **Acceptance criteria:** a storage-independent planner over the golden
    records with one law per table row; a filesystem adapter that reopens
    each stage by device and inode identity; every prefix 0 through 21 and
    each mid-write truncation recovers in-process to the documented state;
    `KEEP-MIGRATION-001`, `-004`, `-005`, `-006` rows move to Implemented.
  - **Scope:** in — `src/adapters/store_migration/` recovery planner,
    executor, storage port, filesystem adapter. Out — process-death
    evidence (T-17.3); GC artifacts (F-22).
  - **User stories:** Human — an operator whose migration was interrupted
    reruns it and it finishes instead of waiting for a human. API user —
    `FilesystemStoreMigrationAuthority::recover` returns a receipt naming
    which prefix it found and what it did. MCP user — a "migrate store"
    tool is safe to retry. Agent — an agent can drive migration to
    completion without reading logs.
  - **Interface:** `recover_store_migration(...)`, trait
    `StoreMigrationRecoveryStorage`, `StoreMigrationRecoveryReceipt`,
    `StoreMigrationRecoveryError`; filesystem
    `FilesystemStoreMigrationAuthority::recover`.
  - **Contract schema:** no new durable bytes; receipt is in-memory and
    binds the observed prefix, the intent digest, and every phase executed.
  - **Test plan:** golden — each table row from the v2 corpus; edges —
    prefix boundaries at each of the 21 phases, truncated `intent.next`,
    `FORMAT.next` complete but unlinked, receipt stage with wrong intent
    digest; known failures — every ambiguity row; fuzz — recovery planner
    over mutated inventories; stress — a store with 2,097,152 inventory
    entries recovers within the inventory ceiling.
  - **Definition of done:** ledger rows Implemented; README gap table row
    for migration recovery removed.
  - **Complexity:** L.
  - **Documentation:** `migration-recovery.md` Status, `recovery.md`,
    `requirements.md`, CHANGELOG.
  - **Dependencies:** needs T-17.1. Blocks T-17.3, F-43.
- [ ] T-17.3 Migration crash matrix `KEEP-CRASH-053` to `-073`
  (`KEEP-MIGRATION-007`).
  - **Requirements:** real writer processes killed before, during, and
    after each of the 21 boundaries (`KEEP-CRASH-060` needs one case per
    admitted directory-prefix length); restart runs T-17.2 and the forward
    retry reports the predicted outcome; the matrix runs in debug and
    optimized xtask profiles; no sleeps, wall-clock, or scheduler luck.
  - **Acceptance criteria:** `cargo xtask durability-crash-matrix` covers
    001 to 073 plus retention (T-18.2); every case asserts catalog
    visibility, `FORMAT` presence, intent and receipt state, directory
    set, and recovery report; `KEEP-MIGRATION-007` and `-008` Implemented.
  - **Scope:** in — `xtask/src/durability_crash_matrix/`, fault-injecting
    port decorators behind `repository-tasks`. Out — power loss.
  - **User stories:** Human — CI proves migration survives being killed at
    every step. API user — none directly. MCP user — none. Agent — an agent
    can trust the migration retry rule because it is machine-verified.
  - **Interface:** `cargo xtask durability-crash-matrix --sequence migration`.
  - **Contract schema:** `conformance/segment-store/v2/transitions.tsv`
    listing 053 to 073 with pre-state, interrupted class, post-state,
    recovery posture (mirrors the v1 table).
  - **Test plan:** golden — transitions table; edges — kill during
    directory sync, kill between link and stage removal; known failures —
    none expected; stress — the full matrix under CI's ten-second deadline.
  - **Definition of done:** CI green on the extended matrix; #19 residue
    closed.
  - **Complexity:** L.
  - **Documentation:** `migration-crash.md` "does not yet claim crash
    recovery" removed; README "Proven restart recovery" bullet updated.
  - **Dependencies:** needs T-17.2.

### F-18 Retention publication

**Status:** Partial. Forward publication is Done (issue #19, PR #78).
Recovery and the `KEEP-CRASH-036` to `-052` matrix are In review (PR #99).
Explicit disposition of complete orphans waits for F-22.

A retain or release names a namespace, an expected state (absent or an
exact `RootGeneration`), a complete anchor set, and the realization
profile. Publication recovers every fixed retention stage, admits the
current head, manifest, and root, compares generations, verifies the
closure against the pinned catalog, then executes seventeen ordered
durability phases: stage `root.next`, link the root into its pool, stage
`manifest.next`, link the manifest, stage `head.next`, atomically replace
`retention/HEAD`, and remove the stages only after the head commits. It
returns a receipt binding every coordinate. It refuses retained stages,
superseded candidates, substituted files, replaced protocol directories,
and every namespace or capacity violation before writing anything.

- [x] T-18.0 Forward publication with filesystem authority —
  `KEEP-RETENTION-004`, `-005`, `-009`; `execute_retention_publication`,
  `RetentionPublicationPhase` (17), `FilesystemRetentionPublicationAuthority`,
  `RetentionCurrentStateRefusal`.
- [ ] T-18.1 Retention publication recovery (`KEEP-RETENTION-007`; PR #99).
  - **Requirements:** truncated stage with no later effect is discarded;
    complete root or manifest stage is linked into its pool and retained
    as a recovery-protected orphan, and publication refuses until
    disposition; complete head over linked stages is finalized and both
    stages removed; stages the published head already names are cleaned
    up; anything else refuses with a typed value before any effect.
    `verify_current` runs recovery first.
  - **Acceptance criteria:** storage-independent planner with one law per
    classification; `FilesystemRetentionPublicationAuthority::recover`
    reopens stages by identity; every prefix 0 through 18 and each
    mid-write truncation recovers in-process; `RecoveryRefused` and
    `RecoveryStepRefused` are distinguishable.
  - **Scope:** in — `src/adapters/retention/` recovery planner, executor,
    storage port, filesystem adapter. Out — orphan disposition (F-22).
  - **User stories:** Human — an operator whose retain was interrupted
    reruns it and it either finishes or names the orphan that needs a
    decision. API user — `recover` returns a receipt naming each stage and
    its disposition. MCP user — a "retain" tool is idempotent across
    crashes. Agent — an agent can retry retention without inspecting the
    directory.
  - **Interface:** `execute_retention_recovery(...)`, trait
    `RetentionRecoveryStorage`, `FilesystemRetentionPublicationAuthority::recover`.
  - **Contract schema:** none new; in-memory receipt.
  - **Test plan:** golden — recovery planning laws over the v2 golden
    records; edges — each of the three stage files complete, truncated,
    absent, and byte-identical at a new inode; known failures — every
    "anything else" refusal; fuzz — none new; stress — none.
  - **Definition of done:** PR #99 merged; ledger row Implemented.
  - **Complexity:** L (delivered in PR #99).
  - **Documentation:** `recovery.md` "Retention publication recovery"
    table marked implemented; CHANGELOG.
  - **Dependencies:** none. Blocks F-22 orphan disposition, F-43.
- [ ] T-18.2 Retention crash matrix `KEEP-CRASH-036` to `-052` (PR #99).
  - **Requirements:** real process death before, during, and after each
    of the 17 phases (51 coordinates); restart recovers and the forward
    retry reports the predicted outcome; full matrix green in debug and
    release.
  - **Acceptance criteria:** 156-case combined matrix green in CI;
    `KEEP-RETENTION-006` crash-injection remainder closed.
  - **Scope:** in — `xtask/src/durability_crash_matrix/`. Out — migration
    coordinates (T-17.3).
  - **User stories:** as T-17.3, for retention.
  - **Interface:** `cargo xtask durability-crash-matrix`.
  - **Contract schema:** `conformance/segment-store/v2/transitions.tsv`
    rows 036 to 052.
  - **Test plan:** as T-17.3.
  - **Definition of done:** PR #99 merged; README "waits for a human"
    paragraph rewritten to name only orphan disposition.
  - **Complexity:** L (delivered in PR #99).
  - **Documentation:** `recovery.md`, README gap table.
  - **Dependencies:** T-18.1.
- [ ] T-18.3 Closure-member re-verification under filesystem authority
  (`closure.md` Status; `KEEP-RETENTION-006` source-chain obligation).
  - **Requirements:** when recovery or publication re-reads a
    closure-member segment under authority, the original decode or
    admission error travels as the `source` of the operation-level error;
    no wrapping erases it.
  - **Acceptance criteria:** a law downcasts through the publication error
    to the exact `SegmentRecordAdmissionError` that caused it.
  - **Scope:** in — error wrapping in the filesystem retention authority.
    Out — verification reports (F-21).
  - **User stories:** Human — a refused retain says which segment record
    failed and why. API user — `source()` chains are complete. MCP user —
    the tool error names the record. Agent — an agent can route the
    failure to the right remediation.
  - **Interface:** none new.
  - **Contract schema:** none.
  - **Test plan:** golden — none; edges — corrupt chunk record, corrupt
    layout record, missing member; fuzz — none.
  - **Definition of done:** `closure.md` Status updated.
  - **Complexity:** S.
  - **Documentation:** `closure.md`, `closure-corruption.md`.
  - **Dependencies:** T-18.1.

### F-19 Reader fence and immutable version-2 snapshots

**Status:** In review (PR #99; `KEEP-RETENTION-008`).

`reader.lock` is a persistent, zero-length regular file whose existence
and contents prove nothing. A version-2 reader acquires a kernel-managed
shared lock on it before opening the catalog `HEAD` or `retention/HEAD`;
the returned `ReaderFence` owns the lock for the snapshot's lifetime and
releases only the lock, never the file. Readers double-collect both heads
around complete transitive admission and accept only identical coordinates
before and after, retrying within a bounded attempt limit. GC takes writer
authority then the exclusive reader lock, in that order; publication never
waits on readers because it deletes nothing.

- [ ] T-19.1 `ReaderFence`, `collect_retention_view`,
  `FilesystemRetentionSnapshot` (PR #99).
  - **Requirements:** shared lock acquired before either head is opened;
    fence released on drop or process death without deleting `reader.lock`;
    double-collect compares catalog generation and digest plus retention
    liveness generation and manifest digest; bounded retries; exhaustion
    refuses with a typed value; the snapshot binds one catalog, one
    manifest, and every root generation the manifest names.
  - **Acceptance criteria:** nine laws (per PR #99): fence before head,
    identical-coordinates acceptance, changed-coordinates retry, retry
    exhaustion refusal, fence survives publication, exclusive acquisition
    blocks new readers, drop releases, process death releases, file never
    deleted.
  - **Scope:** in — `src/adapters/retention/` snapshot and fence. Out —
    GC's exclusive acquisition (F-22), durable read receipts (F-23).
  - **User stories:** Human — an operator can run a verification pass
    while a writer publishes and see one consistent view. API user —
    `FilesystemRetentionSnapshot::open(root)` returns a view whose
    coordinates are named on the receipt. MCP user — a "snapshot store"
    tool returns a handle that later reads bind to. Agent — an agent's
    long-running audit is not invalidated by concurrent retains.
  - **Interface:** `ReaderFence`, `collect_retention_view`,
    `FilesystemRetentionSnapshot::{open, catalog, manifest, root}`.
  - **Contract schema:** none new; `reader.lock` semantics in
    `recovery.md`.
  - **Test plan:** golden — none; edges — publication between the two
    collections, `reader.lock` missing (refuse; it is created by
    migration), `reader.lock` replaced by a directory or symlink; known
    failures — retry exhaustion; concurrency — two readers and one writer
    under `cfg(target_os = "linux")`; stress — a reader held across a full
    crash-matrix run.
  - **Definition of done:** PR #99 merged; `KEEP-RETENTION-008`
    Implemented; README gap table row removed.
  - **Complexity:** M (delivered in PR #99).
  - **Documentation:** `recovery.md` "Reader fence" Status; ADR-0009
    consequence satisfied; CHANGELOG.
  - **Dependencies:** none. Blocks F-22 and F-23.

### F-20 Model-based retention transition evidence

**Status:** In review (PR #99; `KEEP-RETENTION-010`).

Every three-operation sequence of retain, release, and re-read across
namespaces agrees with a deterministic namespace-to-anchor-set map
observed through the fenced view, and a source-architecture contract keeps
clocks, paths, environment, and caller identity out of the core.

- [ ] T-20.1 125 three-operation sequences against the model (PR #99).
  - **Requirements:** the model is a `BTreeMap<RetentionNamespace,
    BTreeSet<RetentionAnchor>>` with generation counters; each sequence
    compares the fenced view with the model after every step; the source
    contract greps `src/retention/` and the core `src/adapters/retention/`
    planners for `std::time`, `std::env`, `std::path`, and process
    identity.
  - **Acceptance criteria:** the ledger row names the test module; a
    deliberately wrong transition planner fails at least one sequence.
  - **Scope:** in — `tests/retention_model.rs` or equivalent. Out —
    four-operation sequences (property tests may extend later).
  - **User stories:** Human — a maintainer trusts that retain and release
    compose. API user — none directly. MCP user — none. Agent — an agent
    composing retention operations relies on documented sequence laws.
  - **Interface:** none.
  - **Contract schema:** none.
  - **Test plan:** golden — none; edges — retain then release same anchor,
    release absent anchor, retain at stale generation, empty anchor set;
    property — random sequences up to length 8 as a follow-up.
  - **Definition of done:** PR #99 merged; `KEEP-RETENTION-010` Implemented.
  - **Complexity:** M (delivered in PR #99).
  - **Documentation:** ledger row; v2 README Status.
  - **Dependencies:** F-19.

### F-21 Precise verification reports and corruption refusal

**Status:** Planned (#20, P1, M4). The most-cited open blocker: F-22,
F-23, F-24, F-27, F-28, F-29, F-30, F-31, F-33, and F-34 all name it.

Verify content and store structure at explicit, enumerated depths; report
exactly what was established and nothing more; refuse when evidence is
missing, conflicting, or corrupt. Verification never repairs, substitutes,
quarantines, or rewrites physical state.

- [ ] T-21.1 Verification policy and report types.
  - **Requirements:** policy is an enum of depths, never a set of boolean
    flags: framing, checksum, chunk identity, layout identity, complete
    blob identity, catalog reachability, retention-root closure, and (once
    F-19 lands) snapshot binding; a report states the depth reached per
    subject and can never present partial verification as complete;
    typed failures retain expected and observed identities, lengths,
    generations, and format versions; missing, corrupt, and ambiguous
    (conflicting) are distinct; reports contain no plaintext, keys, or
    unbounded paths.
  - **Acceptance criteria:** `VerificationDepth` enum; `VerificationReport`
    with one `VerifiedSubject` per subject naming the depth established;
    `VerificationRefusal` with `Missing`, `Corrupt { expected, observed }`,
    `Ambiguous { candidates }` variants; a compile-time law that a report
    at depth N cannot be converted into one at depth N+1.
  - **Scope:** in — a `verification` core module and its adapters over
    segment, catalog, layout, and retention readers. Out — repair (never),
    remote attestation, application trust decisions, GC (F-22).
  - **User stories:** Human — an operator asks "is this store sound to
    depth X?" and gets a report they can file, not a boolean. API user —
    `verify(store, policy)` returns a typed report with a subject list.
    MCP user — a "verify store" tool with a depth parameter returns a
    structured report the client can render. Agent — an agent decides
    whether to retain, migrate, or escalate from the report's typed
    refusals, without parsing prose.
  - **Interface:** `verify_blob(view, BlobId, VerificationDepth)`,
    `verify_catalog(view, VerificationDepth)`,
    `verify_retention(view, RetentionNamespace, VerificationDepth)`; an
    adapter would expose `keep verify --depth <depth> [--blob <id>]` and an
    MCP tool `keep.verify { depth, subject }`.
  - **Contract schema:** in-memory types only in this task. A durable
    report format is T-21.3.
  - **Test plan:** golden — a report over the v1 and v2 corpora at every
    depth; edges — empty store, depth beyond what the view supports
    (refuse, not degrade), subject absent versus subject corrupt; known
    failures — every existing corruption law must map to exactly one
    refusal variant; fuzz — report decoder once T-21.3 exists; stress —
    verifying a store at the catalog entry ceiling within the documented
    memory bound.
  - **Definition of done:** types merged with rustdoc stating memory, I/O,
    and complexity per depth.
  - **Complexity:** M.
  - **Documentation:** new `docs/invariants/verification/` page; ADR-0009
    consequence ("report the exact verification depth") satisfied.
  - **Dependencies:** none for the reference store; F-19 for
    snapshot-bound depths.
- [ ] T-21.2 Permanent corruption matrix over every durable structural
  field.
  - **Requirements:** every field of segment header, record header, record
    checksum, seal, catalog header, entry, trailer, publication head,
    `FORMAT`, intent, receipt, root, manifest, and retention head has a
    named mutation whose refusal variant and depth are asserted through
    the report, not only through the decoder.
  - **Acceptance criteria:** a mutation table per format under
    `conformance/`; the Golden File Worldline `corruption refusal`
    capability row flips from `declared-future`.
  - **Scope:** in — tests and corpora. Out — new formats.
  - **User stories:** Human — a maintainer sees which byte a refusal is
    about. API user — none new. MCP user — none. Agent — an agent adding
    a field extends a table, not a prose list.
  - **Interface:** `cargo xtask conformance-check` extended.
  - **Contract schema:** `mutations.tsv` per format.
  - **Test plan:** golden — the tables; edges — multi-field mutations must
    report the first refusal in the documented check order; fuzz —
    existing decoder targets; stress — none.
  - **Definition of done:** Worldline capability row executable.
  - **Complexity:** M.
  - **Documentation:** each format README "Mutation ledger".
  - **Dependencies:** T-21.1.
- [ ] T-21.3 Durable refusal and verification receipts.
  - **Requirements:** a canonical, versioned, checksummed report record
    binding `BlobId`, admitted view (catalog generation and digest,
    liveness generation and manifest digest), exact `LayoutId` if present,
    refusal classification, proof stage, and contract version; no key
    material or plaintext; bounded length.
  - **Acceptance criteria:** golden fixture; decoder refuses every
    structural fault; a receipt written by one process is admitted by
    another.
  - **Scope:** in — record format, codec, corpus. Out — where receipts are
    stored (application choice; Keep does not persist them itself).
  - **User stories:** Human — an operator attaches a receipt to an incident
    ticket. API user — receipts serialize without Serde-defined bytes. MCP
    user — the tool returns the receipt bytes and its text form. Agent —
    an agent hands a receipt to another agent and both agree on what it
    proves.
  - **Interface:** `CanonicalVerificationReceipt::{encode, decode}`.
  - **Contract schema:** `KEEP:VERIFY:RCPT` magic, version, depth, subject
    kind, subject identity slot (60 bytes), view coordinates, refusal
    variant, checksum; exact layout in a new
    `docs/formats/verification-receipt-v1/`.
  - **Test plan:** golden — fixture; edges — every reserved byte; known
    failures — a range-read receipt must not decode as a complete-object
    receipt; fuzz — new `verification_receipt` target.
  - **Definition of done:** `KEEP-RECONSTRUCT-006` "durable refusal
    receipts planned" resolved.
  - **Complexity:** M.
  - **Documentation:** new format page; `docs/formats/README.md` registry
    row.
  - **Dependencies:** T-21.1; F-19 for view coordinates.

### F-22 Garbage collection, compaction, and recovery dispositions

**Status:** Planned (#21, P1, M4). Grammars are frozen and their presence
refuses (`KEEP-GC-001`, `-002`).

Plan GC from an immutable liveness snapshot; classify every segment as
live, unreachable, corrupt, ambiguous, recovery-protected,
reader-protected, or already retired; retire only whole immutable segments
that the current catalog no longer names; compact mixed segments by
copying and verifying live records into new segments and publishing a
catalog successor first; hold writer authority then the exclusive reader
lock; write and sync `gc/intent` before any unlink; unlink candidates in
canonical order with a directory sync after every one; publish
`gc/receipt` after all are absent. A verified orphan from an interrupted
publication stays recovery-protected until an explicit finalize-or-retire
disposition receipt exists. `BlobId`, `ChunkId`, and `LayoutId` never move.

- [ ] T-22.1 GC record codecs and namespace admission (`KEEP-GC-001`).
  - **Requirements:** `GcRetirementIntent` (320-byte header, 72-byte
    candidates, at most 65,536, digest, checksum), `GcRetirementReceipt`
    (320 bytes), `RecoveryDispositionReceipt` (320 bytes) encode and decode
    exactly per `gc.md`; presence of any such artifact without the
    implementing recovery still refuses until T-22.4.
  - **Acceptance criteria:** golden `.hex` fixtures added to
    `conformance/segment-store/v2/`; decoders refuse every structural
    fault; `gc_format` fuzz target seeded.
  - **Scope:** in — codecs, corpus, fuzz. Out — execution.
  - **User stories:** Human — none yet. API user — the types exist so
    tooling can inspect an intent left by a crash. MCP user — none. Agent —
    none.
  - **Interface:** `CanonicalGcRetirementIntent`, `AdmittedGcRetirementIntent`,
    and receipt equivalents.
  - **Contract schema:** as `gc.md`; domains `keep.gc-candidate-set/v2`,
    `keep.gc-retirement-intent/v2`, `keep.gc-retirement-receipt-checksum/v2`,
    `keep.recovery-disposition-receipt-checksum/v2`.
  - **Test plan:** golden — fixtures; edges — candidate count 0 and
    65,537, unsorted candidates, reader-lock identity zero; fuzz — new
    target.
  - **Definition of done:** `KEEP-GC-001` Implemented.
  - **Complexity:** M.
  - **Documentation:** `gc.md` Status; corpus README.
  - **Dependencies:** none.
- [ ] T-22.2 Deterministic `GcPlan` from a liveness snapshot.
  - **Requirements:** input is one immutable snapshot (F-19): manifest
    generation and digest, complete namespace map, every anchor and
    closure, every profile coordinate, catalog generation and digest,
    traversal limits; planning is pure and observational; output is an
    immutable, inspectable, `#[must_use]` plan classifying every segment;
    no segment is a candidate while the current catalog names any record
    in it; ambiguous or corrupt state is refused, never collected.
  - **Acceptance criteria:** the plan for the golden v2 store is a golden
    fixture; a model test proves that live sets before and after planning
    are identical; every classification has a law.
  - **Scope:** in — `src/retention/gc/` planner. Out — execution
    (T-22.4), compaction (T-22.3).
  - **User stories:** Human — an operator runs a dry run and reads exactly
    which segments would go and why. API user — `plan_gc(snapshot)` returns
    a `GcPlan` they can inspect before executing. MCP user — a "plan gc"
    tool returns the plan as data; nothing changes on disk. Agent — an
    agent reviews the plan's ambiguity list and refuses to proceed if
    nonempty.
  - **Interface:** `plan_gc(&RetentionSnapshot, GcLimits) -> Result<GcPlan,
    GcPlanError>`; adapter `keep gc plan` and MCP `keep.gc.plan`.
  - **Contract schema:** in-memory `GcPlan`; the intent record is derived
    from it in T-22.4.
  - **Test plan:** golden — plan fixture; edges — empty store, store with
    only orphans, segment shared between live and released anchors; known
    failures — a segment named by a retained stage; property — random
    retain and release histories, then plan, then assert the live closure
    is untouched; stress — planning at the closure node ceiling.
  - **Definition of done:** planner merged with its model test.
  - **Complexity:** L.
  - **Documentation:** `gc.md` planning section; a warning per
    Documentation Standards §5.4 on every page that describes execution.
  - **Dependencies:** F-19, F-21 (planning consumes verification depth).
- [ ] T-22.3 Identity-preserving compaction.
  - **Requirements:** copy live records into new immutable segments,
    verify them, publish a catalog successor naming the new locations,
    revalidate expected catalog and retention generations before acting,
    keep old segments readable until the successor is durable; `BlobId`,
    `ChunkId`, `LayoutId` stable; every step is a named crash point.
  - **Acceptance criteria:** model test proves reads and retention sets are
    equivalent before and after; crash injection covers copy, verify,
    publication, retirement, deletion, and recovery of compaction;
    benchmarks report amplification, sync count, reclaimed bytes, latency,
    and peak temporary space.
  - **Scope:** in — compaction planner, executor, storage port, filesystem
    adapter, crash points `KEEP-CRASH-074` onward. Out — re-encoding
    (representation change) which waits for F-28 or F-39.
  - **User stories:** Human — an operator reclaims space from a store
    whose segments are half released. API user — `compact(plan)` returns a
    receipt naming the new catalog generation. MCP user — "compact" is
    refused unless a dry-run plan id is supplied. Agent — an agent
    schedules compaction only when the plan's reclaimable bytes exceed a
    threshold it computes.
  - **Interface:** `execute_compaction(...)`, trait `CompactionStorage`.
  - **Contract schema:** none new beyond the catalog successor.
  - **Test plan:** golden — a compacted golden store; edges — nothing to
    compact, everything live, a segment at the 1 GiB ceiling; known
    failures — crash after new segment sealed but before catalog
    published leaves a valid orphan, not a loss; crash matrix — every new
    crash point; soak — repeated compaction cycles preserve every identity.
  - **Definition of done:** Worldline `compaction stability` row executable.
  - **Complexity:** L.
  - **Documentation:** `gc.md` compaction section; ADR-0002 compaction
    example cross-linked.
  - **Dependencies:** T-22.2.
- [ ] T-22.4 GC execution, retirement, and recovery (`KEEP-GC-002`).
  - **Requirements:** writer authority then exclusive `reader.lock`;
    intent written, flushed, synced before any unlink; canonical order;
    per-unlink directory sync; receipt after all absent; a retained intent
    makes admission recovery-required and excludes catalog publication,
    retention transitions, and another GC until resolved; recovery
    resolves idle, active, partial, completion-pending, receipt-transition,
    and complete states and refuses everything else.
  - **Acceptance criteria:** crash points for every GC boundary; the GC
    state table in `gc.md` executable; a store cannot lose a live segment
    under any crash prefix (model test over the matrix).
  - **Scope:** in — executor, storage port, filesystem adapter, recovery,
    crash matrix sequence. Out — background scheduling policy; secure
    erasure.
  - **User stories:** Human — an operator runs GC once and, if the host
    dies, reruns it to completion. API user — `execute_gc(intent)` is
    idempotent per intent. MCP user — "gc execute" requires the plan id
    and reports the receipt. Agent — an agent never runs GC without a
    fresh plan whose snapshot coordinates match the current heads.
  - **Interface:** `execute_gc(...)`, `recover_gc(...)`, trait `GcStorage`.
  - **Contract schema:** `gc/intent`, `gc/receipt` per T-22.1.
  - **Test plan:** golden — intent and receipt for the golden store;
    edges — zero candidates (refuse: nothing to do is not an intent),
    candidate already absent before intent (ambiguity); crash matrix —
    before, during, after each boundary; concurrency — a reader holding
    the fence blocks GC until it drops; stress — 65,536 candidates.
  - **Definition of done:** `KEEP-GC-002` Implemented; README gap table
    row removed; a Documentation Standards §5.4 warning on every GC page.
  - **Complexity:** XL across T-22.2 to T-22.5.
  - **Documentation:** `gc.md`, `recovery.md`, `requirements.md`, CHANGELOG.
  - **Dependencies:** T-22.1, T-22.2, F-19.
- [ ] T-22.5 Explicit orphan disposition.
  - **Requirements:** a finalize-or-retire decision for a recovery-protected
    orphan is durable as `recovery/dispositions/<digest>.receipt` via the
    fixed-stage protocol; retirement proves the artifact is named by no
    `HEAD`, no fixed stage, no pending publication, no retained closure,
    and no active reader; at most 65,536 receipts.
  - **Acceptance criteria:** publication that was refusing on an orphan
    proceeds once a disposition exists; GC admits only the exact receipt.
  - **Scope:** in — disposition planner, executor, adapter. Out —
    automatic disposition (a human or an explicit policy decides).
  - **User stories:** Human — the "waits for a human" case becomes one
    command with a stated consequence. API user — `dispose(orphan,
    Decision)` returns the receipt. MCP user — "dispose orphan" demands the
    decision and the orphan digest. Agent — an agent may finalize (safe)
    without escalation but must escalate retire.
  - **Interface:** `plan_recovery_disposition`, `execute_recovery_disposition`.
  - **Contract schema:** `RecoveryDispositionReceipt` per T-22.1.
  - **Test plan:** golden — receipt fixture; edges — dispose an artifact
    the head now names (refuse), dispose twice (idempotent), receipt
    ceiling; crash matrix — stage, link, sync, remove boundaries.
  - **Definition of done:** README "waits for a human" paragraph removed.
  - **Complexity:** M.
  - **Documentation:** `recovery.md` dispositions section with a §5.4
    warning.
  - **Dependencies:** T-18.1, T-22.1.

### F-23 Durable authenticated reads and refusal receipts

**Status:** Planned. `KEEP-RECONSTRUCT-009` and `-010` cite #22 and #23,
which are closed; no open Keep issue owns this. Open one.

The durable segment, catalog, publication, and recovery surfaces do not
yet form one high-level `BlobId`-to-writer contract. A durable read must
bind to one admitted immutable snapshot, prevent its evidence from being
collected during the read, verify the retained closure, resolve exact
immutable records, preserve the view while successors publish, and return
a receipt naming the view, with refusal distinct from operational failure
and no hidden whole-blob allocation.

- [ ] T-23.1 `DurableStore` read surface over a fenced snapshot.
  - **Requirements:** `reconstruct`, `reconstruct_layout`, `read_range`
    with the same laws as `ReferenceStore` but bound to a
    `FilesystemRetentionSnapshot`; receipts gain the view coordinates;
    chunk loads stream from admitted segment records with bounded memory;
    a snapshot dropped mid-read is impossible by construction (the read
    borrows it).
  - **Acceptance criteria:** the Worldline restart and range assertions run
    against the durable backend; `KEEP-RECONSTRUCT-009`, `-010`
    Implemented; every `ReferenceStore` read law has a durable twin.
  - **Scope:** in — a `durable` adapter module composing snapshot, catalog
    lookup, segment record reads, layout admission, and the existing
    reconstruction core. Out — writes (F-24), verification depth beyond
    what the read needs (F-21).
  - **User stories:** Human — an operator reads a blob out of a store by
    identity with one call and gets a receipt naming the generation it
    came from. API user — `DurableStore::open(root)?.reconstruct(id,
    &mut out)?` is the durable twin of the README example. MCP user — a
    "read blob" tool returns bytes plus receipt. Agent — an agent reading
    across a store restart sees identical receipts for identical views.
  - **Interface:** `DurableStore::{open, snapshot, contains_blob,
    reconstruct, reconstruct_layout, read_range}`; adapters expose
    `keep cat <blob-id>` and MCP `keep.read { blob, range? }`.
  - **Contract schema:** `DurableReconstructionReceipt` extends
    `ReconstructionReceipt` with catalog generation and digest and
    liveness generation and manifest digest.
  - **Test plan:** golden — Worldline over the durable backend; edges —
    blob present in catalog but segment unreadable (operational failure,
    not refusal), blob whose layout names a chunk the catalog lacks
    (evidenced refusal), range across a segment boundary; known failures —
    every reference refusal law; fuzz — none new; stress — reading at the
    catalog ceiling with the documented memory bound; concurrency — reads
    during publication and during a blocked GC.
  - **Definition of done:** README "Try it" gains a durable example
    (Linux-only, marked).
  - **Complexity:** L.
  - **Documentation:** `docs/invariants/authenticated-reconstruction/README.md`
    "durable" section; `docs/architecture/durable-store/` new page.
  - **Dependencies:** F-19; F-22 T-22.4 for the "evidence cannot be
    collected" law (until then, it holds vacuously because nothing
    collects).

### F-24 Bounded production ingestion through the durable store

**Status:** Planned (#82, P1, M6). Needs #74 and #72 (F-06) and F-21.

One bounded production path from an unknown-length source through the
registered CDC profile, chunk verification and deduplication, immutable
segment publication, catalog admission, and an exact receipt. It preserves
`keep.fastcdc-gear64/v1`. Deduplication, batching, and backpressure must not
change `BlobId`, `ChunkId`, `LayoutId`, publication order, recovery, or
error precision.

- [ ] T-24.1 Backend-neutral ingestion contract.
  - **Requirements:** a trait or port that `ReferenceStore` and the durable
    writer both satisfy where their durability claims overlap: stage,
    commit, receipt; staging admits count-and-byte limits; the reference
    adapter stays honest about being non-durable.
  - **Acceptance criteria:** one integration test suite runs against both
    backends; a compile-time law that a `ReferenceStore` receipt cannot be
    passed where a durable receipt is required.
  - **Scope:** in — a `store` port module. Out — asynchronous APIs.
  - **User stories:** Human — none. API user — code written against the
    port runs in tests on the reference store and in production on disk.
    MCP user — the same tool schema regardless of backend. Agent — an
    agent's test harness and production path share one contract.
  - **Interface:** trait `ContentStore { stage, commit, reconstruct,
    read_range }` or the smallest equivalent.
  - **Contract schema:** none.
  - **Test plan:** golden — Worldline through the port; edges — every
    existing ingestion law on both backends.
  - **Definition of done:** both adapters implement the port.
  - **Complexity:** M.
  - **Documentation:** `docs/architecture/` port page.
  - **Dependencies:** T-06.4.
- [ ] T-24.2 Durable staged ingestion with deduplication.
  - **Requirements:** single pass over an unknown-length source; existing
    chunks reused only after exact identity and representation
    verification against the pinned catalog; missing chunks stream into a
    `StagedSegment` without materializing the blob; layout and complete
    `BlobId` verified before visible admission; commit publishes the
    segment and a catalog successor through the existing 26 v1 crash
    points; receipt binds profile, blob, layout, segment digests, catalog
    generation, and exact byte counts (logical, physical new, physical
    reused).
  - **Acceptance criteria:** Worldline `chunk reuse` and `production
    ingest` capability rows executable; every write boundary has short
    write, interruption, process death, and restart evidence (reusing the
    v1 matrix); benchmark reports throughput, peak memory, allocations,
    sync count, dedup ratio.
  - **Scope:** in — durable staging adapter, segment rollover at the
    1 GiB and 1,048,576-record ceilings, catalog successor publication.
    Out — new CDC algorithm; convergent encryption; retention (a caller
    retains afterwards through F-18).
  - **User stories:** Human — an operator ingests a directory of files and
    sees dedup ratio in the receipt. API user — `DurableStore::stage(&mut
    source, limits)?.commit()?` returns a receipt with byte counts. MCP
    user — an "ingest" tool accepts a path or stream and returns the
    `BlobId` and receipt. Agent — an agent ingests a build artifact and
    retains it in one namespace in two calls, both idempotent.
  - **Interface:** `DurableStore::{stage, commit}`; adapters
    `keep put <path>` and MCP `keep.ingest`.
  - **Contract schema:** `DurableIngestionReceipt`.
  - **Test plan:** golden — ingest the Worldline inputs and compare
    catalog bytes with the v1 corpus; edges — source shorter than one
    chunk, exactly at the segment ceiling, every chunk already present,
    rollover mid-blob; known failures — interrupted source discards the
    stage and leaves a reusable staged segment; crash matrix — the
    existing v1 points driven by ingestion instead of by fixtures; soak —
    ingest until the catalog entry ceiling and confirm the typed refusal;
    stress — memory ceiling law over a multi-GiB synthetic source.
  - **Definition of done:** #82 closed; README gap table row removed;
    README "Try it" durable example writes as well as reads.
  - **Complexity:** L.
  - **Documentation:** `docs/architecture/durable-store/` ingestion page
    with the memory bound; CHANGELOG.
  - **Dependencies:** T-24.1, T-06.3, T-06.4, F-18, F-21 (representation
    verification depth), F-23.
- [ ] T-24.3 Bounded streaming write-through pipeline (#72, P3).
  - **Requirements:** a source adapter emitting verified range segments, a
    sink adapter applying an exactly-once write protocol, and a transfer
    adapter coordinating a bounded chunk window with receipts and
    cancellation; zero-copy handoff of immutable chunk bytes within the
    window; cancellation never upgrades partial output into success.
  - **Acceptance criteria:** benchmark shows lower CPU and allocation than
    a caller-owned copy loop for large blobs; one read-to-write and one
    copy-to-write integration test; full identity, range correctness, and
    refusal laws preserved.
  - **Scope:** in — `ChunkPipeline` in adapters. Out — multi-threaded
    pipelines until a bounded-memory proof exists.
  - **User stories:** Human — none. API user — copying a blob between two
    stores does not buffer it. MCP user — a "copy blob" tool streams. Agent
    — an agent mirroring a namespace to a second store does so in bounded
    memory.
  - **Interface:** `transfer(source, sink, window) -> TransferReceipt`.
  - **Contract schema:** none.
  - **Test plan:** golden — copy the Worldline inputs; edges — sink fails
    mid-window, source dies mid-window, profile mismatch between stores;
    stress — window of one chunk.
  - **Definition of done:** #72 closed.
  - **Complexity:** M.
  - **Documentation:** architecture page.
  - **Dependencies:** T-24.2, F-23.

### F-25 Echo adapter and transaction boundary

**Status:** Done in the Echo repository (issues #22 and #23 closed
2026-08-15; work tracked as flyingrobots/echo#721 and #722). Keep's side
is the authenticated reconstruction contract (F-07) and the no-Echo-types
law (`KEEP-STORE-016`).

Echo owns causal meaning; Keep owns exact physical bytes. Echo never
commits a causal reference to content lacking verified physical retention.
No subprocess or Node sidecar sits in the storage path.

- [x] T-25.1 Contract: success, evidenced refusal, operational failure —
  PR #77; `docs/invariants/authenticated-reconstruction/`.
- [x] T-25.2 Adapter and cutover — echo#722 (outside this repository).

Residual Keep obligations from #22 and #23 live in F-23 (durable refusal
receipts, pinned durable reads) because those issues closed before #20
did.

### F-26 Graft Golden File Worldline end to end

**Status:** Planned (#24, P2, M5). Needs F-22 and F-25.

Prove that Graft can continuously retain and recover nearby workspace
states through Echo and Keep with no Git or Node subprocess in the storage
hot path: observe state A, admit only after stage, verify, retain; produce
state B by an early insertion; recover both by identity; show chunk reuse
without reuse being identity; range reads load only overlapping chunks; a
stale-basis write refuses; termination and restart recover a lawful state;
corruption refuses precisely; compaction preserves every identity; Git
publication changes no Keep or Echo identity.

- [ ] T-26.1 Cross-repository fixture and matrices.
  - **Requirements:** pinned Keep, Echo, and Graft revisions; the twelve
    scenario steps executable; kill-and-restart and corruption matrices
    across every cross-repository durability boundary; POSIX and
    agent-native Graft reads return the same coordinate and bytes; metrics
    for logical bytes, physical bytes, reuse, amplification, sync count,
    latency, peak memory, restart time.
  - **Acceptance criteria:** the fixture is a reusable cross-language
    conformance corpus; the demonstration distinguishes implemented
    guarantees from watcher completeness and application policy.
  - **Scope:** in — a fixture under `conformance/graft-worldline/v1/` or in
    the Graft repository with Keep pinned. Out — replacing Git publication;
    cross-organization transport; cryptographic settlement.
  - **User stories:** Human — a Graft user edits `Foo.txt`, kills the
    machine, and gets both states back byte-exact. API user — none
    directly. MCP user — a Graft MCP tool reads a workspace state by
    coordinate. Agent — an agent operating on a Graft workspace relies on
    the stale-basis refusal to avoid clobbering.
  - **Interface:** none in Keep.
  - **Contract schema:** the fixture's `steps.tsv` and `capabilities.tsv`
    in the Worldline style.
  - **Test plan:** golden — the fixture; crash matrix — every
    cross-repository boundary; corruption — every layer; benchmark — the
    listed metrics.
  - **Definition of done:** #24 closed with the fixture linked from
    `docs/conformance/`.
  - **Complexity:** XL (three repositories).
  - **Documentation:** `docs/conformance/graft-worldline.md`.
  - **Dependencies:** F-22, F-23, F-24, F-25.

### F-27 git-cas import posture

**Status:** Planned (#25, P2, M5). An ADR is the deliverable.

Decide whether and how existing git-cas assets can be imported without
letting Git representation details or legacy validation weakness enter
Keep's native contracts: which manifest and encryption versions are
accepted; streaming, staged, independently verified import; re-identification
under Keep's canonical `BlobId`; handling of missing manifest hashes,
malformed sub-manifest topology, conflicting digest and OID pairs; one-way
only; provenance and weaker-evidence posture.

- [ ] T-27.1 ADR: import now, import later, or never.
  - **Requirements:** the decision names concrete migration evidence;
    native formats stay independent of Git OIDs, trees, refs, and process
    execution; imported bytes verify against both source evidence and the
    Keep destination identity; weak legacy evidence is represented
    explicitly, never silently upgraded; interruption and restart specified;
    fixtures are license-safe with no real workspace material.
  - **Acceptance criteria:** `docs/adr/0010-git-cas-import-posture.md` (or
    the next free number) Accepted with alternatives evaluated.
  - **Scope:** in — the decision. Out — the importer (a new feature if
    chosen), bidirectional sync, treating git-cas handles as Keep identity.
  - **User stories:** Human — a git-cas user learns whether their store can
    move to Keep and what evidence they lose. API user — none until the
    importer exists. MCP user — none. Agent — an agent migrating a
    workspace knows whether to plan an import or a re-ingest.
  - **Interface:** none.
  - **Contract schema:** none.
  - **Test plan:** none; a decision record.
  - **Definition of done:** ADR merged; #25 closed.
  - **Complexity:** S for the ADR; L if import is chosen.
  - **Documentation:** the ADR; `docs/adr/README.md` index.
  - **Dependencies:** F-21 (verification depth vocabulary for "weaker
    evidence"); informed by F-28.

### F-28 Authenticated encrypted representations

**Status:** Planned (#86 ADR, P1; #83 implementation, P1; M6).

A native representation contract for authenticated encryption that
preserves `BlobId` and `LayoutId` across encryption, re-encryption,
recipient changes, and key rotation: an audited AEAD suite and nonce
strategy, independently authenticated frames with canonical AAD binding
blob, layout, representation version, frame coordinate, and declared
context; a random per-representation data-encryption key wrapped for each
recipient key-encryption key; a semantic key capability crossing the port
without importing a keychain, KMS, CLI, or application key-reference type;
key material never in durable formats, diagnostics, receipts, or logs; no
plaintext released before the authenticated boundary succeeds. Convergent
and deterministic encryption are excluded from the private-content
profile.

- [ ] T-28.1 ADR: encrypted representation contract (#86).
  - **Requirements:** freeze identity, framing, AAD, nonce, recipient
    envelope, and verification laws; define `RepresentationId` for the
    encrypted codec (first assignment of the reserved envelope, T-03.3);
    distinguish missing, unavailable, unauthorized, revoked, malformed,
    and authentication-failed key paths; review dependency, side-channel,
    zeroization, crash, backup, and recovery implications; require
    independent golden and mutation corpora before codec admission.
  - **Acceptance criteria:** ADR Accepted; a written dependency review for
    the chosen crypto crate and every enabled feature per
    `docs/dependencies/`.
  - **Scope:** in — the decision and threat model. Out — external key
    custody, purge authority, metadata-oblivious storage.
  - **User stories:** Human — an operator learns what an attacker with the
    disk sees (lengths, boundaries, identities) and does not see (bytes).
    API user — the port names one `KeyCapability` trait to implement. MCP
    user — none until T-28.2. Agent — an agent knows which errors mean
    "get a key" versus "the data is corrupt".
  - **Interface:** none.
  - **Contract schema:** the ADR fixes the frame and envelope byte tables.
  - **Test plan:** none; a decision record.
  - **Definition of done:** #86 closed.
  - **Complexity:** M (the threat model is the work).
  - **Documentation:** the ADR; `docs/formats/README.md` reserves the
    codec.
  - **Dependencies:** F-21 for verification-depth vocabulary; ADR-0001 and
    ADR-0002.
- [ ] T-28.2 Framed envelope encryption and key rotation (#83).
  - **Requirements:** encrypt and decrypt stream within explicit frame,
    count, and byte bounds; every frame binds AAD and refuses reordering,
    duplication, omission, truncation, and cross-representation
    substitution; recipient add and remove and KEK rotation do not
    re-encrypt payload frames or move `BlobId`; rotation receipt binds old
    and new representation and envelope coordinates without key material;
    catalog entries carry a representation coordinate so one blob may have
    plaintext and encrypted representations side by side.
  - **Acceptance criteria:** golden, mutation, corruption, property, fuzz,
    process-death, recovery, and public-contract tests; benchmarks for
    throughput, peak memory, frame overhead, rotation work; the Worldline
    `encryption` capability row executable.
  - **Scope:** in — representation codec, catalog representation
    coordinate (a v3 catalog entry or a v2 sidecar, decided in T-28.1),
    key capability port, rotation protocol with crash points. Out —
    external key stores; deterministic encryption; treating rotation as
    deletion.
  - **User stories:** Human — an operator rotates a key and no payload is
    rewritten. API user — `DurableStore::stage_encrypted(source,
    recipients)` returns the same `BlobId` as plaintext staging. MCP user —
    "ingest" gains a `recipients` parameter; "read" fails with a typed
    "key unavailable" the client can act on. Agent — an agent holding a
    recipient key reads; one without is refused before any plaintext.
  - **Interface:** `KeyCapability` trait; `stage_encrypted`,
    `rotate_recipients`; adapters `keep put --encrypt-to <recipient>`,
    `keep keys rotate`.
  - **Contract schema:** frame header (magic, version, frame index, AAD
    digest, ciphertext length, tag), envelope record (recipient id digest,
    wrapped DEK), representation record binding `LayoutId` and frame
    count; exact tables in a new `docs/formats/encrypted-representation-v1/`.
  - **Test plan:** golden — fixtures produced with a fixed test key (never
    a real key); edges — zero frames (refuse), maximum frame count, one
    recipient, maximum recipients, rotation with zero remaining recipients
    (refuse); known failures — every frame-tamper case; fuzz — frame and
    envelope decoders; crash matrix — rotation boundaries; soak — repeated
    rotations preserve readability.
  - **Definition of done:** #83 closed; README gap table row removed.
  - **Complexity:** XL.
  - **Documentation:** format page, `docs/dependencies/` entries, SECURITY.md
    scope update, CHANGELOG.
  - **Dependencies:** T-28.1, F-21, T-06.4, T-24.3, F-24.

### F-29 Retention-derived lifecycle surfaces

**Status:** Planned (#85 ADR, P1, M6). Needs F-18, F-21, F-22.

Decide how named vaults, mutable root sets, managed cache sets, and
expiry-safe replay sets lower onto retention namespaces and generations
without importing application meaning, ambient clocks, or unsafe early
release into the core; which primitives live in core, an optional policy
crate, or the application; how named assets map to opaque digests without
leaking labels; how time enters only as caller-supplied observations.

- [ ] T-29.1 ADR: lifecycle surfaces over retention.
  - **Requirements:** a core, policy, and application boundary per
    surface; no path, caller identity, wall clock, TTL, LRU score, or label
    in content identity; all mutations compare exact expected and observed
    generations and return immutable evidence; scoped acquisitions retain
    a complete generation until explicit release; replay-set release is
    expiry-only; private naming has an equality-leak and key-rotation
    threat model; crash-state and concurrency tables per cross-generation
    transition; the design names which surfaces wait for F-22 and which
    ship over retention alone.
  - **Acceptance criteria:** ADR Accepted; F-30 through F-33 each cite the
    section that governs them.
  - **Scope:** in — the decision. Out — any implementation.
  - **User stories:** Human — an application author learns which of these
    they get from Keep and which they build. API user — one policy crate
    boundary to depend on. MCP user — the tool vocabulary for vault, set,
    cache, and replay is fixed. Agent — an agent knows that "expire" is a
    caller observation, not a Keep clock.
  - **Interface:** none.
  - **Contract schema:** none.
  - **Test plan:** none.
  - **Definition of done:** #85 closed.
  - **Complexity:** M.
  - **Documentation:** the ADR.
  - **Dependencies:** F-18, F-21, F-22.

### F-30 Opaque asset, page, and bundle handles

**Status:** Planned (#89, P2, M6). Needs F-24 and F-29.

Validated opaque handles for immutable assets, bounded pages, and
deterministic structured bundles so applications never manage segment
coordinates, catalog locations, or physical identifiers. A handle
identifies one completely validated immutable graph and implies no
durability, retention, publication, or application meaning its receipt
does not establish.

- [ ] T-30.1 Handle grammar, codecs, traversal.
  - **Requirements:** asset handles bind exact `BlobId`, `LayoutId`, and
    admitted representation evidence; page handles enforce explicit
    count-and-byte bounds; bundle handles use a canonical descriptor
    grammar refusing duplicates, misorder, depth, fanout, and aggregate
    byte violations; streaming bounded traversal; named-member reads do
    not materialize unrelated members; handles contain no paths, offsets,
    generations, keys, or labels; retain and publish return
    generation-scoped evidence separate from the handle; re-encoding or
    relocation preserves the handle when the governed graph is unchanged.
  - **Acceptance criteria:** golden, mutation, corruption, graph-cycle,
    property, fuzz, and public-contract tests; a bundle at maximum fanout
    traverses within the documented bound.
  - **Scope:** in — a `handle` module and a bundle descriptor format. Out —
    mutable documents; application schemas; access-control tokens.
  - **User stories:** Human — none directly. API user — an application
    stores a directory as one bundle handle and reads one file from it
    without loading the rest. MCP user — "read member" takes a handle and
    a member name. Agent — an agent packages a build output as a bundle
    and hands one handle to the next stage.
  - **Interface:** `AssetHandle`, `PageHandle`, `BundleHandle`,
    `BundleDescriptor`, `read_member`.
  - **Contract schema:** bundle descriptor record: magic, version, member
    count, sorted members (name digest, kind, handle), descriptor digest,
    checksum; in `docs/formats/bundle-v1/`.
  - **Test plan:** golden — descriptor fixtures; edges — empty bundle,
    single member, duplicate name digest, cycle through a nested bundle;
    fuzz — descriptor decoder; stress — maximum fanout and depth.
  - **Definition of done:** #89 closed.
  - **Complexity:** L.
  - **Documentation:** format page; architecture page.
  - **Dependencies:** F-18, F-21, F-24, F-29.

### F-31 Private named vault and mutable root sets

**Status:** Planned (#92, P1, M6). Needs F-22, F-29, F-30.

Application-friendly private named vaults and mutable root sets over exact
retention namespaces and generations: opaque private-name digests map to
validated handles without storing plaintext names; root-set replacement is
an exact generation compare-and-swap that never merges stale candidates;
release publishes a successor and never claims deletion; read acquisitions
pin one complete generation; inspection reports verification, retention,
and generation posture separately.

- [ ] T-31.1 Vault and root-set surfaces.
  - **Requirements:** as above; private-name construction has an explicit
    equality-leak and key-rotation contract; recovery covers every staged
    generation, name-index, head, and cleanup crash prefix.
  - **Acceptance criteria:** concurrency, corruption, property, model,
    fuzz, and process-death tests; crash points allocated.
  - **Scope:** in — a policy-layer crate or module per T-29.1. Out — Git
    ref semantics; ambient identity or ACLs; metadata confidentiality
    beyond the admitted private-name profile.
  - **User stories:** Human — an operator names a release "v1.2" privately
    and later asks what it points at. API user — `vault.put(name, handle,
    expected_generation)` returns evidence. MCP user — "vault get" and
    "vault put" tools with generation parameters. Agent — an agent updates
    a root set optimistically and retries on the typed stale error.
  - **Interface:** `Vault::{get, put, remove, inspect}`,
    `RootSet::{replace, acquire, release}`.
  - **Contract schema:** name-index record binding name digest, handle,
    generation; format page.
  - **Test plan:** golden — index fixture; edges — put at stale generation,
    remove absent, acquire during replace; model — vault operations
    against a `BTreeMap`; crash matrix — every stage boundary.
  - **Definition of done:** #92 closed.
  - **Complexity:** L.
  - **Documentation:** format and architecture pages; a §5.4 warning on
    release.
  - **Dependencies:** F-18, F-21, F-22, F-29, F-30.

### F-32 Managed cache sets

**Status:** Planned (#90, P2, M6). Needs F-22, F-29, F-31.

An optional managed cache policy over retention generations: explicit TTL
observations, entry and logical-byte limits, deterministic approximate-LRU
eviction, bounded inspection, and scoped acquisitions that keep a selected
generation retained during use. Cache policy may release retention but
cannot alter identity, bypass reader safety, infer time from an ambient
clock, or make an acquired generation collectible before scope release.

- [ ] T-32.1 Cache-set policy surface.
  - **Requirements:** time enters as a validated caller observation under
    a named policy; checked arithmetic on entry and byte accounting;
    deterministic eviction under identical observations; eviction
    publishes an exact successor and returns evidence; scoped acquisition
    retains until release or crash recovery disposes its durable scope;
    inspection cannot shorten live windows.
  - **Acceptance criteria:** model tests over admission, access, eviction,
    acquisition, release, stale writers, restart, and GC interaction;
    benchmarks for policy work, metadata growth, acquisition cost,
    retained bytes without assigning deduplicated bytes to one owner.
  - **Scope:** in — policy-layer surface. Out — wall-clock ownership in
    core; exact global LRU; cache membership as application authority.
  - **User stories:** Human — an operator caps a cache at 10 GiB and it
    stays there. API user — `cache.acquire(handle, observation)` pins a
    generation for the scope. MCP user — "cache touch" and "cache evict"
    with an observation parameter. Agent — an agent supplies its own clock
    reading and gets deterministic eviction.
  - **Interface:** `CacheSet::{admit, touch, acquire, release, evict, inspect}`.
  - **Contract schema:** cache-set policy record and per-entry observation
    record.
  - **Test plan:** model — the listed operations; property — identical
    observation sequences produce identical eviction; crash matrix —
    scope boundaries; benchmark — as listed.
  - **Definition of done:** #90 closed.
  - **Complexity:** L.
  - **Documentation:** format and policy pages.
  - **Dependencies:** F-22, F-29, F-31.

### F-33 Expiry-safe replay sets

**Status:** Planned (#87, P2, M6). Needs F-22, F-29.

An expiry-safe replay-marker surface: atomic add-if-absent, digest-only
durable metadata, exact successor evidence, and expiry-only release. A
live marker cannot disappear before admitted expiry through remove,
repair, capacity, LRU, stale writer, recovery, or GC paths; duplicate
admission returns the exact existing evidence or a typed conflict.

- [ ] T-33.1 Replay-set surface.
  - **Requirements:** marker identity is a typed domain-separated digest;
    plaintext tokens never in durable metadata or diagnostics; add-if-absent
    is atomic against one exact generation and refuses stale writers;
    expiry is validated policy evidence, not a Keep clock; no public
    remove, repair, capacity, or LRU path releases a live marker; release
    admits only an observation at or beyond expiry and publishes a
    successor; restart preserves every unexpired marker through all crash
    prefixes.
  - **Acceptance criteria:** receipts distinguish newly admitted, already
    present, expired-and-released, stale, unavailable, corrupt; model,
    concurrency, corruption, property, fuzz, process-death tests cover
    premature-release attempts.
  - **Scope:** in — policy-layer surface. Out — authentication or token
    validation; sharing cache eviction; wall-clock truth.
  - **User stories:** Human — none directly. API user — an application
    rejects a replayed token by one `add_if_absent` call. MCP user —
    "replay check" tool. Agent — an agent deduplicates its own actions
    across restarts.
  - **Interface:** `ReplaySet::{add_if_absent, release_expired, inspect}`.
  - **Contract schema:** marker record (digest, expiry observation,
    generation).
  - **Test plan:** model — add, re-add, release before and after expiry;
    concurrency — two writers race on one marker; crash matrix — stage
    boundaries.
  - **Definition of done:** #87 closed.
  - **Complexity:** M.
  - **Documentation:** policy page.
  - **Dependencies:** F-18, F-21, F-22, F-29.

### F-34 Portable bindings

**Status:** Planned (#91 ADR, P3, M6). Needs F-21, F-26, and stable
handles from F-30.

One stable foreign binding contract for Node.js, Bun, Deno, and future
consumers that does not fork identity, formats, verification, durability,
or recovery per runtime. Identity and durable bytes come only from the Rust
core; typed failure distinctions survive every binding; large I/O is
bounded and streaming; cancellation cannot upgrade partial output into a
success receipt; secret-bearing buffers have explicit lifetime and logging
policy; one conformance suite runs against every runtime adapter.

- [ ] T-34.1 ADR: binding architecture.
  - **Requirements:** choose C ABI, N-API, WebAssembly, or a generated
    boundary; decide which operations are synchronous, worker-blocking, or
    streaming callbacks; specify cancellation, backpressure, process death,
    and version negotiation; reject reimplementation in JS or TS and
    runtime-specific identity.
  - **Acceptance criteria:** ADR Accepted; a live executable witness per
    claimed runtime before any runtime is named as supported.
  - **Scope:** in — the decision. Out — shipping bindings before native
    contracts stabilize (F-43).
  - **User stories:** Human — none. API user (JS) — the same `BlobId` text
    as Rust for the same bytes. MCP user — an MCP server written in
    TypeScript can wrap Keep without a subprocess. Agent — an agent in a
    JS runtime gets the same typed refusals.
  - **Interface:** none until implemented.
  - **Contract schema:** none.
  - **Test plan:** the Worldline through each binding.
  - **Definition of done:** #91 closed.
  - **Complexity:** M for the ADR; XL for bindings.
  - **Documentation:** the ADR.
  - **Dependencies:** F-21, F-26, F-30, F-43.

### F-35 Read-only FUSE projection

**Status:** Planned (#66 ADR, P3). No mount is exposed until F-19 and
F-22 land.

Expose an admitted durable snapshot as a read-only filesystem while
preserving the core law: a namespace of identity-addressed files or named
retained roots; random reads that never silently upgrade a range receipt
into whole-blob verification; stable inode, size, and ordering semantics
without clocks or host-order iteration; every open handle bound to one
catalog and retention snapshot; typed adapter errors mapped to FUSE codes;
no writable, rename, truncate, link, or repair operation.

- [ ] T-35.1 ADR and fake-port proof.
  - **Requirements:** the ADR decides viability and names authority and
    verification boundaries; deterministic tests against a fake FUSE port
    prove namespace ordering, inode stability, snapshot pinning, short and
    random reads, verification refusal, and unmount cleanup; a Linux-only
    proof mount skippable in CI; benchmarks for lookup latency, read
    throughput, verification work, allocations, page-cache behaviour;
    dependency, platform, privilege, and crash-surface audit.
  - **Acceptance criteria:** ADR Accepted with the read-only invariant
    stated; fake-port suite green.
  - **Scope:** in — an out-of-core adapter crate. Out — writes (F-36);
    treating paths or inodes as identity; network filesystems.
  - **User stories:** Human — an operator mounts a snapshot and greps it.
    API user — none. MCP user — none (the mount is the interface). Agent —
    an agent points ordinary tools at a mount and gets refusals as I/O
    errors, never plausible bytes.
  - **Interface:** `keep mount --read-only <root> <mountpoint>` in the
    adapter crate.
  - **Contract schema:** none.
  - **Test plan:** fake-port laws; Linux proof mount; benchmarks.
  - **Definition of done:** #66 closed.
  - **Complexity:** L.
  - **Documentation:** the ADR; adapter README.
  - **Dependencies:** F-19, F-22, F-23; informed by F-21.

### F-36 Transactional write-enabled projection

**Status:** Planned (#73, P2). Extends F-35.

A write-capable projection for copy and migration workflows that preserves
authenticated source identity, records mutation intents as first-class
causal events, and replays safely on reopen: writes are append-only at the
projection boundary unless an explicit transaction boundary is proven; no
silent patch application; no unbounded rehydration.

- [ ] T-36.1 Design and end-to-end proof.
  - **Requirements:** a tracked design on recovery and crash boundaries;
    one end-to-end test (mount-style view, write path, remap to a verified
    layout, unchanged identity of untouched bytes, deterministic replay on
    reopen); negative tests for untrusted mutation and partial writes;
    failure modes for partial visibility without a durable intent log,
    replay reordering, and power-loss gaps between mount transaction and
    durable publication.
  - **Acceptance criteria:** as listed in #73.
  - **Scope:** in — adapter crate. Out — a general mutable POSIX layer.
  - **User stories:** Human — an operator copies a tree into a mount and
    gets one committed bundle. API user — none. MCP user — none. Agent —
    an agent writes through the mount and later reads the exact bytes by
    identity.
  - **Interface:** `keep mount --transactional`.
  - **Contract schema:** mutation-intent journal record.
  - **Test plan:** end-to-end; crash matrix over journal boundaries.
  - **Definition of done:** #73 closed.
  - **Complexity:** XL.
  - **Documentation:** design page; adapter README.
  - **Dependencies:** F-35, F-24.

### F-37 Repository verification tooling and CI gates

**Status:** Done (issues #32, #33, #34, #35, #40, #44, #47, #55, #57,
and #59; ADR-0006, ADR-0007, ADR-0008).

`cargo xtask` is the single repository-owned automation boundary, in
Rust, with no tracked Python: `verify`, `documentation-integrity-check`
(markdownlint-cli2 0.23.2, lychee 0.21.0 offline with fragments, actionlint
1.7.12), `documentation-refusal-check`, `source-structure-check` (500-line
hard limit, forbidden filenames), `golden-file-worldline-check`,
`conformance-check`, `durability-crash-matrix`, `benchmark-baseline`,
`prepare-fuzz-corpus`, `fuzz run --profile smoke|scheduled`,
`fuzz describe`, `fuzz check-corpus`. CI pins every action, persists no
credentials on read-only jobs, runs a scheduled fuzz campaign, and
Dependabot covers every manifest.

- [x] T-37.1 xtask crate and source-structure law — #44.
- [x] T-37.2 Documentation integrity in xtask — #32, #35, #47.
- [x] T-37.3 Fuzz orchestration in xtask; scheduled campaigns — #33, #55.
- [x] T-37.4 Conformance oracles in xtask; bounded `b3sum` — #57, #59.
- [x] T-37.5 CI hygiene — #34, #40.
- [x] T-37.6 Child-process boundary — ADR-0006, ADR-0007, ADR-0008.

Gates the Rust standard §24 expects that are not yet automated, each a
candidate S task without an issue: Miri on a nightly subset; a
mutation-testing subset; a public API diff check; a format fixture diff
check; a coverage threshold; a forbidden-terms check.

### F-38 Documentation status drift

**Status:** Proposed. Small, and worth doing before the next release
note.

Several living pages lag `main`. None changes behaviour; each misleads a
reader about ownership.

- [ ] T-38.1 Fix the README gap table.
  - **Requirements:** the rows "Restart recovery for retention publication
    and migration" and "Reader fence" point at #19, which closed on
    2026-09-08 under a different title; retention recovery and the fence
    are in PR #99, and migration recovery has no open issue. Open one issue
    per remaining gap (migration recovery, durable reads F-23) and point
    the table at them; rewrite the "waits for a human until #19 lands"
    sentence.
  - **Acceptance criteria:** every gap-table row names an open issue or an
    open PR.
  - **Scope:** README, one or two new issues. Out — everything else.
  - **User stories:** Human — a reader following a gap lands on live work.
    API user, MCP user, Agent — same.
  - **Interface:** none. **Contract schema:** none.
  - **Test plan:** `cargo xtask documentation-integrity-check`.
  - **Definition of done:** merged. **Complexity:** S.
  - **Documentation:** README. **Dependencies:** none.
- [ ] T-38.2 Correct the crate doc in `src/lib.rs`.
  - **Requirements:** the sentence "Partial-prefix migration recovery,
    filesystem retention execution, immutable reader snapshots, and
    garbage collection remain intentionally absent" predates
    `FilesystemRetentionPublicationAuthority`; rewrite to the true set (and
    again when PR #99 merges).
  - **Acceptance criteria:** `cargo doc` output matches the ledger.
  - **Scope, stories, interface, schema:** as T-38.1.
  - **Test plan:** `cargo test --doc`. **Definition of done:** merged.
  - **Complexity:** S. **Documentation:** rustdoc. **Dependencies:** none.
- [ ] T-38.3 Reconcile v2 pages with each other.
  - **Requirements:** `migration-inventory.md` says verification-first
    storage "remains in progress" while `KEEP-MIGRATION-003` says
    Implemented; `retention-publication.md` describes v2 catalog
    publication proving retained closures but the ledger has no
    requirement row or evidence for it. Add the row and its test, or mark
    the paragraph as a gap.
  - **Acceptance criteria:** no two v2 pages disagree on a status.
  - **Scope, stories, interface, schema:** as T-38.1.
  - **Test plan:** documentation gates plus the phrase-pin contract tests.
  - **Definition of done:** merged. **Complexity:** S.
  - **Documentation:** the pages named. **Dependencies:** none. F-14 covers
    the v1 pages.

### F-39 Compression representation codec

**Status:** Proposed. Every decision record defers compression to a
representation codec without naming an owner.

A representation codec that stores a chunk or layout compressed, changes
only `RepresentationId`, and is verified by decompressing to the exact
chunk bytes and re-hashing before any byte is emitted.

- [ ] T-39.1 ADR and codec.
  - **Requirements:** frame-per-chunk compression with the uncompressed
    `ChunkId` and length in the frame header; decompression bounded by the
    declared length and refused on overrun; the catalog carries the
    representation coordinate (shared with T-28.2); already-compressed
    input is detected by measurement, not by extension; the benchmark
    scenario "already compressed data" (T-09.3) gates the default policy.
  - **Acceptance criteria:** golden and mutation corpora; a compression
    bomb (declared length small, actual large) refuses before allocation;
    the benchmark shows the ratio and CPU cost per profile.
  - **Scope:** in — one audited pure-Rust codec behind a feature flag with
    a dependency review. Out — dictionary training; per-file heuristics
    from paths.
  - **User stories:** Human — an operator turns on compression and sees
    physical bytes drop in the ingestion receipt. API user — reads are
    unchanged; `BlobId` is unchanged. MCP user — "ingest" gains a
    `representation` parameter. Agent — an agent chooses compression per
    blob from measured entropy.
  - **Interface:** `stage` with a `RepresentationPolicy`.
  - **Contract schema:** compressed frame header; format page.
  - **Test plan:** golden — fixtures; edges — incompressible input, empty
    chunk (impossible: chunks are nonempty), maximum chunk; known failures
    — declared-length lies; fuzz — frame decoder; benchmark — ratio and
    CPU.
  - **Definition of done:** ADR Accepted; codec shipped behind a flag.
  - **Complexity:** L.
  - **Documentation:** ADR; format page; dependency review.
  - **Dependencies:** T-03.3 (shares the representation coordinate with
    F-28), F-24, F-21.

### F-40 Additional chunking profiles and hierarchical layouts

**Status:** Proposed. ADR-0003 and the layout rationale both name the
evidence required and defer the decision.

Register a second CDC profile (the benchmark measured 16 KiB and 256 KiB
targets) only when workload evidence justifies a durable coordinate, and
add a hierarchical layout codec only when a workload exceeds 256 GiB per
plan or needs bounded plan streaming.

- [ ] T-40.1 Second registered profile.
  - **Requirements:** a new 96-byte profile record and `StorageProfileId`;
    the conformance corpus gains vectors for it; selection is explicit
    per stage call, never inferred from paths; the closure verifier's
    profile replay admits it.
  - **Acceptance criteria:** benchmark evidence on a designated runner
    shows a measurable win for a named workload; ADR-0003 amended or a
    new ADR.
  - **Scope:** in — profile registry, corpus, benchmark. Out — keyed or
    privacy-oriented chunking (needs its own threat model).
  - **User stories:** Human — an operator chooses a small-chunk profile for
    a source-code store. API user — `stage(source, profile, limits)`. MCP
    user — "ingest" gains a `profile` parameter. Agent — an agent selects a
    profile by measured reuse.
  - **Interface:** as above.
  - **Contract schema:** profile record.
  - **Test plan:** golden — new corpus; property — partition invariance;
    benchmark — the win.
  - **Definition of done:** profile registered.
  - **Complexity:** M.
  - **Documentation:** ADR; corpus README.
  - **Dependencies:** T-09.2.
- [ ] T-40.2 Hierarchical layout codec.
  - **Requirements:** a new codec (codec 1 has no extension point) with
    its own depth, fanout, cycle, aggregate, and allocation laws; range
    planning across levels; closure accounting extended.
  - **Acceptance criteria:** golden and mutation corpora; the 256 GiB
    ceiling lifted to a stated new bound.
  - **Scope:** in — codec 2. Out — changing codec 1.
  - **User stories:** Human — a store holds a 1 TiB image. API user —
    reads are unchanged. MCP user — none. Agent — none.
  - **Interface:** none new.
  - **Contract schema:** layout codec 2 record.
  - **Test plan:** golden; edges — depth exactly at bound; fuzz — decoder.
  - **Definition of done:** codec registered in `docs/formats/README.md`.
  - **Complexity:** L.
  - **Documentation:** new format page.
  - **Dependencies:** a workload that needs it.

### F-41 Platform adapters beyond Linux ext4

**Status:** Proposed. `recovery.md` defers Windows "until an adapter and
crash harness prove equivalent semantics"; macOS is not admitted for
production; the crash matrix does not simulate power loss.

- [ ] T-41.1 macOS APFS production admission.
  - **Requirements:** a platform profile that probes APFS for the same
    capabilities (atomic no-clobber link, atomic same-filesystem
    replacement, durable `fsync` semantics including `F_FULLFSYNC`,
    advisory locks, device and inode identity); refuses anything else;
    the crash matrix runs on macOS in CI.
  - **Acceptance criteria:** `KEEP-RECOVERY-003` gains a macOS row; the
    105-case matrix (and the v2 matrices) green on a macOS runner.
  - **Scope:** in — `filesystem_platform_profile.rs` sibling. Out —
    case-insensitive volumes (refuse), network volumes.
  - **User stories:** Human — a developer on a Mac runs a durable store
    locally. API user — same API. MCP user — same. Agent — same.
  - **Interface:** none new.
  - **Contract schema:** none.
  - **Test plan:** the full crash matrix on macOS; platform refusal laws.
  - **Definition of done:** README platform admission sentence updated.
  - **Complexity:** L.
  - **Documentation:** `recovery.md` platform contract.
  - **Dependencies:** none; benefits from T-41.2.
- [ ] T-41.2 Host power-loss simulation.
  - **Requirements:** a Linux harness at the block layer, not the process
    layer: `dm-log-writes` to record every write and replay each prefix,
    `dm-flakey` to drop and corrupt writes in a window, or a CrashMonkey
    style checker that enumerates legal reorderings of unsynced writes
    across barriers; at each `KEEP-CRASH` point, every legal on-disk
    state the recorded writes admit is restarted and asserted with the
    same restart assertions as the process-death matrix. Process death
    proves only that the writer's own ordering is right; this proves the
    store survives write reordering, torn records, and a lost volatile
    drive cache before a barrier.
  - **Acceptance criteria:** every `KEEP-CRASH` point has a power-loss
    twin; torn-record cases (a record cut inside a sector) refuse as
    truncation, never admit; README "Proven restart recovery" states what
    is now covered and what is not.
  - **Scope:** in — xtask harness, privileged CI job. Out — torn writes
    below the sector size the device guarantees atomic (state as a
    nonclaim); hardware that lies about flush completion.
  - **User stories:** Human — an operator trusts the store after a real
    power cut, not only after `kill -9`. Others — same.
  - **Interface:** `cargo xtask durability-crash-matrix --power-loss`.
  - **Contract schema:** none.
  - **Test plan:** the matrix under simulated loss.
  - **Definition of done:** v1 requirements "host-power-loss simulation
    remains outside" sentence removed.
  - **Complexity:** L.
  - **Documentation:** `recovery.md`, README.
  - **Dependencies:** a privileged Linux runner (T-41.3).
- [ ] T-41.3 Designated Linux runner class.
  - **Requirements:** one named runner class for benchmarks, the crash
    matrix, and power-loss simulation, with recorded kernel, filesystem,
    and hardware identity.
  - **Acceptance criteria:** baselines and matrices name the class.
  - **Scope:** CI configuration.
  - **User stories:** Human — reproducible numbers. Others — none.
  - **Interface:** none. **Contract schema:** none.
  - **Test plan:** none. **Definition of done:** in use by T-09.2.
  - **Complexity:** S. **Documentation:** benchmark README.
  - **Dependencies:** none.
- [ ] T-41.4 Windows adapter — deferred; opens only after T-41.1 proves
  the second-platform pattern. Complexity XL. No task fields until then.
- [ ] T-41.5 Barrier and journal semantics per filesystem.
  - **Requirements:** a written account, per candidate filesystem (XFS,
    btrfs, ZFS, APFS, and ext4 in each journaling mode), of what `fsync`
    on a file and on its directory guarantees, whether a rename or link is
    durable without a parent sync, whether the journal can reorder data
    against metadata, and whether copy-on-write can leave a stale block
    visible after a crash; each claim tied to a T-41.2 run on that
    filesystem, not to documentation alone.
  - **Acceptance criteria:** the platform profile probe admits a
    filesystem only when its row is proven; the v1 `recovery.md` platform
    contract cites the table.
  - **Scope:** in — the table and the probe. Out — writing to a raw block
    device to sidestep filesystems (a separate decision; it forfeits the
    hard-link and directory-sync protocol the formats depend on).
  - **User stories:** Human — an operator learns why their filesystem is
    refused, by row. Others — same.
  - **Interface:** none. **Contract schema:** none.
  - **Test plan:** T-41.2 on each filesystem.
  - **Definition of done:** table merged and cited by the probe.
  - **Complexity:** L. **Documentation:** `recovery.md` platform contract.
  - **Dependencies:** T-41.2, T-41.3.

### F-42 Operator surfaces

**Status:** Proposed. Keep's core exposes no CLI or MCP surface by design;
both belong in separate adapter crates that depend on `keep` and import
nothing back.

- [ ] T-42.1 `keep-cli` adapter crate.
  - **Requirements:** every command maps one-to-one onto a public API call
    and prints the receipt or the typed refusal; no command invents
    policy; dangerous commands (`gc execute`, `compact`, `dispose`) demand
    an explicit plan identifier and print the §5.4 warning; exit codes
    distinguish success, evidenced refusal, and operational failure;
    output has a stable machine form (a canonical JSON profile per
    ADR-0004) and a human form.
  - **Acceptance criteria:** a golden transcript per command over the
    golden store; the crate is not a workspace default member and is not
    published with `keep`.
  - **Scope:** in — `keep-cli/`. Out — anything the core cannot do.
  - **User stories:** Human — `keep verify --depth blob /example/store`
    prints a report. API user — none. MCP user — the MCP server may shell
    out to nothing; it links the crate. Agent — an agent uses the machine
    form.
  - **Interface:** `keep init`, `keep put`, `keep cat`, `keep verify`,
    `keep retain`, `keep release`, `keep migrate`, `keep recover`,
    `keep gc plan|execute`, `keep compact`, `keep dispose`, `keep mount`.
  - **Contract schema:** the canonical JSON output profile.
  - **Test plan:** golden transcripts; edges — every refusal variant has
    an exit code; fuzz — argument parser.
  - **Definition of done:** crate merged with transcripts.
  - **Complexity:** M once F-23 and F-24 exist.
  - **Documentation:** crate README; a how-to page per Documentation
    Standards.
  - **Dependencies:** F-23, F-24; F-21 for `verify`; F-22 for `gc`.
- [ ] T-42.2 `keep-mcp` adapter crate.
  - **Requirements:** an MCP server exposing tools that mirror the CLI
    commands with JSON schemas derived from the same canonical profile;
    every tool result carries the receipt or the typed refusal; tools that
    mutate require the same plan identifiers as the CLI; no tool exposes
    physical paths, offsets, or key material; secret-bearing inputs
    (recipient keys, F-28) have an explicit lifetime and are never logged.
  - **Acceptance criteria:** a conformance run of the Worldline through the
    MCP tools; schema snapshots as golden fixtures.
  - **Scope:** in — `keep-mcp/`. Out — remote transport security beyond
    what the MCP host provides.
  - **User stories:** Human — a person in an MCP-capable client asks for a
    blob by identity and gets exact bytes or a refusal. API user — none.
    MCP user — the tool list is the whole surface. Agent — an agent
    ingests, retains, verifies, and reads with typed outcomes it can branch
    on.
  - **Interface:** tools `keep.ingest`, `keep.read`, `keep.verify`,
    `keep.retain`, `keep.release`, `keep.snapshot`, `keep.gc.plan`,
    `keep.gc.execute`, `keep.recover`, `keep.dispose`.
  - **Contract schema:** tool input and output JSON schemas; golden
    snapshots.
  - **Test plan:** Worldline through the tools; edges — every refusal
    variant round-trips; fuzz — tool input decoder.
  - **Definition of done:** crate merged; a how-to page.
  - **Complexity:** M after T-42.1.
  - **Documentation:** crate README; how-to.
  - **Dependencies:** T-42.1 (shared canonical profile), F-34 if the
    server is not Rust.

### F-43 Public release and API stability

**Status:** Proposed. Keep is `0.0.0`, `publish = false`, and states it
will follow SemVer "after its public API and format compatibility policies
are established".

- [ ] T-43.1 Format compatibility policy.
  - **Requirements:** a written policy stating which formats are frozen
    (`keep.flat-chunks/v1`, `keep.segment-store/v1`, `/v2`, the profile
    record), how a successor is introduced (new coordinate, one-way
    migration, no downgrade), and what a reader of version N promises for
    version N minus one.
  - **Acceptance criteria:** a cross-version compatibility test that opens
    every committed golden store with the current reader.
  - **Scope:** in — policy page and test. Out — new formats.
  - **User stories:** Human — an operator knows whether upgrading Keep can
    strand a store. API user — same for the crate. MCP user, Agent — same.
  - **Interface:** none. **Contract schema:** none.
  - **Test plan:** the compatibility test. **Definition of done:** merged.
  - **Complexity:** S. **Documentation:** `docs/formats/README.md` policy
    section. **Dependencies:** F-17 complete.
- [ ] T-43.2 Public API surface review and `0.1.0`.
  - **Requirements:** the flat re-export list in `src/lib.rs` (over two
    hundred names) is reviewed for what a caller needs versus what the
    crash matrix needs; internal-only names move behind
    `repository-tasks` or `pub(crate)`; a public API diff gate (F-37
    candidate) runs in CI; rustdoc builds with no warnings and every
    important workflow has a doctest; `publish = true`; CHANGELOG `0.1.0`
    section.
  - **Acceptance criteria:** `cargo semver-checks` or an equivalent passes;
    crates.io dry run passes; README "Try it" runs from the published
    crate.
  - **Scope:** in — API surface, docs, manifest. Out — new features.
  - **User stories:** Human — `cargo add keep` works. API user — a stable
    surface with a documented deprecation policy. MCP user — none. Agent —
    an agent reads the rustdoc and finds the durable entry point on the
    front page.
  - **Interface:** none. **Contract schema:** none.
  - **Test plan:** API diff gate; doc tests. **Definition of done:** tag.
  - **Complexity:** M. **Documentation:** README, CHANGELOG, rustdoc.
  - **Dependencies:** F-17, F-18, F-19, F-21, F-22, F-23, F-24 (a
    `0.1.0` without a durable write path is not worth promising).

### F-44 Multi-writer, replication, and remote tiers

**Status:** Out of scope by ADR-0005 ("Multi-writer, distributed locking,
and network filesystems require another decision") and the v1 rationale.
Listed so the boundary is visible.

What would have to exist first, in order: a durable read path (F-23), a
durable write path (F-24), GC with reader fencing (F-22), a
representation coordinate so a remote tier is a location, not an identity
(T-03.3), and a decision record on cross-host authority that ADR-0005
explicitly did not make. Until then: one writer, many readers, one local
host, Linux ext4.

Also out of scope, by explicit nonclaim and not revisited here: secure
deletion or erasure (ADR-0009, README); application semantics for Echo,
Git, Graft, or WARP inside the core (`KEEP-STORE-016`); Git objects or refs
as a storage or retention protocol (ADR-0005, ADR-0009); clock-based grace
(ADR-0009); Serde output as a durable format (ADR-0004).

### F-45 Formal verification of the durable protocols

**Status:** Proposed. Every proof Keep has today is empirical: property
tests, model-based tests against a boring reference, golden corpora,
fuzzing, and process-death injection. None of it is a proof over all
interleavings.

The publication, migration, retention, reader-fence, and GC protocols are
each a small state machine with explicit phases, named crash points, and a
recovery classifier. That is exactly the shape a model checker handles.
The Rust codecs and planners are pure functions over bounded byte arrays,
which is the shape a bounded model checker handles.

- [ ] T-45.1 TLA+ model of publication, migration, and retention.
  - **Requirements:** one PlusCal or TLA+ specification per protocol
    (catalog publication `KEEP-CRASH-001` to `-035`, migration `-053` to
    `-073`, retention publication `-036` to `-052`, then GC once F-22
    lands) with a crash action enabled at every phase boundary and a
    recovery action that models the classifier; safety properties: one
    verified head always selects one complete catalog; no live segment is
    ever unlinked; a reader under the fence never observes two
    generations; every crash prefix reaches exactly one lawful state;
    liveness: recovery always terminates and a retry always completes or
    refuses. The model is checked with TLC over a bounded number of
    phases, writers (one), readers (at least two), and crash points.
  - **Acceptance criteria:** TLC reports no counterexample for each
    property; the phase order in the spec is generated from or checked
    against `CatalogPublicationPhase::ALL`, `StoreMigrationPhase::ALL`, and
    `RetentionPublicationPhase::ALL` so the model cannot drift from the
    code; a deliberately reordered phase produces a counterexample.
  - **Scope:** in — `formal/` directory, an xtask command that runs TLC in
    CI with pinned versions. Out — proving the Rust implementation refines
    the model (that is T-45.2 and beyond).
  - **User stories:** Human — a reviewer reads a counterexample trace
    instead of reasoning about interleavings by hand. API user — none.
    MCP user — none. Agent — an agent changing a phase order must update
    the spec and gets a machine-checked answer.
  - **Interface:** `cargo xtask formal-check`.
  - **Contract schema:** none.
  - **Test plan:** TLC over the bounded model; a mutation set of known-bad
    orderings that must each fail.
  - **Definition of done:** three specs checked in CI; a rationale note
    stating what the model does and does not cover.
  - **Complexity:** L.
  - **Documentation:** `formal/README.md`; each protocol page links its
    spec.
  - **Dependencies:** none; GC spec waits for F-22.
- [ ] T-45.2 Bounded model checking of codecs and planners.
  - **Requirements:** Kani proofs (or an equivalent bounded checker) over
    every decoder that the codec refuses, never panics, and never reads
    past its input for all byte arrays up to the record's bounded length;
    over the range planner that the selected interval is minimal and
    covers the range for all admissible layouts up to a bounded entry
    count; over every checked-arithmetic path that overflow is a typed
    refusal, not a wrap.
  - **Acceptance criteria:** proofs run in CI on a pinned toolchain;
    every existing fuzz target has a proof harness for the same entry
    point; a deliberately introduced unchecked add fails a proof.
  - **Scope:** in — proof harnesses next to the fuzz targets. Out —
    proving filesystem behaviour (T-45.1 and T-41.2 own that).
  - **User stories:** Human — a reviewer trusts that no input bitstream
    panics a decoder. API user — the "never panics" claim in rustdoc is
    machine-backed. MCP user — none. Agent — an agent adding a field adds
    a harness and gets a proof or a counterexample.
  - **Interface:** `cargo xtask formal-check --bounded`.
  - **Contract schema:** none.
  - **Test plan:** the proofs; the mutation set.
  - **Definition of done:** every decoder and planner has a harness; the
    Rust standard's "no panic" gate cites it.
  - **Complexity:** L.
  - **Documentation:** `formal/README.md`; a `docs/dependencies/` review
    for the checker.
  - **Dependencies:** none.
- [ ] T-45.3 Refinement from model to code — deferred. Proving the Rust
  executor refines the TLA+ model (Creusot, Verus, or a trace-checking
  harness that replays TLC traces through the storage-port fakes) is the
  step after T-45.1 and T-45.2. The trace-replay form is tractable early:
  every TLC trace becomes a deterministic test over the existing fault-
  injecting fakes. Complexity XL for deductive refinement; M for trace
  replay. No task fields until T-45.1 lands.

### F-46 Condition coverage and mutation analysis

**Status:** Proposed. The Rust standard §24 lists a coverage threshold and
a mutation subset as expected gates; neither runs today.

- [ ] T-46.1 Mutation analysis in CI.
  - **Requirements:** `cargo-mutants` over `src/` on a schedule and over
    changed files on every pull request; every surviving mutant is either
    killed by a new law or listed in a reviewed exclusion file with a
    reason; timeouts and unviable mutants are distinguished from
    survivors.
  - **Acceptance criteria:** zero unexplained survivors on `main`; the
    exclusion file is short and each entry cites the invariant that makes
    the mutant unobservable.
  - **Scope:** in — xtask command, CI job, exclusion file. Out — mutating
    tests, fuzz targets, or xtask itself.
  - **User stories:** Human — a reviewer sees that deleting a checksum
    comparison fails a named test. API user — none. MCP user — none. Agent
    — an agent's new code is rejected if a mutant survives.
  - **Interface:** `cargo xtask mutation-check [--changed]`.
  - **Contract schema:** `mutants-exclusions.toml`.
  - **Test plan:** the run itself; a seeded known-survivable mutant to
    prove the exclusion mechanism.
  - **Definition of done:** scheduled job green; PR job advisory for one
    release then required.
  - **Complexity:** M.
  - **Documentation:** CONTRIBUTING; `docs/dependencies/` review.
  - **Dependencies:** none.
- [ ] T-46.2 Branch and condition coverage gate.
  - **Requirements:** instrumented coverage with branch granularity on the
    core and adapter modules; a documented threshold per module family
    (codecs and planners at 100 percent branch, orchestration at a stated
    lower bound); a report of every uncovered condition; modified
    condition/decision coverage for the recovery classifiers and the
    publication readiness checks, where each boolean sub-condition is
    shown to independently flip the outcome.
  - **Acceptance criteria:** the gate runs in CI on a pinned toolchain;
    every decoder, planner, and classifier reaches its threshold; the
    MC/DC set for the classifiers is a checked-in table naming the test
    that flips each condition.
  - **Scope:** in — coverage tooling, thresholds, the MC/DC table. Out —
    coverage of xtask, benchmarks, and tests themselves.
  - **User stories:** Human — a reviewer sees which condition a PR left
    unexercised. API user — none. MCP user — none. Agent — an agent adding
    a condition adds the flipping test.
  - **Interface:** `cargo xtask coverage-check`.
  - **Contract schema:** `coverage-thresholds.toml`; `mcdc.tsv`.
  - **Test plan:** the run; a seeded uncovered branch to prove the gate
    fails.
  - **Definition of done:** thresholds enforced; the MC/DC table covers
    every recovery classifier.
  - **Complexity:** M for branch coverage; L for MC/DC.
  - **Documentation:** CONTRIBUTING; Rust standard §24 row updated from
    "expected" to "enforced".
  - **Dependencies:** none; T-46.1 first, since mutation results tell you
    which uncovered conditions matter.
