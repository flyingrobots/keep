# GC and Disposition Records

This page owns the canonical planned `GcRetirementIntent`,
`GcRetirementReceipt`, and `RecoveryDispositionReceipt` byte grammars for
`keep.segment-store/v2`.

Implemented: all three codecs, the planner, explicit disposition, and
retirement. `GcRetirementIntent` admits a canonical candidate set over its
coordinates; `CanonicalGcRetirementIntent` and `AdmittedGcRetirementIntent`
reproduce and admit the frozen `one-candidate-gc-intent.hex`;
`CanonicalGcRetirementReceipt` and `AdmittedGcRetirementReceipt` bind a
receipt to its admitted intent, and `decode_unbound` reads one without an
intent. `CanonicalRecoveryDispositionReceipt` and
`AdmittedRecoveryDispositionReceipt` reproduce and admit the frozen
`one-orphan-retire-disposition.hex` over the enumerations registered in
`definition.tsv`. The protocol that writes `gc/intent` and `gc/receipt`, its
recovery, and its process-death matrix are owned by
[GC execution and recovery](gc-execution.md); explicit disposition by
[recovery](recovery.md#explicit-disposition-of-protected-orphans);
identity-preserving compaction by [compaction](compaction.md). Namespace
admission admits exactly those records as regular files and nothing else in
`gc`. Re-encoding compaction remains **Planned in #21**.

## Common rules

All integers are unsigned and big-endian. Flags and reserved bytes are zero.
Every length and count is checked before allocation. Decoders reject truncation,
trailing bytes, unsupported versions, unknown mandatory flags, nonzero reserved
bytes, overflow, noncanonical ordering, duplicates, digest or checksum
mismatch, and values above fixed ceilings.

Every digest and checksum uses domain-separated BLAKE3-256. Fixed names are
never replaced to obtain idempotence.

## GC retirement intent

`GcRetirementIntent` consists of:

```text
320-byte fixed-width header
candidate-count × 72-byte candidate entries
32-byte intent digest
32-byte checksum
```

The maximum candidate count is 65,536. Its maximum encoded length is
4,718,976 bytes.

<!-- markdownlint-disable MD013 -->

| Offset | Width | Field | Canonical value |
| ---: | ---: | --- | --- |
| 0 | 16 | magic | `KEEP:GC:INTENT2\0` |
| 16 | 2 | version | `2` |
| 18 | 2 | header length | `320` |
| 20 | 4 | flags | `0` |
| 24 | 8 | total record length | derived exact length |
| 32 | 8 | GC generation | positive checked successor |
| 40 | 2 | candidate width | `72` |
| 42 | 2 | reserved | zero |
| 44 | 4 | candidate count | `1..=65,536` |
| 48 | 8 | liveness generation | exact current value |
| 56 | 32 | retention-manifest digest | exact current digest |
| 88 | 8 | catalog generation | exact successor value |
| 96 | 32 | catalog digest | names no candidate segment |
| 128 | 4 | realization-profile identity | exact retained profile |
| 132 | 4 | realization-profile version | exact retained profile |
| 136 | 32 | realization-profile digest | exact retained profile |
| 168 | 32 | catalog-successor proof digest | complete verified proof |
| 200 | 32 | segment-pool identity digest | exact admitted pool |
| 232 | 32 | disposition-set digest | exact admitted receipts |
| 264 | 8 | reader-lock device identity | exact locked file |
| 272 | 8 | reader-lock mount identity | exact locked file |
| 280 | 8 | reader-lock file identity | exact locked file |
| 288 | 32 | candidate-entry-set digest | exact canonical entries |

<!-- markdownlint-enable MD013 -->

Each 72-byte candidate entry is:

| Offset | Width | Field |
| ---: | ---: | --- |
| 0 | 32 | segment digest |
| 32 | 8 | segment length |
| 40 | 32 | complete verification-evidence digest |

Candidate entries use canonical segment-digest order and are duplicate-free.
The entry-set, intent, and checksum domains are:

```text
keep.gc-candidate-set/v2\0
keep.gc-retirement-intent/v2\0
keep.gc-retirement-intent-checksum/v2\0
```

The checksum covers header, entries, and intent digest. The intent digest
covers the header and entries.

## GC retirement receipt

`GcRetirementReceipt` is exactly 320 bytes:

<!-- markdownlint-disable MD013 -->

| Offset | Width | Field | Canonical value |
| ---: | ---: | --- | --- |
| 0 | 16 | magic | `KEEP:GC:RECEIPT2` |
| 16 | 2 | version | `2` |
| 18 | 2 | record length | `320` |
| 20 | 4 | flags | `0` |
| 24 | 8 | GC generation | exact intent generation |
| 32 | 32 | intent digest | exact durable intent |
| 64 | 32 | retired candidate-set digest | exact intent set |
| 96 | 32 | post-retirement pool-state digest | verified synchronized pool |
| 128 | 8 | liveness generation | revalidated exact value |
| 136 | 32 | retention-manifest digest | revalidated exact value |
| 168 | 8 | catalog generation | revalidated exact value |
| 176 | 32 | catalog digest | revalidated exact value |
| 208 | 8 | reader-lock device identity | exact exclusive lock |
| 216 | 8 | reader-lock mount identity | exact exclusive lock |
| 224 | 8 | reader-lock file identity | exact exclusive lock |
| 232 | 8 | completed synchronization count | exact intent-derived count |
| 240 | 48 | reserved | zero |
| 288 | 32 | checksum | BLAKE3-256 over bytes `0..288` |

<!-- markdownlint-enable MD013 -->

The checksum domain is `keep.gc-retirement-receipt-checksum/v2\0`.

## Recovery disposition receipt

`RecoveryDispositionReceipt` is exactly 320 bytes:

<!-- markdownlint-disable MD013 -->

| Offset | Width | Field | Canonical value |
| ---: | ---: | --- | --- |
| 0 | 16 | magic | `KEEP:REC:DISP2\0\0` |
| 16 | 2 | version | `2` |
| 18 | 2 | record length | `320` |
| 20 | 4 | flags | `0` |
| 24 | 2 | artifact kind | registered enum |
| 26 | 2 | decision | finalize or retire |
| 28 | 2 | admitted recovery classification | registered enum |
| 30 | 2 | reserved | zero |
| 32 | 8 | artifact length | exact observed length |
| 40 | 32 | artifact identity digest | physical evidence identity |
| 72 | 32 | artifact content digest | exact verified bytes |
| 104 | 8 | publication-head generation | exact observed value |
| 112 | 32 | publication-head checksum | exact observed value |
| 144 | 8 | catalog generation | exact observed value |
| 152 | 32 | catalog digest | exact observed value |
| 184 | 8 | liveness generation | exact observed value |
| 192 | 32 | retention-manifest digest | exact observed value |
| 224 | 8 | reader-lock device identity | exact safety coordinate |
| 232 | 8 | reader-lock mount identity | exact safety coordinate |
| 240 | 8 | reader-lock file identity | exact safety coordinate |
| 248 | 32 | decision-evidence digest | complete canonical proof |
| 280 | 8 | reserved | zero |
| 288 | 32 | checksum | BLAKE3-256 over bytes `0..288` |

<!-- markdownlint-enable MD013 -->

The checksum domain is `keep.recovery-disposition-receipt-checksum/v2\0`.
The artifact content digest is BLAKE3-256 of the artifact's exact bytes under
`keep.recovery-disposition-artifact/v2\0`; the artifact identity digest is
the artifact's pool-name digest. The three enumerations are registered in
`definition.tsv` and any other code refuses:

<!-- markdownlint-disable MD013 -->

| Field | Registered values |
| --- | --- |
| artifact kind | `segment:1`, `catalog:2`, `retention-root:3`, `retention-manifest:4`, `retention-head:5` |
| decision | `finalize:1`, `retire:2` |
| classification | `complete-orphan:1`, `complete-stage:2`, `stale-generation:3` |

<!-- markdownlint-enable MD013 -->

A `complete-orphan` is a complete, verified artifact linked into its pool
that no head, catalog, or manifest names; a `complete-stage` is a complete,
verified fixed stage not yet linked; a `stale-generation` is a complete
artifact whose generation a later publication superseded before it became
visible. Every generation field must be positive.

The pool coordinate is:

```text
recovery/dispositions/<artifact-identity-digest-64-lower-hex>.receipt
```

The version-2 maximum is 65,536 disposition receipts. A future successor must
migrate the namespace before raising the ceiling. The protocol that writes a
receipt for a recovery-protected retention orphan is
[explicit disposition](recovery.md#explicit-disposition-of-protected-orphans);
liveness generation zero beside the initial retention-state digest records a
decision made while no retention head was published.

## Planning

`plan_gc(&GcLivenessSnapshot, GcLimits)` is the pure, deterministic
comparison ADR-0009 requires between one immutable liveness snapshot and one
bounded physical inventory. It reads nothing and writes nothing.
`observe_gc_liveness` assembles the snapshot from a fenced
`FilesystemRetentionSnapshot`: it re-admits the fenced catalog, projects every
retained root's verified closure onto the segments that hold its records,
reads and admits every entry of the segment pool within the
`CatalogRestartPolicy` byte bound, walks the catalog predecessor chain to
find segments a durably published successor superseded, and admits every
exact disposition receipt. Each released segment carries the digest of the
record that released it: the predecessor catalog or the receipt.

The plan classifies every inventoried segment exactly once, in this order:

<!-- markdownlint-disable MD013 -->

| Classification | Meaning | Candidate |
| --- | --- | --- |
| `live` | the current catalog names it and at least one retained closure reaches it | no |
| `named-unreachable` | the current catalog names it and no retained closure reaches it; only a compaction successor can release it | no |
| `recovery-protected` | no catalog in the chain names it and no disposition retires it: an orphan of an interrupted publication | no |
| `unreachable-superseded` | a predecessor catalog named it and the current catalog omits it | yes |
| `unreachable-disposed` | a durable disposition receipt retired it | yes |

<!-- markdownlint-enable MD013 -->

A superseded or disposed segment absent from the inventory is reported as
already retired. Any contradiction refuses the whole plan as a typed
`GcPlanAmbiguity`: a named segment absent from the inventory, a closure
member the catalog does not name or the inventory lacks, or a superseded or
disposed segment the current catalog still names. More candidates than
`GcLimits` admit refuse rather than truncate. Reader protection is not a
planning classification: execution takes writer authority and the exclusive
reader lock and re-proves every coordinate the plan names before acting.

The plan for the frozen version-2 store is
[`gc-plan.tsv`](../../../conformance/segment-store/v2/gc-plan.tsv); the
planner laws in `src/adapters/gc/planner_tests.rs` cover every
classification, every ambiguity, the limit, the golden plan, and a
512-universe model in which the live set is always exactly the union of the
retained closures and no live or named segment is ever a candidate.

> **Warning.** Execution unlinks immutable segments. It is specified and
> proven on [GC execution and recovery](gc-execution.md): writer authority,
> the exclusive reader lock, a durable intent before the first unlink, a
> re-proven plan, and a recovery report afterwards. Recovery admits one
> canonical absent candidate prefix and treats every other residue as
> unrecoverable ambiguity. `plan_gc` is the dry run.

## Disposition transition

A disposition transition writes and synchronizes
`recovery/disposition.next`, verifies and links the immutable receipt without
replacement, synchronizes `recovery/dispositions`, removes the stage, and
synchronizes `recovery`. Until that completes, the artifact remains
recovery-protected.

All three grammars have golden fixtures, parsers, corruption matrices, and a
seeded fuzz target; the planner has its golden plan and model law; retirement
and disposition have their in-process prefix laws and the process-death
matrix; compaction has its identity-stability and interruption laws.
Compaction benchmarks are **Planned in #21**.
