# Requirements, Compatibility, and Evidence

This page owns the requirement ledger, compatibility boundary, security
nonclaims, and golden evidence for `keep.segment-store/v1`.

## Requirement ledger

<!-- markdownlint-disable MD013 -->

| ID | Requirement | Design evidence | Status |
| --- | --- | --- | --- |
| `KEEP-STORE-001` | Segment, record, seal, catalog, entry, and head have exact versioned canonical grammars | Field tables and golden artifacts | Specified in #14 |
| `KEEP-STORE-002` | Every persistent integer is fixed-width big-endian and checked | Canonical primitives and bounds | Specified in #14 |
| `KEEP-STORE-003` | Checksums and physical digests use named domain-separated preimages | Checksum formulas and fixture oracle | Specified in #14 |
| `KEEP-STORE-004` | Sealed segments and published catalogs are immutable | State model and publication protocol | Specified in #14 |
| `KEEP-STORE-005` | Logical identity is separate from physical location | Record and catalog entry grammars | Specified in #14 |
| `KEEP-STORE-006` | Catalog ordering and duplicate refusal are deterministic | Catalog-entry rules | Specified in #14 |
| `KEEP-STORE-007` | Publication has explicit flush, sync, no-clobber link, unlink, head-replacement, and directory-sync order | `KEEP-CRASH-001`–`026` | Specified in #14 |
| `KEEP-STORE-008` | File presence alone proves no state | Core law and visibility table | Specified in #14 |
| `KEEP-STORE-009` | Writer exclusion uses one persistent kernel-managed lock | Writer-exclusion contract | Specified in #14 |
| `KEEP-STORE-010` | Readers observe one complete immutable generation | Reader-snapshot protocol | Specified in #14 |
| `KEEP-STORE-011` | Opening is observational and recovery is explicit | Recovery protocol | Specified in #14 |
| `KEEP-STORE-012` | Required recovery classes remain distinct | Recovery classification ledger | Specified in #14 |
| `KEEP-STORE-013` | Ambiguous or corrupt durable state is refused | Recovery and refusal order | Specified in #14 |
| `KEEP-STORE-014` | Memory, record, segment, and catalog sizes are bounded | Bounds table | Specified in #14 |
| `KEEP-STORE-015` | Unsupported filesystem semantics fail closed | Platform contract | Specified in #14 |
| `KEEP-STORE-016` | No Echo, Graft, Git, or application policy enters the protocol | ADR-0005 and physical namespace | Specified in #14 |
| `KEEP-STORE-017` | Catalog locations equal verified top-level segment-record spans | Catalog-entry admission | Specified in #14 |
| `KEEP-STORE-018` | Truncated-stage discard has explicit unlink and directory-sync crash boundaries | `KEEP-CRASH-027`–`028` | Specified in #14 |
| `KEEP-STORE-019` | Leftover next-head recovery either finalizes an exact successor or discards explicit evidence durably | `KEEP-CRASH-025`–`026`, `029`–`030` | Specified in #14 |
| `KEEP-STORE-020` | Initialization is writer-locked, idempotent, recoverable from every partial canonical namespace set, and admitted only after root synchronization | `KEEP-CRASH-031`–`035` | Specified in #14 |
| `KEEP-STORE-021` | Explicit recovery completes a durable fixed-name stage into its immutable pool without publishing a head | `KEEP-CRASH-008`–`012`, `016`–`020` | Specified in #14 |

<!-- markdownlint-enable MD013 -->

## Segment implementation evidence

The issue #15 boundary implements immutable segment creation and admission. It
does not claim catalog publication, namespace durability, restart recovery,
retention, or garbage collection.

<!-- markdownlint-disable MD013 -->

