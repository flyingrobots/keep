# Changelog

All notable changes to Keep will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project intends to follow [Semantic Versioning](https://semver.org/)
after its public API and format compatibility policies are established.

## [Unreleased]

### Changed

- Version-2 marker, migration-intent, and completion-receipt admission now bind
  exact catalog, predecessor, root, definition, store, empty-state, checksum,
  digest, and synchronization-mask coordinates. Retention preflight combines
  expected-generation planning with deterministic closure verification;
  authority-revalidated 17-phase orchestration returns an unforgeable
  complete-coordinate receipt after durable cleanup.
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

### Added

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
