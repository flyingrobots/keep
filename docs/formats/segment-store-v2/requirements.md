# Requirements and Evidence

This ledger owns stable requirements for `keep.segment-store/v2`. A planned
case is not evidence.

## Retention transitions

<!-- markdownlint-disable MD013 -->

| ID | Requirement | Evidence | Status |
| --- | --- | --- | --- |
| `KEEP-RETENTION-001` | `RetentionNamespace`, `RootGeneration`, `LivenessGeneration`, profile coordinates, limits, anchors, and digests are validated typed values | `tests/retention_values.rs` and `tests/retention_root_encoding.rs` | Implemented |
| `KEEP-RETENTION-002` | Root, manifest, and head codecs implement the exact canonical grammars and fixed bounds | independent golden corpus plus `tests/retention_root_encoding.rs`, `tests/retention_root_decoding.rs`, `tests/retention_manifest_codec.rs`, and `tests/retention_head_codec.rs` | Implemented |
| `KEEP-RETENTION-003` | Every structural field, truncation boundary, ordering law, duplicate, overflow, flag, reserved byte, digest, checksum, and trailing byte has a precise refusal | corruption matrix and fuzz | Planned in #19 |
| `KEEP-RETENTION-004` | Retain and release compare expected and observed generations and publish exact successors only | unforgeable storage-independent readiness and preflight proofs with preserved disposition and coordinates in `tests/retention_transition.rs` and `tests/retention_preflight.rs`; exact global successor preparation in `tests/retention_publication_preparation.rs`; publication evidence remains | In progress in #19 |
| `KEEP-RETENTION-005` | Closure derivation is deterministic, bounded, cycle-safe, fail-closed, and verifies complete blob reconstruction | exact accounting, reconstruction, adversarial-catalog, and exhaustive model laws in `tests/retention_closure.rs`; corrupt members refuse through the inherited segment-record admission laws and seeded `segment_format` fuzz target routed by `closure-corruption.md` | Implemented |
| `KEEP-RETENTION-006` | Publication follows the exact ordered durability protocol, including new namespace-directory admission and retention of fixed-stage evidence until head commit, and returns only after cleanup synchronization | typed 17-phase vocabulary in `tests/retention_publication_phase.rs` and blocking capability port in `tests/retention_publication_storage.rs`; ordered execution and `KEEP-CRASH-036..=052` crash-injection tests remain | In progress in #19 |
| `KEEP-RETENTION-007` | Restart resolves every fixed-stage crash prefix to one documented lawful state or typed ambiguity | debug and release crash matrix | Planned in #19 |
| `KEEP-RETENTION-008` | Readers double-collect catalog and retention heads and bind one complete catalog, manifest, and root-generation view under a `ReaderFence` | immutable snapshot and concurrency tests | Planned in #19 |
| `KEEP-RETENTION-009` | Exact already-committed retry is idempotent only while its successor remains current | byte-identical readiness and stale-state planning in `tests/retention_transition.rs`; publication retry remains | In progress in #19 |
| `KEEP-RETENTION-010` | Model operation sequences agree with a deterministic namespace-to-anchor-set map and never admit caller identity, paths, clocks, or application policy | model-based and source-architecture tests | Planned in #19 |

<!-- markdownlint-enable MD013 -->

## Migration

<!-- markdownlint-disable MD013 -->

| ID | Requirement | Evidence | Status |
| --- | --- | --- | --- |
| `KEEP-MIGRATION-001` | Exact version-1 stores remain admitted until a durable migration artifact exists | compatibility fixtures | Planned in #19 |
| `KEEP-MIGRATION-002` | Intent and receipt have complete fixed byte tables, named domains, bounds, checksums, deterministic store identity, and exact initial-state digests | golden-format fixtures | Planned in #19 |
| `KEEP-MIGRATION-003` | Migration revalidates version-1 head, catalog, pools, root identity, and writer authority before mutation | capability-relative integration tests | Planned in #19 |
| `KEEP-MIGRATION-004` | Every partial migration prefix continues idempotently under writer authority | state-machine and recovery tests | Planned in #19 |
| `KEEP-MIGRATION-005` | Unknown, out-of-order, substituted, corrupt, conflicting, or changed evidence is unrecoverable ambiguity | corruption and mutation matrix | Planned in #19 |
| `KEEP-MIGRATION-006` | Migration never rewrites or deletes admitted version-1 immutable bytes | byte-for-byte before/after witness | Planned in #19 |
| `KEEP-MIGRATION-007` | Process death around every intent stage, canonical link, namespace prefix, marker stage, receipt stage, cleanup, and synchronization boundary reaches a documented lawful state | `KEEP-CRASH-053..=073` crash-injection matrix | Planned in #19 |
| `KEEP-MIGRATION-008` | Version-1 admission refuses every version-2 or partial-migration artifact after migration begins | compatibility and fuzz tests | Planned in #19 |

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
- A version-2 format specification is not proof that a version-2 production
  writer exists.
- Benchmarks are required before performance-sensitive retention or migration
  optimization.