| ID | Implemented requirement | Executable evidence | Status |
| --- | --- | --- | --- |
| `KEEP-SEGMENT-001` | Segment, record-header, record, and seal codecs match the frozen version-1 bytes exactly | `tests/segment_header.rs`, `tests/segment_record_header.rs`, `tests/segment_record.rs`, `tests/segment_seal.rs` | Implemented in #15 |
| `KEEP-SEGMENT-002` | Staged and sealed writer states are distinct, consuming types | `tests/segment_writer.rs` | Implemented in #15 |
| `KEEP-SEGMENT-003` | Short, interrupted, zero-progress, invalid-count, storage, permission, flush, and synchronization failures retain exact phases and offsets | `tests/segment_writer/write_contract_laws.rs`, `tests/segment_writer/refusal_laws.rs`, `tests/segment_writer/durability_laws.rs` | Implemented in #15 |
| `KEEP-SEGMENT-004` | Complete-segment admission verifies bounds, framing, checksums, logical identities, duplicate refusal, terminal state, and physical digest before exposure | `tests/segment.rs`, `tests/segment/identity_laws.rs`, `tests/segment/framing_laws.rs` | Implemented in #15 |
| `KEEP-SEGMENT-005` | The public sealed receipt exposes no mutable stage handle | `src/adapters/sealed_segment.rs` | Implemented in #15 |
| `KEEP-SEGMENT-006` | Malformed, unsupported, partial, conflicting, and corrupt input returns boundary-typed errors | `tests/segment_header/mutation_laws.rs`, `tests/segment_record_header/framing_laws.rs`, `tests/segment_seal/framing_laws.rs`, `tests/segment/identity_laws.rs` | Implemented in #15 |
| `KEEP-SEGMENT-007` | Record, nested-layout, segment-length, and temporary identity-index allocation remain explicitly bounded | `tests/segment_memory.rs`, `tests/segment_record_memory.rs`, `tests/segment_seal_memory.rs` | Implemented in #15 |
| `KEEP-SEGMENT-008` | Filesystem staging uses exclusive fixed-name creation and never enumerates storage as a content index | `tests/segment_filesystem_stage.rs`, `src/adapters/filesystem_segment_stage.rs` | Implemented in #15 |
| `KEEP-SEGMENT-009` | Every implemented write and durability phase has deterministic fault injection, while dropped unsealed stages preserve recovery evidence | `tests/segment_writer/`, `tests/segment_filesystem_stage.rs` | Implemented in #15 |
| `KEEP-SEGMENT-010` | Every public segment-format parser boundary is fuzzed from canonical deterministic seeds | `fuzz/fuzz_targets/segment_format.rs`, `xtask/src/fuzz_seed_corpus/segment_seeds.rs` | Implemented in #15 |

<!-- markdownlint-enable MD013 -->

## Catalog implementation evidence

Issue #16 implements catalog-generation admission, writer-locked filesystem
publication mechanics, and immutable reader snapshots. Production publisher
construction requires `FilesystemPlatformAdmission`, whose platform-checked
producer is implemented as the initialization slice of issue #17. Explicit
recovery remains separate work.

<!-- markdownlint-disable MD013 -->

| ID | Implemented requirement | Oracle | Executable evidence | Status |
| --- | --- | --- | --- | --- |
| `KEEP-CATALOG-001` | `CatalogGeneration` admits positive values and refuses overflow when deriving a successor | Checked scalar model | `tests/catalog_generation.rs` | Implemented in #16 |
| `KEEP-CATALOG-002` | Catalog and publication-head codecs reproduce every frozen version-1 artifact and refuse noncanonical bytes; catalog checksum and digest admission precede entry semantics | Independent golden corpus and mutation precedence oracle | `tests/catalog.rs`, `tests/catalog/integrity_laws.rs`, `tests/publication_head.rs` | Implemented in #16 |
| `KEEP-CATALOG-003` | Catalog entries are sorted by logical identity and duplicate keys are refused independently of input order | Ordered reference map | `tests/catalog_ordering.rs` | Implemented in #16 |
| `KEEP-CATALOG-004` | Every catalog location equals a verified top-level record span in the exact named segment; construction and admission require every supplied segment to be referenced, and admission scans each referenced segment once | Bounded grouped lookup plan and golden artifacts | `tests/catalog_encoding.rs`, `tests/catalog_locations.rs` | Implemented in #16 |
| `KEEP-CATALOG-005` | Publication admits only the exact expected successor and reports expected and observed generation and digest on staleness | Generation transition model | `tests/catalog_transition.rs` | Implemented in #16 |
| `KEEP-CATALOG-006` | A reader retains one complete catalog generation and never combines it with a concurrent head | Immutable snapshot model | `tests/catalog_snapshot.rs` | Implemented in #16 |
| `KEEP-CATALOG-007` | Retained kernel locks on the pinned store root and persistent writer file exclude a second cooperative writer even if the directory entry is replaced; neither lock is deleted on release, and lock ownership alone cannot construct a publisher without platform admission | Multi-handle lock model, replacement fixture, and construction architecture law | `tests/catalog_writer_lock.rs`, `tests/catalog_filesystem_publication/directory_laws.rs` | Implemented in #16; replacement-hardened in #17 |
| `KEEP-CATALOG-008` | Segment, catalog, and head publication follows the documented synchronization order; retained fixed-name recovery state refuses before mutation; an absent head requires empty immutable pools; retry of an already-current candidate performs no publication mutation and re-synchronizes the root | Fault-recording port and filesystem fixtures | `tests/catalog_publication.rs`, `tests/catalog_filesystem_publication.rs` | Implemented in #16 |
| `KEEP-CATALOG-009` | Restart loading refuses corrupt, unsupported, noncanonical, dangling, and conflicting catalog state | Corruption matrix | `tests/catalog_restart.rs` | Implemented in #16 |
| `KEEP-CATALOG-010` | Model-based transitions and lookups agree with a deterministic `BTreeMap` catalog | Boring reference catalog | `tests/catalog_model.rs` | Implemented in #16 |
| `KEEP-CATALOG-011` | Every public catalog and publication-head parser boundary is fuzzed from canonical deterministic seeds | Canonical generation-1, generation-2, and bundle artifacts | `fuzz/fuzz_targets/catalog_format.rs`, `xtask/src/fuzz_seed_corpus/catalog_seeds.rs` | Implemented in #16 |

