# Changelog

All notable changes to Keep will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project intends to follow [Semantic Versioning](https://semver.org/)
after its public API and format compatibility policies are established.

## [Unreleased]

- Recovery preserves interrupted roots with invalid complete realization-profile or closure-policy groups, returning the exact domain refusal before stage discard (#99).

- Recovery preserves interrupted roots and manifests whose complete generation/predecessor fields contradict initial or successor history (#99).

- Recovery preserves interrupted roots declaring empty or oversized namespaces, applying the domain length rule before payload arrival (#99).

- Recovery preserves interrupted root and manifest stages that declare counts above the format ceiling, even when their declared lengths are self-consistent (#99).

- Recovery rejects and preserves short root and manifest headers whose complete size fields contradict their declared record length (#99).

- Recovery verifies available checksum bytes in interrupted head stages and preserves contradictory prefixes as corrupt evidence instead of discarding them (#99).

- Recovery preserves short heads whose complete generation/predecessor fields contradict initial or successor history, sharing semantic admission with complete head decoding (#99).

- Recovery preserves interrupted head stages whose complete manifest-length field violates canonical bounds or alignment, returning the precise corruption cause instead of discarding them (#99).

- Clarify that retention recovery and fenced readers are implemented but still under correctness remediation and independent review; link the completed process-death reader evidence and name remaining recovery gaps (#99).

- Filesystem recovery tests now require the exact checksum-corrupt root-stage refusal and unchanged retained evidence, with calibrated diagnostic and deletion checks (#99).

- Recovery now reports `ManifestStageWithoutRootStage` when complete head and manifest stages lack their root stage, instead of incorrectly reporting a missing manifest; refusal preserves retained evidence (#99).

- The retention process-death matrix now independently reads the recovered head generation and exact selected-root bytes before forward retry, including absence before publication (#99).

- Successor recovery evidence now covers every ordered publication prefix and verifies the recovered generation and exact selected-root bytes through the fenced reader; the ledger distinguishes this from sampled mid-write truncations (#99).

- Publication now identifies recovery-observation failures through `RecoveryObservationRefused`, preserving the original typed or OS cause in its error chain; explicit recovery and later planning/execution boundaries retain their existing behavior (#99).

- Publication tests now require the exact missing-manifest recovery refusal for a retained complete head, rejecting unrelated recovery errors (#99).

- The fenced reader corruption law now requires the precise checksum-mismatch cause and expected/observed checksum bytes instead of accepting any root error (#99).

- Correct the retention recovery documentation's obsolete integration status and explicitly retain its open incomplete-stage pinning and post-removal failure findings (#99).

- Recovery fixtures now use production's head-to-manifest binding validation, preventing independently valid but contradictory records from becoming observed retention state (#99).

- Repository-task migration admission now reports root capability-clone failures as namespace failures and identity-probe failures as root-identity failures, preserving their I/O causes (#99).

- Reader collection tests now reject same-generation digest changes and require preserved I/O causes at all three read boundaries, using returned payloads instead of script load counters (#99).

- Retention model tests now require exact preparation and superseded-publication refusals, including their identifying coordinates; unrelated errors no longer satisfy a rejected transition, and the harness sequence-count assertion is removed (#99).

- Derive the recovery manifest read bound from the codec's checked framing calculation and semantic entry limit, removing a separately maintained size literal (#99).

- Keep the reader fence private to the crate; callers retain snapshots through `FilesystemRetentionSnapshot`, which owns the fence for its lifetime (#99).

- Retention readers load catalog bytes through the originally pinned store directory, so replacement of its ambient path cannot redirect catalog collection into another directory.

### Added

- Model-based retention evidence: every three-operation sequence over initial
  publications of two namespaces, a successor, a byte-identical retry, and a
  stale initial (125 sequences, each in a fresh migrated store) agrees with a
  deterministic namespace-to-(generation, anchor-set) map and liveness after
  every step, observed through the fenced reader view; a source contract
  keeps clocks, paths, environment, and identity out of the retention core.
- `FilesystemRetentionSnapshot` is the version-two reader view: it admits the
  root as version two, acquires a shared `ReaderFence` on `reader.lock`,
  double-collects the catalog and retention heads around loading through
  `collect_retention_view` (bounded by `ReaderAttemptLimit`, refusing an
  exhausted limit or an absent catalog), binds the catalog snapshot, the
  retention head, and its manifest, and verifies each selected root against
  the manifest on demand while the fence is held.
- Storage-independent retention recovery planning: `assess_root_stage`,
  `assess_manifest_stage`, and `assess_head_stage` classify each fixed stage
  as absent, complete, truncated, or corrupt through the decoders' own
  truncation laws; `plan_retention_recovery` turns that evidence, the observed
  current state, and pool-entry observations into an ordered
  `RetentionRecoveryPlan` (discard a pre-effect truncated stage, link and
  protect complete orphans, finalize a complete head over linked stages, clean
  up stages the published head already names) or a typed
  `RetentionRecoveryRefusal`. `RetentionRecoveryStorage` names one blocking
  capability per step and `execute_retention_recovery` runs a plan in order,
  stopping at the first refused step with the completed prefix named in
  `RetentionRecoveryError`. `FilesystemRetentionPublicationAuthority::recover`
  observes the stages within their format bounds, reopens complete stages
  bound to their identity, and executes the plan under the retained writer
  lock, so a crash after the head stage is synchronized finalizes on restart
  and a byte-identical retry is already committed. Laws drive every
  publication prefix from 0 through 18 phases, truncate each stage mid-write,
  and replay successor prefixes over a published generation; each recovers to
  its documented state, recovery is idempotent, and the forward retry reports
  the predicted outcome. Publication runs that recovery as its first step, so
  an interrupted publication no longer waits for a human unless it left a
  complete orphan; `RecoveryRefused` and `RecoveryStepRefused` carry
  recovery's own errors through `RetentionCurrentStateRefusal`. The crash
  matrix gains `KEEP-CRASH-036` through `052`: a child migrates a golden
  bundle store, publishes retention generation one, and is killed before,
  during, or after each of the seventeen phases; restart reopens the store,
  runs recovery, and requires the documented steps, outcome, and forward
  retry. `FilesystemVersionTwoAdmission::reopen_unchecked_for_repository_tasks`
  and `FilesystemStoreMigrationAuthority::open_unchecked_for_repository_tasks`
  give repository tools the same bypass version one already had.
- Recovery receipts retain the exact observed namespace prefix and bound
  migration intent digest, including observations of completed migration.

- Recovery jointly verifies retained stages and canonical records during
  adoption, refusing substituted canonical inodes before forward execution.

- Late-stage refusals report receipt effects as receipts when no marker
  artifact exists.

- Migration recovery revalidates current writer authority before observing
  residue, preventing stale expected intents from admitting a corrupt store.

- README recovery and reader-fence gaps point to their current open owners.

- Migration recovery planning, bounded filesystem residue admission, explicit
  truncated-stage discard, and receipts listing resumed phases. The production
  crash matrix now includes 68 migration process-death cases (Refs #108).

- `FilesystemRetentionPublicationAuthority` executes the 17 ordered retention
  publication phases against a completely migrated version-2 root. It stages
  `root.next`, `manifest.next`, and `head.next` exclusively, verifies device
  and inode identity at every transition, hard-links both immutable pool
  entries without replacement, atomically replaces `retention/HEAD`, and
  removes retained stages only after its canonical target verifies. A
  successor is admitted only when the prepared head names the observed
  manifest as its exact predecessor at the next liveness generation; a
  superseded candidate, an exact already-committed retry, and every retained
  stage refuse or return with zero retention mutation.
- `FilesystemVersionTwoAdmission::reopen` grants version-two writer authority
  as its own type, so no version-one publisher can consume it. It reopens
  `FORMAT`, `migration.intent`, and `migration.receipt` without following
  links, bounds each to its canonical length, admits the receipt only against
  the decoded intent and marker, and on Linux admits `retention`,
  `retention/roots`, `retention/manifests`, `gc`, `recovery`, and
  `recovery/dispositions` against the root's filesystem, mount, and inode
  flags exactly as the version-1 protocol directories are admitted.
  `FilesystemPlatformAdmissionError::MigrationRecord` names a record refusal.
- `observe_current` returns the published retention head and its
  cross-verified pool manifest.
- `RetentionCurrentStateRefusal` travels as the source of every `InvalidData`
  that filesystem current-state verification returns, so a superseded
  candidate, a stale committed retry, an absent head over populated pools, and
  each corruption or decode refusal are distinguishable to callers and
  preserve their underlying decode errors.
- Version-2 format marker, typed canonical intent and receipt construction,
  and record admission bind exact catalog, predecessor, root, definition,
  store, empty-state, checksum, digest, and synchronization-mask coordinates.
- `StoreMigrationPhase` freezes the 21 migration transitions with explicit
  storage and verification-first execution. The filesystem migration
  authority derives and revalidates one canonical intent from exact Linux
  root, namespace, head, catalog, and inventory coordinates; its streamed
  inventory is bounded and completely admits every immutable-pool artifact
  under the writer lock.
- Retention preflight combines expected-generation planning with
  deterministic closure verification, and authority-revalidated 17-phase
  orchestration returns its receipt only after durable cleanup.
- The `retention_format` and `migration_format` fuzz targets drive the
  retention root, manifest, and head decoders and all three migration record
  decoders.
- Specified `keep.segment-store/v2` retention values, root generations,
  liveness manifests, reader snapshots, one-way staged migration, exact crash
  boundaries, and reserved GC/disposition records. Validated public
  `RetentionNamespace`, namespace-digest, `RootGeneration`,
  `LivenessGeneration`, `RetentionAnchor`, realization profile, closure limits,
  and semantic root values now establish the core boundary. The canonical root
  encoder reproduces the independent version-2 golden bytes, and the decoder
  verifies framing, checksum, root digest, anchor-set digest, nested identities,
  resource bounds, canonical anchor order, and semantic invariants before
  admission. Validated global manifest values and their canonical encoder and
  decoder now reproduce the independent manifest fixture and enforce liveness
  history, namespace uniqueness, bounds, ordering, and all three integrity
  layers. Typed manifest lengths and semantic global heads now reproduce and
  admit the exact 144-byte head fixture with fixed framing, checksum-first
  semantic admission, and explicit generation-history laws. Storage-independent
  transition planning now compares absent or exact-generation expectations,
  admits only same-namespace exact successors, preserves expected and observed
  stale coordinates, and distinguishes byte-identical already-committed
  replay. Deterministic storage-independent closure verification now derives
  unique catalog members, enforces exact node, depth, encoded-byte, and
  physical-byte accounting, replays the registered storage profile,
  authenticates each complete retained blob, and emits a catalog-bound
  canonical closure digest. Version-1 immutable bytes remain authoritative;
  production version-2 writing remains unavailable until issue #19's
  executable evidence is complete.
- Accepted ADR-0009 defines caller-supplied retention namespaces,
  `BlobId`/`LayoutId` reconstruction anchors, fail-closed canonical closure,
  generation-checked retention publication, immutable liveness snapshots,
  release nonclaims, and GC evidence boundaries. This records the M4 design
  contract; it does not claim that retention transitions or GC are
  implemented.
- Checked catalog generations; canonical catalog and publication-head codecs;
  exact logical-record-to-segment admission with one bounded physical lookup
  plan, one scan per referenced segment, and refusal of every unreferenced
  caller-supplied segment during construction or admission; deterministic
  successor proofs; immutable reader snapshots; seeded parser fuzzing; and
  `BTreeMap` transition-model evidence for `keep.segment-store/v1`.
- Blocking `FilesystemCatalogPublisher` publication under a persistent
  kernel-managed writer lock and required `FilesystemPlatformAdmission`, with
  pinned directory capabilities,
  no-replacement immutable-pool links, complete post-link verification,
  explicit file and directory synchronization, transitive `head.next`
  verification, atomic `HEAD` replacement, and stale or recovery-required
  refusal before mutation. New filesystem segment publication consumes the
  sealed stage through its creating publisher, checks process-local publisher
  authority, and closes the writable handle before any immutable-pool link;
  publisher teardown closes every retained writable handle before releasing
  writer authority.
  Retry of an already-current complete candidate re-synchronizes the root and
  returns an explicit `CatalogPublicationOutcome::AlreadyPublished` receipt
  without repeating publication mutations. Retained `head.next` or
  `current.cat`, an unselected `current.seg`, and every fixed-name stage on an
  already-current retry now refuse at current-state verification before any
  publication mutation. An absent `HEAD` with any retained segment-pool or
  catalog-pool entry also refuses before mutation.
- Bounded `FilesystemCatalogSnapshot` restart loading that follows only exact
  checksummed head, catalog, and segment coordinates; refuses symbolic links,
  nonregular files, malformed or conflicting bytes, dangling entries, and
  resource-limit violations; and retains immutable bytes for pinned logical
  reads.
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
- Writer-authorized `FilesystemSegmentStage` creation for the fixed
  `current.seg` staging name, with a lifetime that retains the
  `FilesystemCatalogPublisher` lock, atomic no-replacement admission,
  preserved existing evidence, zero-origin writing, and no implicit cleanup
  from `Drop`.
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
  catalog stages, or the store root for `head.next`. Segment and catalog
  production are implemented; crash recovery remains assigned to issue #17.
  The golden corpus now includes a generation-2 catalog/head pair whose
  predecessor field is the exact generation-1 catalog digest.
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

### Changed

- Documentation refreshed after the version-two merge: the README, the
  version-two overview status, the closure and recovery status lines, the
  reconstruction contract's retention note, and the requirements ledger state
  what is implemented and what remains planned in #19, #21, and #97.
- The version-two retention and recovery pages each split their largest
  sections into `retention-publication.md` (closure admission and the
  generation transition) and `migration-recovery.md` (the one-way migration
  protocol and partial-migration recovery), so every page sits well under the
  300-line review threshold; the overview routes to both and the protocol
  contract pins their phrases.
- The README's two overlong link lines use reference-style links so every
  line fits the 80-column width without breaking a URL.
- Retention pool and namespace name predicates live beside their emitters in
  one module, and the namespace census classifies entries from the directory
  listing's file type instead of a metadata call per entry; its doc states the
  bound on the work it performs.
- The version-two admission type boundary is proven by a `compile_fail`
  doctest on `FilesystemRetentionPublicationAuthority::open`; the source
  contract keeps only per-file markers instead of exact signatures.
- Version-1 and version-2 pool and namespace filenames render their digest
  component through one shared lowercase-hexadecimal renderer instead of two
  identical private copies.
- Stage publishers and fixed-record readers share one exact-record module for
  no-follow non-blocking opens, exact-length reads, trailing-byte refusal,
  device-and-inode reverification, absence checks, and no-replacement links;
  the retention stage, the migration fixed-record stage, the retention
  current-state reader, the version-two record reader, and the version-one
  segment and catalog publishers' pool links consume it, the three pinned
  directory identity types collapse onto its `EntryIdentity`, and a contract
  test keeps ported modules from reimplementing those primitives.
- The retention FIFO laws run only on Linux through `mknodat`; the `mkfifo(1)`
  fallback for other hosts is removed, and a contract test keeps every module
  under `src/` free of process spawns. The fallback's spawned child briefly
  held copies of other tests' lock descriptors, which surfaced locally as an
  intermittent `WriterLock { source: Busy }` at reopen.
- The formats index and the version-two overview state what is implemented
  (one-way migration, version-two reopen, forward retention publication) and
  what remains planned in issue #19 (retention recovery, reader fencing,
  collection), instead of describing the whole version as planned.
- `FilesystemPlatformAdmissionError` is `#[non_exhaustive]`, so a future
  admission refusal can be added without a breaking change; a source contract
  pins the attribute.
- `ObservedRetentionState` carries the decoded head and manifest alongside
  their exact bytes, so disposition, already-committed verification, and
  predecessor verification decode each record once; the disposition names the
  observed state it was derived from, which removes an unreachable
  already-committed-without-a-head refusal.
- The retention head, catalog head, format marker, migration intent, and
  migration receipt readers take their fixed record lengths from the decoders
  that define those formats instead of restating the numbers; a contract test
  keeps the record-reading modules free of bare length literals.
- Version-one reopen refuses a migrated root, and `admit_version_two` owns the
  separate version-2 namespace boundary. Version-one recovery adapters refuse
  version-two residue at the store root before pinning any pool: a format
  marker, reader fence, migration record or stage, or a `retention`, `gc`, or
  `recovery` directory means recovery discard, completion, resume, and
  finalization refuse instead of rewriting a migrated store's version-one
  pools. Unknown entries continue to be refused by recovery name
  classification.
- The Rust quality gates now build the crate documentation with
  `cargo doc --workspace --no-deps --locked`, so a broken intra-doc link under
  `#![deny(warnings)]` fails CI instead of only failing anyone who documents
  the crate locally.
- The adapters module root now declares modules only: its public re-export
  surface lives in `adapters/exports.rs` and the recovery-stage adapters live
  under `adapters/recovery/` behind one facade. No public path changed; the
  root fell from 498 lines to 238 against the 500-line hard ceiling.
- Restart artifact reads use one exact read into a pre-reserved buffer followed
  by trailing-byte rejection; the interim chunked transfer layer, which pumped
  every artifact through an 8 KiB buffer without lowering peak memory, is
  removed with no change to refusal behaviour.
- Repository crash-matrix execution now terminates isolated writer process
  groups at all 105 canonical before/during/after coordinates, retains open
  writer and stage authority until termination, executes production
  initialization, segment-writing, catalog-publication, and recovery-discard
  protocols through fault-injecting port decorators, and verifies exact Golden
  File Worldline namespaces, bytes, hard links, released locks, recovery
  classifications, immutable artifacts, and published visible state after
  restart. CI runs the complete matrix in debug and optimized profiles.
- Production filesystem initialization now admits only one documented
  writable, non-casefolded Linux ext4 profile, independently applies it to
  every existing protocol directory, requires each child to share the root's
  device and mount identity, refuses ambiguous or foreign root namespaces
  before mutation, completes the canonical directory shape idempotently,
  retains writer authority, and returns only after synchronizing the root.
- Published filesystem stores can now reacquire writer authority without
  mutation through a typed platform-admission boundary that requires the exact
  initialized root shape plus a regular `HEAD`.
- Writer-lock acquisition now reopens `writer.lock` after kernel locking and
  refuses when the resolved entry no longer has the locked device and inode.
- Writer authority now also retains an advisory lock on the pinned store-root
  inode, so replacing `writer.lock` cannot split live cooperative authority.
- Recovery inventory now counts the root and three protocol directories before
  retaining names, enforces the configurable protocol-bounded entry ceiling,
  stops namespace counting at the first globally excessive entry, refuses
  count drift and duplicates exactly, and returns deterministic
  namespace-and-raw-byte ordering through a read-only storage port.
- Filesystem recovery inventory now pins the root and all three protocol
  directories without following links, verifies child-directory identity
  before and after bounded scanning, and preserves raw Linux entry-name bytes
  without mutating protocol state.
- Fixed recovery stages can now be fingerprinted relative to the pinned
  recovery inventory capability. Observation admits only regular files, never
  follows links, streams under the name-selected bound, and refuses entry
  replacement or length drift without mutating protocol state.
- Complete caller-supplied segment-stage bytes now classify as a validated
  reusable prefix, a complete admitted immutable segment, or an exact
  truncation only while every available fixed-framing byte remains canonical.
  Proven partial-framing corruption, complete-looking corruption, duplicate
  identities, and caller-policy excess remain typed refusals.
- Storage-independent reusable-segment recovery now plans only from an exact
  reusable assessment within the selected resource policy, consumes reopening
  authority, re-admits the materialized prefix against saved evidence, rebuilds
  digest and duplicate-identity state, and returns the ordinary append-only
  stage without rewriting admitted bytes.
- Filesystem reusable-segment recovery now retains pinned root, namespace, and
  writer-lock authority in the returned stage; reopens `current.seg` read-write
  without following links or truncation; bounds, materializes, and re-admits
  its exact prefix; recomputes exact stage evidence immediately before handoff;
  revalidates the final entry and append position; and refuses missing,
  changed, linked, replaced, or namespace-drifted evidence before writing.
- Complete caller-supplied catalog and candidate-head stages now distinguish
  exact fixed-header, declared-body, or fixed-width truncation from canonical
  bytes only while every available fixed-framing byte remains canonical.
  Proven partial-framing corruption, complete-looking corruption, and
  oversized stages remain typed refusals without claiming transitive catalog
  reachability.
- Read-only recovery assessment now admits materialized stage bytes only when
  their canonical-name stage, exact length, and recomputed versioned
  fingerprint equal prior observation evidence, then dispatches through the
  stage-selected semantic classifier.
- Explicit truncated-stage recovery now plans only from an exact semantic
  truncation, retains its evidence and reason, refuses changed evidence before
  mutation, and returns a discard receipt only after the name-selected parent
  directory is synchronized. An already absent stage remains an idempotent
  input and still requires synchronization.
- Filesystem truncated-stage discard now admits the platform, retains the root
  and `writer.lock` locks, pins every protocol namespace, reopens stage bytes
  without following links, refuses replacement or fingerprint drift before
  unlink, and synchronizes the typed `staging` or root parent before returning
  a receipt.
- Explicit complete-stage recovery now plans only from exact complete segment
  or catalog assessments, owns bounded stage evidence and immutable-pool
  coordinates, re-synchronizes an exact present stage before linking, verifies
  existing pool entries, synchronizes the selected pool before exact stage
  removal, and returns a valid-orphan receipt only after staging
  synchronization. It never creates or finalizes a publication head.
- Filesystem complete-stage recovery now retains pinned root and writer
  authority, revalidates exact stage evidence at synchronization and link
  boundaries, uses no-clobber immutable-pool links, never follows stage or pool
  links, preserves conflicting or replaced entries, verifies exact pool bytes,
  and accepts stage/pool, reappeared-stage, and completed pool-only retries.
- Storage-independent next-head recovery now binds a complete `head.next`
  assessment to its exact transitive catalog snapshot, admits only generation
  one over an uninitialized root or the exact successor of an expected current
  snapshot, synchronizes a ready candidate before replacement, distinguishes
  ready from already-finalized retries, and returns a receipt only after root
  synchronization.
- Filesystem next-head recovery now retains pinned root and writer authority,
  reconstructs complete current and candidate views under exact namespace and
  stage evidence, synchronizes and reverifies the candidate before atomic
  replacement, refuses reappeared candidates on retry, and returns only after
  root synchronization.
- Store initialization now exposes one storage-port state machine that admits
  the platform before mutation, opens and locks `writer.lock`, admits the three
  protocol directories in order, synchronizes the root, and preserves the
  exact failed phase without executing later transitions.
- Repository crash-matrix tooling now exposes one typed, ordered vocabulary for
  `KEEP-CRASH-001` through `KEEP-CRASH-035`. Each identifier is bound to its
  segment, catalog, head, recovery-discard, or initialization sequence, and
  only record append admits an occurrence counter.
- Catalog decoding now verifies the catalog checksum and physical digest before
  interpreting entry semantics. Corrupt identity-bearing bytes therefore fail
  at the integrity boundary instead of producing a semantic entry error.
- Filesystem catalog publisher construction consumes an unforgeable
  `FilesystemPlatformAdmission`; acquiring `FilesystemWriterLock` alone does
  not authorize production construction.
- Filesystem segment selection now consumes sealed stages through the publisher
  that created them. Process-local publisher authority prevents an unrelated
  metadata-equivalent `ClosedSegment` from authorizing retained
  `staging/current.seg` bytes.
- First catalog publication now admits an absent `HEAD` only after proving that
  both immutable pools are empty. Retained segment or catalog bytes require
  explicit recovery and remain untouched.
- Filesystem catalog publishers now retain no-follow, read-capable directory
  handles so required durability synchronization works on Linux instead of
  failing on `O_PATH` descriptors.
- The fuzz-workspace dependency gate now loads the reviewed repository
  `deny.toml` explicitly and admits non-Apache licenses only through exact
  package/version exceptions.
- Documentation corpus selection, pinned tool admission, Markdown and fragment
  checks, workflow linting, Dependabot coverage, and Node lock-graph policy now
  run through bounded Rust `xtask` code; CI and `cargo xtask verify` use that
  boundary, and the seven superseded Python checkers have been removed. The
  boundary rejects duplicate repository JSON fields and unlocked installer
  substitutions, admits only the exact reviewed Node lock artifact, retains
  simultaneous Markdown and link failures, retains both the primary tool
  failure and a simultaneous snapshot-cleanup failure, parses documentation
  workflow commands as YAML, rejects guarded or non-string `run` values,
  preserves declarations after Dependabot directory lists, refuses duplicate
  Dependabot YAML mapping keys before semantic admission, compares Dependabot
  maintenance fields as typed YAML values, requires every reviewed
  documentation CI command and the pinned Node setup action exactly once,
  admits only the exact pinned checkout and Node setup actions, requires actions
  and commands to execute in one reviewed order, rejects checkout overrides and
  unreviewed action steps, refuses
  alternate setup-node actions, requires the reviewed Node version, rejects
  unreviewed workflow/job run defaults and step execution fields, pins the
  documentation runner and job deadline, requires the exact top-level
  `contents: read` permission mapping, rejects guarded or failure-tolerant
  documentation jobs and required steps, refuses step mappings that define
  neither a reviewed action nor a run command, requires each Dependabot update
  block to choose exactly one directory field form, and requires the
  documentation workflow to run for pushes to `main` and every pull request,
  executes malformed Markdown and workflow evidence through the named
  `cargo xtask documentation-refusal-check` boundary instead of a
  zero-match-successful libtest substring filter,
  bounds each admitted documentation source to 4 MiB and each selected corpus
  to 64 MiB before external tools start, retains every selected source
  identity, refuses device, inode, size, modification-time, or change-time
  drift before and after each external tool, re-inventories complete corpus
  membership so newly added sources cannot bypass those tools, revalidates
  retained source identities after each rebuilt membership set so same-path
  replacement cannot cross the inventory boundary, executes the tools against
  a private snapshot copied from the admitted source descriptors so transient
  path substitution cannot redirect their reads, copies bounded regular
  non-Markdown namespace files exactly so link-fragment checks observe admitted
  target bytes, refuses nonregular namespace targets instead of substituting
  placeholders, retains each fixed policy file identity through semantic
  admission and external tool execution so a replaced path cannot validate
  bytes from a superseded file,
  and applies a two-minute deadline across Git inventory, validation-tool
  execution, and output collection. Validation tools clear the inherited
  environment and admit only the executable search path and `C` locale, so
  preload hooks and host-specific configuration cannot alter evidence.
  Git inventory uses a separate explicit profile that also nulls system and
  global configuration and disables optional locking, so repository overrides
  cannot redirect selection or cause incidental index writes. Failed Git
  inventory commands report their exit status and diagnostic before attempting
  path-stream decoding, so malformed stdout cannot mask the authoritative
  failure.
  Git-backed process fixtures clear the inherited environment, explicitly
  admit the executable search path and `C` locale, ignore system and global Git
  configuration, and preserve non-UTF-8 template paths without lossy
  conversion. Repository-backed Git fixtures share one bounded process
  authority with dedicated groups, null standard streams, and a two-minute
  deadline. Documentation Git inventory and tools start from one retained
  repository directory handle, so transient replacement of the ambient
  repository path cannot redirect validation. Retained and per-spawn directory
  descriptors are allocated at descriptor 3 or above, so child standard-stream
  setup cannot overwrite the working-directory authority.
  Source-structure inspection refuses nonregular executable candidates instead
  of silently omitting them from the Python and hard-line-limit policy.
  Terminal signals now become typed refusals while an external repository task
  is active, so captured and inherited child groups are killed and reaped
  before `xtask` returns. Captured-output readers finish while the process-group
  leader remains waitable, and no cleanup path can address its numeric group
  identity after the child has been reaped. Descendant cleanup evidence now
  uses a pre-established socket disconnect instead of elapsed-time reachability
  polling.
- Fuzz build and run plans now carry external process deadlines from the
  reviewed campaign policy. Both smoke and scheduled CI campaigns build every
  target under the separate build deadline before applying per-target run
  deadlines. Workflow contract evidence parses only executable `run` scalars in
  the reviewed fuzz jobs, so comments, names, and environment values cannot
  impersonate required build or run commands. Run deadlines use checked
  addition of the exploration budget and process-grace interval before
  process-group execution.
- The fuzz dependency-policy gate now grants exact MIT license exceptions to
  the reviewed `memchr` 2.8.3 and `zmij` 1.0.23 transitive dependencies while
  retaining Apache-2.0 as the default license allowlist.
- ChunkId v1 and CDC profile v1 conformance now run through one bounded Rust
  `cargo xtask conformance-check` command, including the external `b3sum`
  witness, reproducible Gear-table recipe, scalar and streaming FastCDC laws,
  source mutations, and exact boundary corpus; the three superseded Python
  programs have been removed.
- Golden File Worldline and protocol-conformance `b3sum` witnesses now share
  one external-digest process boundary. Its deadline begins before process
  spawn and stdin transfer, stdin is streamed without a combined preimage
  allocation, stdout and stderr have independent limits, and every timeout or
  collection failure kills and reaps the process group while retaining typed
  failure context. Child reaping and stalled-reader retirement use fixed
  per-step cleanup grace periods instead of blocking without limit.
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
  documented 500-physical-line hard maximum, including test modules and
  executable sources regardless of filename suffix.
- Repository source verification now uses capability-relative, no-follow file
  opens, normalizes symlink refusal at that capability boundary across Unix
  error conventions, starts Git inventory through a descriptor duplicated from
  that same admitted root, and verifies repository-root identity before
  inventory, after inventory, and after source scanning. Persistent or
  transient ambient-root substitution therefore cannot split path selection
  from source reads, and a source path replaced with a symlink is refused. The
  pure Rust boundary also refuses
  `.py`, `.pyw`, dot-only Python basenames, and Python shebangs in every
  executable regular file regardless of filename suffix, including raw
  non-UTF-8 Git paths and attached `env -S` interpreter strings. Environment
  shebangs parse exact and unambiguous abbreviated long options, combined
  short-option clusters, assignments, quoting, and split strings before
  classifying only the selected utility, so later command arguments cannot
  impersonate Python and unresolved utility substitutions fail closed. Tracked
  file modes are admitted from the Git index and must agree with the worktree,
  so a staged executable cannot defer Python screening until the next checkout.
  Source execution, shebang, and physical-line evidence now come from one
  admitted file descriptor whose identity is revalidated after each read phase,
  so path replacement or in-place mutation cannot splice different file states
  into one verification result.
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
- Duplicate-refusing repository JSON admission now has a one-mebibyte fuzz
  boundary with deterministic evidence for valid nested input, malformed JSON,
  excessive nesting, and duplicate members at nested object depth.
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

### Fixed

- Retention recovery capabilities and execution errors preserve typed exact-record refusals and original I/O causes through `RetentionStorageError`. Callers can inspect `RetentionRecoveryError::storage_error()` directly; shared forward-stage adapters retain the typed source when adapting to their existing I/O boundary (#99).

- Reader-fence tests now replace the lock inode during acquisition, distinguish that case from nonempty lock bytes, and require the exact kernel contention errno; isolated mutations verify that the identity and exclusion assertions detect removed protections (#99).

- Reader double collection preserves catalog length and the complete retention head, so changes to validated length or predecessor coordinates cannot be collapsed into an unchanged generation/digest pair. The public `RetentionViewCoordinates` catalog tuple now includes `CatalogLength`, and its retention field carries `RetentionHead` (#99).

- Fenced retention readers preserve selected catalog and segment admission failures as `FilesystemRetentionSnapshotError::Catalog` with their exact restart source, while coordinate failures remain `View` errors; a moving head retries the speculative catalog result (#99).

- Fenced retention readers compare the opened root's restart-stable device and inode with the jointly admitted migration records before acquiring a fence or loading a catalog. A foreign binding refuses at reader admission with the exact typed identity coordinate, expected value, and observed value (#99).

- Retention recovery reauthenticates the current catalog and selected segments and replays staged-root closure laws before publication effects. Forward publication shares live verification after pure preflight and preserves prepared catalog binding. Recovery exposes an explicit catalog loading policy; the default aggregate retained-segment cap is one protocol-maximum segment (1 GiB), with larger selections refusing before mutation (#99).

- Retention recovery refuses truncated manifest or head stages whose earlier complete stage or immutable pool link is missing, preserving impossible-prefix evidence instead of deleting it (#99).

- Retention recovery requires an uncommitted head's staged root to satisfy the same generation and predecessor rules as root-only and manifest-only recovery. Already committed cleanup remains admissible (#99).

- Retention recovery reopens and authenticates the manifest-selected predecessor before publication effects, refusing missing, corrupt, or substituted roots while preserving retained evidence.

- Retention recovery refuses successor manifests that add, drop, or alter namespace entries unrelated to the staged root before linking the manifest or finalizing the head.

- Retention recovery binds a complete staged head to its manifest's exact byte length and predecessor before committing or cleaning up. Canonical but inconsistent heads refuse with existing typed planning errors and preserve published and retained evidence (#99).

- Retention recovery refuses byte-identical root or manifest pool substitutions before committing the head or removing stages. Pool observation binds exact bytes to the retained stage's device and inode and preserves the typed pool refusal (#99).

- Retention recovery admits namespace names and pool kinds before stage observation or mutation. Direct and publication-triggered recovery preserve retained evidence when unknown entries or noncanonical pool names refuse (#99).

- Recovery synchronizes the exact complete retention stage before creating its immutable root or manifest pool link or replacing the retention head. A synchronization error refuses before that publication; the retained on-disk stage remains recovery evidence.

- Interrupted retention stages with complete zero root or liveness generations now refuse recovery with the existing typed generation error before any mutation (#99). Incomplete generation fields remain eligible for canonical-prefix recovery; complete-record decoding and durable encodings are unchanged.

- Retention recovery refuses an interrupted root, manifest, or head stage when any available magic, version, fixed width, flag, or reserved byte contradicts the canonical format (#99). The typed `PrefixByteMismatch` names the actual byte and offset without inventing missing bytes; refusal preserves retained evidence. Canonical interrupted prefixes remain recoverable. Complete-record decoding and on-disk bytes are unchanged; the decode-error enums gain a diagnostic variant.

- Subprocess test-support readiness admits a queued valid signal after sender exit, preventing a false process-group cleanup failure without retrying the test gate.

- The version-one catalog ledger names its executable ordering integration
  target, duplicate-refusal module, and filesystem publication unit-test owner
  instead of absent test files (#148).

- Benchmark report admission refuses identical or conflicting repeated metadata
  coordinates with a typed failure naming the coordinate (issue #142). Complete
  canonical report admission remains in progress. Ordered metadata keys, exact
  headers/catalogs, complete metric widths and canonical unsigned decimals now
  refuse malformed reports while admitting the committed historical fixture.
  Fixed measurement policies and consistent per-row sample counts are admitted
  before publication. Ratios, throughput, percentile order and reused-chunk
  bounds now refuse inconsistent evidence with typed expected/observed failures;
  arithmetic and numeric parsing refuse without approximation. Byte/count
  metrics admit only their portable unsigned 64-bit range; timing and throughput
  retain unsigned 128-bit precision. Publication requires an immutable admitted
  report; refusal preserves the prior artifact and retained recovery stage.
  Direct parser admission enforces the same one-MiB input ceiling as subprocess
  capture before decoding, and preserves UTF-8 error sources. A dedicated
  I/O-free fuzz facade exercises the production parser with a deterministic
  historical seed; seed preparation and campaign discovery include the target.

- The benchmark workload catalog describes current range authentication once
  before output, matching its single-pass metric definitions (#71).

- Public reference range-read documentation describes its single pre-output
  authentication and immutable emission without a second hash (#71).

- Reference reconstruction and range reads authenticate each selected chunk
  once, then emit its immutable stored bytes after all required checks pass.
  Benchmark authenticated-byte counters now count one verification pass.
  Hash-count and exact range-accounting regressions cover issue #71.

- The version-one ledger discloses the pending repository-task sealed-stage
  capability escape (#146) while preserving the immutability requirement (#69).

- The version-one segment ledger points to the existing filesystem-stage
  regression module for exclusive staging and dropped-stage evidence (#69).

- The version-one recovery overview links its implemented transitive
  candidate-view admission to the next-head boundary and requirement anchors (#69).

- Version-one publication documentation assigns unchecked publisher
  construction to the crash harness and fault injection to its decorators (#69).

- Version-one crash documentation identifies its 105 cases as a subset of
  the complete command, which also executes version-two migration cases (#69).

- Version-one recovery documentation distinguishes streamed inventory
  fingerprint evidence from caller-supplied classifier bytes and the segment
  resumer's separate materialization (#69).

- Living version-1 format pages describe implemented initialization,
  platform admission, publication, restart, recovery, and process-death
  evidence rather than completed issue-era plans (#69). Historical issue
  references and existing requirement/test anchors remain available.

- The migration transition-ledger guard rejects noncanonical line endings,
  extra rows, misplaced discard claims, and counterfeit completion postures.
  Recovery posture and namespace interruption checks use their exact columns.
- The independent migration restart model propagates missing occurrence,
  arithmetic, and extent-conversion failures instead of normalizing them.
- Crash-case admission refuses out-of-range migration namespace occurrences
  before child execution; variable segment-record occurrences remain valid.
- Receipt-only migration residue before a complete namespace reports
  `ReceiptBeforeMarker`, preserving the exact missing-prerequisite boundary.
- Migration residue observation rejects non-regular entries with the typed
  kind refusal before opening them, retaining the post-open kind recheck.
- Migration resumption is internal to verified recovery admission; external
  callers cannot bypass current authority verification and residue adoption.

- Version-two reopen compares persisted device and inode coordinates while
  retaining mount identity as same-process migration evidence (#97).
  Simulated-remount reopen and exact moved-root refusals are regression-tested;
  migration record bytes and the forward publication protocol are unchanged.

- Reference staging exposes its fixed buffer/state allowance and measured
  capacity, deduplication, and repeated-content allocation laws (#74).
  Chunk payload limits and separately bounded metadata are documented without
  claiming constant total memory or durability. Source-failure cleanup after
  partial staging preserves committed content and releases staging heap.

- Source-structure admission refuses the nine Rust source basenames prohibited
  by AGENTS.md with a typed repository-relative filename diagnostic (#144).
  Existing filesystem, Python and line-size checks remain in force.

- Benchmark source-identity laws exercise real Git cleanliness independently
  of ambient CPU-model availability; benchmark hardware admission stays strict.

Review corrections to the unreleased retention and migration work above; none
of these shipped in a release.

- Retention and migration crash tasks share the migration repository-task
  constructor after main integration instead of defining it twice (#19).
- Explicit retention recovery verifies the pinned retention and immutable-pool
  directory identities before any recovery effect, preserving retained stage
  evidence when a protocol directory was replaced (#19, PR #99).
- Retention publication reopens the catalog pool entry this store's `HEAD`
  selects, bounded by the head's declared length, and requires it to decode
  to that generation and digest (`CatalogAbsent`, `CatalogRefused`,
  `CatalogChanged`), so a preparation verified before the catalog was lost no
  longer publishes; closure-member segments are documented as not re-read.
- `FilesystemVersionTwoAdmission` retains the `retention`, `roots`, and
  `manifests` capabilities it admitted and hands them to the publication
  authority, and current-state verification requires those names to still
  resolve to the pinned directories (`ProtocolDirectoryReplaced`), so a
  directory swapped in after reopen is neither opened nor published into.
- Migration refuses a version-one store whose `staging` directory holds a
  retained stage, so an interrupted version-one publication is recovered
  before the intent exists instead of being stranded behind the version-two
  markers.
- A retention store already holding more than 4,096 namespace directories
  refuses every publication as `NamespaceCapacity`, including a successor in
  an existing namespace, instead of only refusing the 4,097th directory.
- Retention pool names whose generation component encodes zero refuse as
  noncanonical; root and liveness generations are positive, so the census no
  longer treats an impossible coordinate as a canonical entry.
- An already-committed retry presented over an absent retention head refuses
  as `CommittedRetryOverAbsentHead` instead of the misnamed
  `StaleCommittedRetry`; the successor-over-absent-head law now downcasts to
  `ExpectedCurrentOverAbsentHead`.
- The root-link and namespace-synchronization phases require the root handed
  to them to name the namespace the attempt admitted, refusing with
  `RetentionCurrentStateRefusal::AttemptNamespaceDisagreed` instead of
  synchronizing whatever directory the attempt holds.
- `FilesystemRecoveryStageError::LengthChanged` reports the stage's actual
  on-disk length in `observed` when trailing bytes are found, instead of the
  expected length plus the one byte that detected them.
- Version-two record admission and catalog-head binding carry their decode
  errors as sources: `VersionTwoRecordRefusal` names which of `FORMAT`,
  `migration.intent`, or `migration.receipt` refused and preserves the
  decoder's diagnosis, and `CatalogHeadRefused` carries the
  `PublicationHeadDecodeError`.
- Retention namespace admission refuses with typed
  `RetentionCurrentStateRefusal` variants (`UnknownRetentionEntry`,
  `NonNamespaceEntry`, `NoncanonicalPoolEntry`, `NamespaceCapacity`,
  `NamespaceExpectationViolated`) instead of bare `InvalidData` strings, so
  callers can tell an unknown entry from a full namespace pool.
- Every coordinate a retention publication retains between phases now lives on
  one publication attempt that current-state verification creates and the next
  verification or cleanup discards: a refused verification admits no later
  phase, a stage handle from an interrupted run is never reused, and namespace
  admission refuses a directory the claimed expectation excludes with
  `RetentionCurrentStateRefusal::NamespaceExpectationViolated`.
- An already-committed retention retry reopens the manifest entry and the root
  pool bytes the head selects and refuses absent, changed, or corrupt evidence
  instead of inferring the commit from head agreement alone.
- Retention current-state verification binds this store's catalog `HEAD` to
  the closure's catalog coordinates for every disposition: a preparation built
  from another store's `CatalogSnapshot` refuses before any forward write, and
  an already-committed retry no longer returns a receipt citing a catalog this
  store does not name. Observing the current state also requires the head's
  predecessor digest to equal its manifest's predecessor.
- A successor retention publication reopens the predecessor root the current
  manifest selects, bounded by the root format's maximum encoded length, and
  requires it to decode to exactly that generation and digest; a namespace
  directory alone no longer stands in for an available predecessor.
- Version-two reopen compares the reopened root's device, mount, and file
  identity with the coordinates bound into `migration.intent` and refuses a
  relocated or restored store with
  `FilesystemPlatformAdmissionError::RootIdentityChanged`, matching the
  comparison the migration authority makes before mutation.
- Version-two namespace admission descends into the protocol directories:
  `retention` must hold both immutable pools, `gc` must be empty, and
  `recovery` must hold exactly an empty `dispositions`, matching what the
  migration writer verifies at completion, so post-migration drift refuses at
  admission instead of surfacing later as a pinning failure.
- Retention publication admits the complete `retention` namespace before any
  forward write: only `HEAD`, `roots`, and `manifests` may exist, every
  namespace directory is 64 lowercase hex, and every pool entry is a regular
  `<generation>-<digest>` file with its canonical suffix.
- Existing namespace directories, including recovery-protected orphans, count
  against the 4,096 namespace ceiling, and a candidate whose namespace would
  be the 4,097th refuses before its root stage exists.
- Current-state verification binds on-disk state to the claimed expectation:
  an absent `retention/HEAD` is the empty state only while both pools are
  empty and admits only an initial head with no predecessor; a namespace
  directory must be absent for an `Absent` expectation and present for a
  `Current` one. Every mismatch refuses as recovery-required before any stage
  is written.
- Each retention stage synchronization also synchronizes the `retention`
  directory, so the `root.next`, `manifest.next`, and `head.next` entries are
  durable before the namespace directory, pool links, or head replacement that
  depend on them.
- Every read-side reopen in retention publication and version-two admission
  opens with `O_NONBLOCK`, so a FIFO planted at `retention/HEAD`, `FORMAT`, or
  a pool name refuses by kind instead of blocking under the writer lock.
- A retention stage left behind by a failed write is documented as recovery
  evidence: it is never unlinked, and the next publication refuses until
  recovery classifies it, exactly as the segment-stage doctrine already states.
- The test and repository-task admission bypass probes root identity through a
  lenient path that records an unreported `STATX_MNT_ID` as zero, so the suite
  and crash matrix run on kernels older than 5.8; every production probe still
  refuses without a reported mount identity.
