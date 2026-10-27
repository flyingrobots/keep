# Requirements and Evidence

This ledger owns stable requirements for `keep.segment-store/v2`. A planned
case is not evidence.

## Retention transitions

<!-- markdownlint-disable MD013 -->

| ID | Requirement | Evidence | Status |
| --- | --- | --- | --- |
| `KEEP-RETENTION-001` | `RetentionNamespace`, `RootGeneration`, `LivenessGeneration`, profile coordinates, limits, anchors, and digests are validated typed values | `tests/retention_values.rs`, `tests/retention_root_encoding.rs`, and typed verified anchor-set evidence in `tests/retention_root_decoding.rs` | Implemented |
| `KEEP-RETENTION-002` | Root, manifest, and head codecs implement the exact canonical grammars and fixed bounds | independent golden corpus plus `tests/retention_root_encoding.rs`, `tests/retention_root_decoding.rs`, `tests/retention_manifest_codec.rs`, and `tests/retention_head_codec.rs` | Implemented |
| `KEEP-RETENTION-003` | Every structural field, truncation boundary, ordering law, duplicate, overflow, flag, reserved byte, digest, checksum, and trailing byte has a precise refusal | seeded `retention_format` fuzz target plus the root, manifest, and head corruption matrix; mutation coverage remains | In progress in #19 |
| `KEEP-RETENTION-004` | Retain and release compare expected and observed generations and publish exact successors only | unforgeable readiness and preflight proofs in `tests/retention_transition.rs` and `tests/retention_preflight.rs`; exact successor preparation and complete receipt evidence in `tests/retention_publication_preparation.rs` and `tests/retention_publication_execution.rs`; writer-locked initial filesystem publication in `filesystem_retention_storage_tests`; observed-head successor publication, exact predecessor binding, and absent-head refusal in `filesystem_retention_successor_tests`; the store's catalog head must name the closure's catalog generation and digest before any forward write in `filesystem_retention_catalog_tests`; a head whose predecessor disagrees with its manifest refuses in `filesystem_retention_current_tests`; a successor reopens and decodes the manifest-selected predecessor root and refuses an absent or changed one in `filesystem_retention_expectation_tests` | Implemented |
| `KEEP-RETENTION-005` | Closure derivation is deterministic, bounded, cycle-safe, fail-closed, and verifies complete blob reconstruction | exact accounting, reconstruction, adversarial-catalog, and exhaustive model laws in `tests/retention_closure.rs`; corrupt members refuse through the inherited segment-record admission laws and seeded `segment_format` fuzz target routed by `closure-corruption.md` | Implemented |
| `KEEP-RETENTION-006` | Publication follows the exact ordered durability protocol, including new namespace-directory admission and retention of fixed-stage evidence until head commit, and returns only after cleanup synchronization | typed vocabulary and blocking port in `tests/retention_publication_phase.rs` and `tests/retention_publication_storage.rs`; ordered execution, conditional namespace sync, and all 17 exact storage-fault boundaries in `tests/retention_publication_execution.rs`; production 17-phase forward filesystem execution, exclusive staging, byte-equal inode-substitution refusal, and retained-stage recovery refusal in `filesystem_retention_storage_tests`; orphan namespace directories count against the 4,096 ceiling and refuse a new namespace before any stage is written in `filesystem_retention_capacity_tests`; crash injection remains | In progress in #19 |
| `KEEP-RETENTION-007` | Complete-stage recovery preserves canonical history; incomplete stages require disposition before mutation | Complete publication-prefix recovery and reader-state laws remain; incomplete direct/publication recovery and process-death laws now require typed refusal and preserved bytes. See [landing ledger](../../testing-evidence/retention-landing.md). | Implemented under the bounded scope merged in #99; automatic incomplete-stage disposition deferred to #155 |
| `KEEP-RETENTION-008` | Readers double-collect catalog and retention heads and bind one complete catalog, manifest, and root-generation view under a `ReaderFence` | `ReaderFence` holds a shared kernel lock on a verified zero-length `reader.lock`; `collect_retention_view` accepts a view only when both head coordinates agree before and after loading and refuses an exhausted attempt limit (`retention_view_collector_tests`); `FilesystemRetentionSnapshot` binds the catalog snapshot, retention head, and manifest under the fence and verifies each selected root on demand while the fence is held, refusing a substituted root and a replaced fence, and two readers share the fence while an exclusive lock waits (`filesystem_retention_snapshot_tests`); Linux public snapshot process laws prove live-reader exclusion, SIGKILL fence release with persistent inode preservation, and kernel-queued new-reader exclusion until collection releases the fence (`tests/reader_fence_process.rs`; [evidence](../../testing-evidence/reader-fence-process.md)) | Implemented |
| `KEEP-RETENTION-009` | Exact already-committed retry is idempotent only while its successor remains current | byte-identical planning in `tests/retention_transition.rs`; authority-revalidated zero-mutation retry receipt in `tests/retention_publication_execution.rs`; exact already-committed filesystem retry with a byte-identical retention witness in `filesystem_retention_storage_tests`; superseded-candidate filesystem refusal with zero mutation in `filesystem_retention_successor_tests`; committed retry reopens the head-selected manifest entry and root pool bytes, refusing absent, changed, or corrupt evidence in `filesystem_retention_current_tests`; every refusal is a typed `RetentionCurrentStateRefusal` source, with superseded, committed-root-absent, committed-root-changed, and head-absent-with-artifacts pinned by downcast | Implemented |
| `KEEP-RETENTION-010` | Model operation sequences agree with a deterministic namespace-to-anchor-set map and never admit caller identity, paths, clocks, or application policy | all three-operation histories over initial publications of two namespaces, successor, release, restore, byte-identical retry and stale initial (343 histories, each in a fresh initialized store) compare fenced namespace generations, anchor sets and liveness against an operation-derived model after every step; exact typed refusals remain checked. The count describes exploration, not correctness. See [release/restore evidence](../../testing-evidence/retention-release-restore-model.md). Core architecture checks remain separate static evidence. | Implemented; release/restore model coverage added for #128 |
| `KEEP-RETENTION-011` | Catalog publication over retention state admits the current retention head, manifest, and every manifest root under writer authority; verifies every retained closure against the candidate before any catalog storage call; and refuses `HEAD` replacement if the retention head changed | port-level laws in `tests/retained_catalog_publication.rs`: dropped and substituted members refuse with the exact missing identity and no catalog stage; every manifest root is verified wherever it sorts; absent, corrupt, substituted, and contradictory head, manifest, and root evidence refuse with typed expected and observed state; a head changed, removed, or published before commit refuses after `head.next` verification without `replace_head`; an exhaustive 24-case candidate-by-root-set model and a repeat-run determinism check | Implemented at the storage port; the filesystem version-2 publisher and version-2 next-head finalization use it (#198, `KEEP-RETENTION-012` and `013`) |
| `KEEP-RETENTION-012` | Under version-2 writer authority, one staged segment and a catalog successor publish only through the retained-closure gate, with no-follow bounded retention reads, head absence admitted only when uncontradicted, fixed retention stages refused, and no reader fence taken | `FilesystemVersionTwoCatalogPublisher`; `tests/version_two_catalog_writer.rs`: a preserving successor publishes, reopens, and reads both blobs while the retention root still verifies; a pinned reader view keeps its generation across publication; a candidate that drops retained content, a contradicted head absence, and a fixed retention stage each refuse with the exact typed error and an unchanged store witness; runtime RED for the contradiction and stage checks | Implemented for one staged segment per publication; segment rollover is #121 |
| `KEEP-RETENTION-013` | Version-2 segment and catalog stage recovery admits version-2 root entries as known, reuses the version-1 executors, and finalizes a staged `head.next` only after the retained-closure gate admits it | `FilesystemVersionTwoRecoveryAdmission`, `classify_version_two_recovery_names`, and the `open_version_two` constructors; `tests/version_two_catalog_writer.rs` covers classification, completion, discard, reusable-prefix segment resume, and finalization that preserves or would lose retained content (runtime RED with the gate result ignored); `filesystem_version_two_catalog_recovery_tests` recovers injected interruptions after segment link, catalog-stage creation, and `head.next` synchronization | Implemented |
| `KEEP-RETENTION-014` | Version-1 writer and recovery entry points refuse version-2 roots, and version-2 entry points refuse version-1 roots, with typed errors | `tests/version_two_catalog_writer.rs` version laws, including cross-publisher segment selection | Implemented |
| `KEEP-RETENTION-015` | Process death in the version-2 writer and its recovery reaches documented recoverable states | `cargo xtask durability-crash-matrix --sequence version-two-catalog` runs `KEEP-CRASH-074..=077`, 12 cases, through recovery and durable reads; `KEEP-CRASH-078..=080` (process death inside version-2 recovery) are planned | In progress: writer rows implemented, recovery rows planned |

<!-- markdownlint-enable MD013 -->

## Format marker

<!-- markdownlint-disable MD013 -->

| ID | Requirement | Evidence | Status |
| --- | --- | --- | --- |
| `KEEP-FORMAT-001` | `FORMAT` has a complete fixed byte table, named domains, a checksum, the registered definition digest, and exact admission; every admission reads it no-follow, bounded, and refuses trailing bytes | `tests/store_format_marker.rs`; independent golden oracle `xtask/tests/retention_store_v2_format_oracle.rs`; corrupt and oversized marker refusals in `filesystem_version_two_admission_tests` and `verification_store_admission_tests`; seeded `format_marker` fuzz target | Implemented (#198); marker obligation of retired `KEEP-MIGRATION-002` |

<!-- markdownlint-enable MD013 -->

## Initialization

<!-- markdownlint-disable MD013 -->

| ID | Requirement | Evidence | Status |
| --- | --- | --- | --- |
| `KEEP-INITIALIZATION-001` | Direct initialization creates exactly the committed namespace under exclusive writer authority, links `FORMAT` from a durable `FORMAT.next` without replacement, and returns authority only after the commit and cleanup synchronizations and complete admission | `a_fresh_root_commits_to_exactly_the_initialized_namespace` in `initialization_resume_tests`; production-profile laws in `tests/single_format.rs` | Implemented (#198) |
| `KEEP-INITIALIZATION-002` | Every interrupted state classifies as exactly one prefix and resumes idempotently; every other nonempty root, including a committed one, refuses with typed evidence before any mutation, writer-lock creation included | `every_interrupted_boundary_classifies_and_resumes_to_one_store` (every boundary of a recorded run, against a model of the protocol table) and the incomplete-stage and sync-failure laws in `initialization_resume_tests`; foreign, out-of-order, stray-retention, foreign-stage, nonempty-lock, linked-directory, committed, and copied-marker refusals with unchanged witnesses in `initialization_prefix_tests` | Implemented (#198) |
| `KEEP-INITIALIZATION-003` | Admission requires a committed marker: writers and readers refuse `FORMAT.next` as `InitializationIncomplete` and a missing `FORMAT` as `MissingFormatMarker`; a committed store without catalog `HEAD` admits writers, refuses readers as `Unpublished`, and its first publication proves both absences instead of assuming them | `filesystem_version_two_admission_tests`; `tests/single_format.rs` initialization laws, including a first publication refused by a stray retention root with an unchanged store | Implemented (#198) |
| `KEEP-INITIALIZATION-004` | Process death before, during, and after every initialization boundary reaches a documented state that `initialize` recovers | `cargo xtask durability-crash-matrix --sequence version-two-initialization` runs 41 cases at `KEEP-CRASH-081..=091`, checks the exact inventory, marker bytes and inode, admission result, and resumed state, then publishes, retains, and reconstructs the first blob | Implemented (#198); process death only, not power loss |
| `KEEP-INITIALIZATION-005` | Concurrent initialization attempts have exactly one initializer: the other refuses on writer authority and, after commit, as already initialized | `concurrent_attempts_have_exactly_one_initializer` in `initialization_resume_tests` | Implemented (#198) |
| `KEEP-INITIALIZATION-006` | One public-API sequence initializes, ingests, reopens, reconstructs exact bytes, retains, publishes a second catalog, interrupts publication, recovers, and reads both retained contents; a candidate that would lose retained content refuses with its evidence intact | `the_single_format_serves_every_operation_from_initialization_to_recovery` in `tests/single_format.rs` | Implemented (#198) |

<!-- markdownlint-enable MD013 -->

## Portability

<!-- markdownlint-disable MD013 -->

| ID | Requirement | Evidence | Status |
| --- | --- | --- | --- |
| `KEEP-PORTABILITY-001` | A valid quiescent copy opens and reads the same content; nothing persisted binds a store to its directory | `a_valid_quiescent_copy_opens_and_reads_the_same_content` in `tests/single_format.rs` | Implemented (#198) |
| `KEEP-PORTABILITY-002` | A corrupt or incomplete copy refuses exactly as the original with the same damage would | `a_corrupt_copy_refuses_exactly_as_the_corrupt_original_would` and `an_incomplete_copy_refuses_exactly_as_the_incomplete_original_would` | Implemented (#198) |
| `KEEP-PORTABILITY-003` | A lock file or protocol directory replaced during an operation still refuses | `a_replaced_lock_or_protocol_directory_during_an_operation_still_refuses` | Implemented (#198) |
| `KEEP-PORTABILITY-004` | A copy and its original take independent locks and diverge independently while unchanged content keeps its identity and bytes | `a_copy_and_its_original_diverge_independently_and_keep_shared_identity` | Implemented (#198) |
| `KEEP-PORTABILITY-005` | Backup round trip: close, copy, make the original unavailable, reopen and verify the copy, interrupt a new write on the copy, and recover it | `a_backup_copy_reopens_verifies_and_recovers_an_interrupted_write` | Implemented (#198) |

<!-- markdownlint-enable MD013 -->

## Retired identifiers

The `KEEP-MIGRATION-001` to `-008` requirements and the `KEEP-CRASH-053` to
`-073` process-death rows were retired with migration. They are never reused.
The [retirement ledger](retired-identifiers.md) maps each one to its last
evidence and states where its obligation went.

## Garbage collection reservation

<!-- markdownlint-disable MD013 -->

| ID | Requirement | Evidence | Status |
| --- | --- | --- | --- |
| `KEEP-GC-001` | Version 2 specifies exact bounded GC intent, receipt, and recovery-disposition grammars but refuses their presence until their parser and recovery protocol are implemented | namespace admission tests | Planned in #21 |
| `KEEP-GC-002` | GC intent, receipt, disposition, reader-fence, retirement, compaction, and recovery laws implement ADR-0009 without changing logical identity | golden-format, model-based, corruption, crash-injection, benchmark, and fuzz evidence | Planned in #21 |

<!-- markdownlint-enable MD013 -->

## Compatibility and nonclaims

- Version 2 keeps the version-1 segment, catalog, and publication-head
  grammars byte for byte. A version-1 store is not converted: version-2 code
  refuses it, and no migration or downgrade exists.
- Retention evidence proves a bounded physical reconstruction claim, not
  application meaning, causal ownership, future policy, or secure erasure.
- A fresh forward writer is not proof that version 2 is restart-safe or
  production-admitted; partial-prefix recovery and crash evidence remain
  mandatory. This applies to retention publication exactly as it applies to
  initialization: the filesystem publication writer never continues a retained
  stage; it recovers complete evidence first, preserving incomplete stages for disposition,
  finalizing a complete head, and refusing a complete orphan until explicit
  disposition. A stage left behind by a failed write is recovery evidence like
  any crash residue and is classified the same way.
- Publication binds this store's catalog `HEAD` to the verified closure and
  reopens the head-selected catalog and its segments under authority. Forward
  publication and recovery both replay the bounded live closure before effects.
- Admission does not detect an unintended but valid store or an older valid
  state; a copy of a store is a second, independent store.
- Benchmarks are required before performance-sensitive retention or
  initialization optimization.