<!-- markdownlint-enable MD013 -->

## Recovery implementation evidence

Issue #17 implements crash injection and explicit recovery in independently
reviewable slices. The first slice freezes executable crash-point identity and
sequence ownership. The second slice establishes the ordered initialization
state machine and exact failure phases. The third slice binds that state
machine to a fail-closed Linux ext4 adapter and canonical namespace. These
slices now also classify canonical recovery names before opening artifact
bytes, bind materialized bytes back to prior stage evidence, and dispatch
complete caller-supplied fixed-stage bytes through their name-selected
classifiers. An exact truncation assessment may now authorize one
evidence-bound, retry-safe discard through a semantic storage port. These
slices now bind that discard to pinned writer-authorized filesystem storage.
A complete segment or catalog assessment may now authorize an owned,
evidence-bound valid-orphan transition through a semantic storage port, and
the filesystem completer now binds that transition to pinned writer-authorized
storage. A complete next-head assessment and exact transitive catalog snapshot
may now authorize a transition-checked finalization through a semantic storage
port. These slices do not yet claim filesystem next-head finalization or
process-death injection.

<!-- markdownlint-disable MD013 -->

| ID | Implemented requirement | Oracle | Executable evidence | Status |
| --- | --- | --- | --- | --- |
| `KEEP-RECOVERY-001` | Crash identifiers `KEEP-CRASH-001` through `KEEP-CRASH-035` form one contiguous typed vocabulary, map to the exact owning protocol sequence, and admit an occurrence counter only for record append | Ordered identifier-and-sequence ledger | `xtask/tests/durability_crash_point_contract.rs` | Implemented in #17 |
| `KEEP-RECOVERY-002` | Initialization admits the platform before mutation, opens and locks the writer file, admits `staging`, `segments`, and `catalogs` in order, and returns a receipt only after root synchronization; every failed operation retains its exact phase and prevents later transitions | Fault-recording initialization port | `tests/store_initialization.rs` | Implemented in #17 |
| `KEEP-RECOVERY-003` | Production initialization admits only a writable, non-casefolded Linux ext4 root, refuses any noncanonical root entry before mutation, completes an empty or partial canonical namespace without replacing evidence, excludes a second initializer, and retains writer authority through the synchronized receipt | Capability-relative filesystem fixture and exact platform-profile classifier | `src/adapters/filesystem_store_initializer_tests.rs`, `src/adapters/filesystem_platform_profile.rs`, `tests/store_initialization.rs` | Implemented in #17 |
| `KEEP-RECOVERY-004` | Writer authority is returned only when the locked handle still has the exact device and inode resolved by the canonical `writer.lock` entry after kernel acquisition | Deterministic lock-entry replacement fixture | `src/adapters/filesystem_writer_lock_tests.rs` | Implemented in #17 |
| `KEEP-RECOVERY-005` | Recovery counts the root and three protocol directories in fixed order before retaining names, refuses at the configured or protocol entry ceiling with the exact observed-at-least count, then returns one duplicate-free inventory sorted by namespace and raw name bytes | Fault-recording inventory port | `tests/recovery_inventory.rs` | Implemented in #17 |
| `KEEP-RECOVERY-006` | Filesystem inventory pins the admitted root and protocol directories without following links, verifies child-directory identity before and after scanning, stops each count at the remaining global budget plus one, preserves raw Linux entry-name bytes, and performs no protocol mutation | Capability-relative filesystem fixture | `src/adapters/filesystem_recovery_inventory_tests.rs`, `tests/recovery_inventory.rs` | Implemented in #17 |
| `KEEP-RECOVERY-007` | Name classification requires the four initialized root entries, admits only fixed protocol names and canonical pool coordinates in their owning namespaces, refuses simultaneous fixed recovery stages before artifact reads, and moves a refused raw name without duplicating its allocation | Canonical-name matrix and allocation counter | `tests/recovery_name_classification.rs`, `tests/recovery_name_classification_memory.rs` | Implemented in #17 |
| `KEEP-RECOVERY-008` | Stage evidence is fingerprinted through a zero-allocation bounded streaming reader under the named recovery domain; metadata and observed bytes cannot exceed the name-selected protocol maximum, and failures retain exact stage and offset | Independent framing oracle, adversarial reader matrix, and allocation counter | `tests/recovery_stage_fingerprint.rs`, `tests/recovery_stage_fingerprint_memory.rs` | Implemented in #17 |
| `KEEP-RECOVERY-009` | Filesystem stage observation uses the pinned inventory capability, never follows a fixed-stage link, admits only regular files, and refuses entry replacement or length drift after bounded fingerprinting | Capability-relative replacement fixtures | `src/adapters/filesystem_recovery_stage_tests.rs` | Implemented in #17 |
| `KEEP-RECOVERY-010` | Whole-byte segment-stage classification distinguishes a validated reusable prefix, a complete admitted immutable segment, and exact header, record, or seal truncation; complete-looking corruption, duplicates, and resource-limit excess remain typed refusals | Canonical prefix and corruption matrix | `tests/recovery_segment_classification.rs`, `tests/recovery_segment_classification/*.rs` | Implemented in #17 |
| `KEEP-RECOVERY-011` | Whole-byte catalog and next-head stage classification distinguishes exact fixed-header, declared-body, and fixed-width truncation from complete canonical bytes; complete-looking corruption and oversize remain typed format or metadata refusals | Canonical publication-artifact truncation and corruption matrix | `tests/recovery_publication_stage_classification.rs`, `tests/recovery_publication_stage_classification/*.rs` | Implemented in #17 |
| `KEEP-RECOVERY-012` | Read-only semantic assessment admits materialized stage bytes only when the canonical-name stage, exact observed length, and `KEEP:RECOVERY:STAGE\0` fingerprint equal prior evidence, then dispatches through the name-selected segment, catalog, or next-head classifier | Evidence-binding mutation matrix and canonical stage assessments | `tests/recovery_stage_assessment.rs`, `tests/recovery_stage_assessment/*.rs` | Implemented in #17 |
| `KEEP-RECOVERY-013` | Explicit discard plans only from an exact truncation assessment, retains the observation evidence and typed truncation reason, refuses changed evidence without mutation, synchronizes the name-selected parent after exact removal or admitted absence, and returns a receipt only after synchronization | Truncation-planning, evidence-drift, operation-order, and retry matrix | `tests/recovery_stage_discard.rs`, `tests/recovery_stage_discard/*.rs` | Implemented in #17 |
| `KEEP-RECOVERY-014` | Filesystem discard retains root and `writer.lock` authority, pins every protocol directory, never follows a fixed-stage link, revalidates bounded fingerprint and entry identity before unlink, refuses drift without mutation, and synchronizes the typed parent after removal or admitted absence | Exact removal, absent retry, mismatch, symlink, replacement, and writer-exclusion matrix | `src/adapters/filesystem_recovery_stage_discard_tests.rs`, `src/adapters/filesystem_recovery_stage_discard_tests/fixture.rs` | Implemented in #17 |
| `KEEP-RECOVERY-015` | Immutable-pool completion plans only from exact complete segment or catalog assessments, owns bounded evidence and validated coordinates, re-synchronizes an exact present stage before linking, verifies an existing pool entry before admission, synchronizes the pool before exact stage removal, synchronizes staging before receipt, accepts completed retries, and never finalizes a head | Complete-only planning, operation-order, staged-file-sync, pool-conflict, and retry matrix | `tests/recovery_stage_completion.rs`, `tests/recovery_stage_completion/*.rs` | Implemented in #17 |
| `KEEP-RECOVERY-016` | Filesystem completion retains root and `writer.lock` authority, pins every protocol directory, re-synchronizes and re-fingerprints exact stage evidence before no-clobber link, never follows stage or pool links, verifies exact pool evidence before removal, preserves conflicting and replaced entries, accepts exact stage/pool, reappeared-stage, and pool-only retries, and returns only after pool and staging synchronization | Segment/catalog completion, three retry states, conflict, link, replacement, stale-evidence, missing-artifact, and writer-exclusion matrix | `src/adapters/filesystem_recovery_stage_completion_tests.rs`, `src/adapters/filesystem_recovery_stage_completion_tests/*.rs` | Implemented in #17 |
| `KEEP-RECOVERY-017` | Next-head finalization plans only from an exact complete `head.next` assessment and its matching complete transitive catalog snapshot, admits only generation one over an uninitialized root or the exact successor of an expected current snapshot, atomically replaces only a ready candidate, accepts an already-finalized retry, and returns a receipt only after root synchronization | Snapshot-coordinate, transition, operation-order, fault-stop, and post-replacement retry matrix | `tests/recovery_next_head_finalization.rs`, `tests/recovery_next_head_finalization/*.rs` | Implemented in #17 |

