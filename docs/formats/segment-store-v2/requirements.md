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
| `KEEP-RETENTION-007` | Restart resolves every fixed-stage crash prefix to one documented lawful state or typed ambiguity | recovery-required refusals before any mutation in `filesystem_retention_expectation_tests`: an absent head over populated pools, a non-initial head prepared against an absent head, an orphan directory for a namespace expected absent, and an absent directory for a namespace expected current; debug and release crash matrix remains; replaced protocol directories, an absent or changed head-selected catalog, an over-full census, zero-generation pool names, and a stage retained by a failed write refuse in `filesystem_retention_*_tests`; storage-independent classification of every fixed-stage crash prefix (discard, link and protect, finalize, clean up, or typed refusal) in `recovery_planner_tests`; every publication prefix 0 through 18, each mid-write truncation, and successor prefixes recover in-process to the documented state, idempotently, with the forward retry reporting the predicted outcome, in `filesystem_retention_recovery_prefix_tests` | In progress in #19 |
| `KEEP-RETENTION-008` | Readers double-collect catalog and retention heads and bind one complete catalog, manifest, and root-generation view under a `ReaderFence` | immutable snapshot and concurrency tests | Planned in #19 |
| `KEEP-RETENTION-009` | Exact already-committed retry is idempotent only while its successor remains current | byte-identical planning in `tests/retention_transition.rs`; authority-revalidated zero-mutation retry receipt in `tests/retention_publication_execution.rs`; exact already-committed filesystem retry with a byte-identical retention witness in `filesystem_retention_storage_tests`; superseded-candidate filesystem refusal with zero mutation in `filesystem_retention_successor_tests`; committed retry reopens the head-selected manifest entry and root pool bytes, refusing absent, changed, or corrupt evidence in `filesystem_retention_current_tests`; every refusal is a typed `RetentionCurrentStateRefusal` source, with superseded, committed-root-absent, committed-root-changed, and head-absent-with-artifacts pinned by downcast | Implemented |
| `KEEP-RETENTION-010` | Model operation sequences agree with a deterministic namespace-to-anchor-set map and never admit caller identity, paths, clocks, or application policy | model-based and source-architecture tests | Planned in #19 |

<!-- markdownlint-enable MD013 -->

## Migration

<!-- markdownlint-disable MD013 -->

| ID | Requirement | Evidence | Status |
| --- | --- | --- | --- |
| `KEEP-MIGRATION-001` | Exact version-1 stores remain admitted until a durable migration artifact exists | compatibility fixtures | Planned in #19 |
| `KEEP-MIGRATION-002` | Format marker, intent, and receipt have complete fixed byte tables, named domains, bounds, checksums, deterministic store identity, and exact initial-state digests | exact admission in `tests/store_format_marker.rs`, `tests/store_migration_intent.rs`, and `tests/store_migration_receipt.rs`; canonical construction in `tests/store_migration_intent_encoding.rs` and `tests/store_migration_receipt_encoding.rs`; seeded `migration_format` fuzz target | Implemented |
| `KEEP-MIGRATION-003` | Migration revalidates version-1 head, catalog, pools, root identity, and writer authority before mutation | bounded canonical pool inventory in `tests/store_migration_inventory.rs`; writer-locked filesystem pool admission in `filesystem_inventory_*_tests`; exact authority observation and drift refusal in `filesystem_migration_authority_tests`; verification-first execution in `tests/store_migration_execution.rs`; fresh filesystem integration and post-publication drift refusal in `filesystem_migration_storage_tests`; a version-one store still holding a retained stage refuses before the intent is observed in `filesystem_migration_storage_tests` | Implemented |
| `KEEP-MIGRATION-004` | Every partial migration prefix continues idempotently under writer authority | state-machine and recovery tests | Planned in #19 |
| `KEEP-MIGRATION-005` | Unknown, out-of-order, substituted, corrupt, conflicting, or changed evidence is unrecoverable ambiguity | forward-execution stage preservation, byte-equal inode-substitution, out-of-order-prefix, and post-publication drift laws in `filesystem_migration_storage_tests`; unknown `retention` entries, non-digest namespace directories, and noncanonical pool names refuse before any retention stage is written in `filesystem_retention_namespace_tests`; restart corruption and mutation matrix remains | In progress in #19 |
| `KEEP-MIGRATION-006` | Migration never rewrites or deletes admitted version-1 immutable bytes | exact segment, catalog, and head before/after witness in `filesystem_migration_storage_tests`; restart-path evidence remains | In progress in #19 |
| `KEEP-MIGRATION-007` | Process death around every intent stage, canonical link, namespace prefix, marker stage, receipt stage, cleanup, and synchronization boundary reaches a documented lawful state | ordered phases and capabilities in `tests/store_migration_phase.rs` and `tests/store_migration_storage.rs`; exact phase-failure execution in `tests/store_migration_execution.rs`; production 21-phase forward execution in `filesystem_migration_storage_tests`; `KEEP-CRASH-053..=073` process-death matrix remains | In progress in #19 |
| `KEEP-MIGRATION-008` | Version-1 admission refuses every version-2 or partial-migration artifact after migration begins | `FORMAT` refusal before mutation in `filesystem_migration_authority_tests`; exact version-1 reopen refusal of a migrated root and separate version-2 namespace admission in `filesystem_initialization_namespace`; version-2 reopen returns a distinct `FilesystemVersionTwoAdmission` that no version-1 publisher can consume (pinned by `tests/version_two_admission_contract.rs`), admits every version-2 protocol directory under the Linux profile, and jointly admits the exact marker, intent, and receipt before returning writer authority, with aliased-directory, corrupt, oversized, and mutually inconsistent record refusals in `filesystem_version_two_admission_tests` and `filesystem_platform_profile_tests`; remaining compatibility and fuzz matrix | In progress in #19 |

<!-- markdownlint-enable MD013 -->

## Garbage collection reservation

<!-- markdownlint-disable MD013 -->

| ID | Requirement | Evidence | Status |
| --- | --- | --- | --- |
| `KEEP-GC-001` | Version 2 specifies exact bounded GC intent, receipt, and recovery-disposition grammars but refuses their presence until their parser and recovery protocol are implemented | namespace admission tests | Planned in #21 |
| `KEEP-GC-002` | GC intent, receipt, disposition, reader-fence, retirement, compaction, and recovery laws implement ADR-0009 without changing logical identity | golden-format, model-based, corruption, crash-injection, benchmark, and fuzz evidence | Planned in #21 |

<!-- markdownlint-enable MD013 -->

## Compatibility and nonclaims

- Version 2 preserves exact version-1 segment, catalog, and publication-head
  bytes.
- Migration is one-way and provides no downgrade.
- Retention evidence proves a bounded physical reconstruction claim, not
  application meaning, causal ownership, future policy, or secure erasure.
- A fresh forward writer is not proof that version 2 is restart-safe or
  production-admitted; partial-prefix recovery and crash evidence remain
  mandatory. This applies to retention publication exactly as it applies to
  migration: the filesystem publication writer refuses every retained stage
  instead of continuing it. A stage left behind by a failed write is recovery
  evidence like any crash residue; it is never unlinked, and the next
  publication refuses until recovery classifies it.
- Publication binds this store's catalog `HEAD` to the verified closure and
  reopens the head-selected catalog pool entry under authority, but it does
  not re-read closure-member segments: every read authenticates them, and
  their re-verification under authority belongs to retention recovery.
- Benchmarks are required before performance-sensitive retention or migration
  optimization.