<!-- markdownlint-enable MD013 -->

## Compatibility and migration

The byte grammars, magic values, field widths and order, endianness, kinds,
flags, algorithm coordinates, bounds, checksum domains, catalog ordering,
physical-name grammar, generation law, crash-point identifiers, publication
order, and recovery classifications are compatibility commitments.

Changing any of them requires a new store protocol version and an explicit
migration decision. A migration writes and verifies new immutable artifacts,
publishes a new compatible head under its accepted protocol, and never
reinterprets existing bytes in place.

Logical `BlobId`, `ChunkId`, and `LayoutId` remain stable when their exact
logical bytes and canonical plans remain unchanged.

## Security and privacy

Version 1 provides integrity checks, not writer authentication or
confidentiality. Logical identities, lengths, record boundaries, catalog
membership, and physical reuse are visible metadata. An attacker controlling
all bytes can recompute unkeyed checksums.

All path operations are capability-relative and no-follow. Untrusted counts
and lengths are bounded before allocation. Diagnostics retain expected and
observed typed coordinates but do not include plaintext payloads, unbounded
paths, secrets, or terminal control characters.

## Golden evidence

The
[durable segment-store corpus](../../../conformance/segment-store/v1/README.md)
contains:

- an empty sealed segment;
- a one-byte chunk segment bound to an existing independent `ChunkId`;
- catalog generation 1 naming its exact physical record;
- a publication head naming that exact catalog;
- catalog generation 2 naming the generation-1 digest as its exact
  predecessor, plus its publication head;
- a two-record segment carrying the chunk and its canonical flat layout;
- a two-entry catalog proving chunk-before-layout order and checked offsets;
- a publication head naming that cross-kind catalog;
- exact canonical bytes, checksums, physical digests, lengths, counts, and
  offsets; and
- the complete `KEEP-CRASH-001`–`KEEP-CRASH-035` transition ledger.

The test-only Rust oracle reconstructs every artifact directly from these
tables and formulas. The issue #15 segment implementation matches the frozen
segment corpus and adds parser fuzzing and corruption evidence. Issue #16
matches the catalog and publication-head corpus, executes the documented
publication order through a real filesystem adapter, reconstructs exact
immutable restart snapshots, and adds deterministic transition-model and
seeded parser-fuzz evidence. Crash-injection and explicit recovery remain
owned by issue #17.

The format-local tradeoffs are recorded in the
[colocated rationale](rationale.md).
