# GC and Disposition Records

This page owns the canonical planned `GcRetirementIntent`,
`GcRetirementReceipt`, and `RecoveryDispositionReceipt` byte grammars for
`keep.segment-store/v2`.

Issue #21 owns their implementation. They are specified now so version 2 has
one exact root grammar, but their presence remains unsupported mandatory state
until every **Planned in #21** requirement becomes executable evidence.

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
Unknown artifact kinds, decisions, or classifications refuse.

The pool coordinate is:

```text
recovery/dispositions/<artifact-identity-digest-64-lower-hex>.receipt
```

The version-2 maximum is 65,536 disposition receipts. A future successor must
migrate the namespace before raising the ceiling.

## State and recovery

GC admits these states:

<!-- markdownlint-disable MD013 -->

| State | Evidence | Recovery |
| --- | --- | --- |
| idle | no `gc/intent` or `gc/receipt` | no retirement authority |
| active | exact intent, every candidate present | begin execution |
| partial | exact intent, one canonical absent candidate prefix | continue at first present candidate |
| completion pending | exact intent, every candidate absent | publish receipt |
| receipt transition | exact intent and exact receipt | synchronize receipt, remove intent, synchronize `gc` |
| complete | exact receipt only | return exact completion |

<!-- markdownlint-enable MD013 -->

An absent candidate outside the canonical absent candidate prefix, substituted
candidate, changed pool, stale coordinate, conflicting receipt, malformed
record, or unexplained absence is unrecoverable ambiguity. Recovery never
guesses which deletion occurred.

A disposition transition writes and synchronizes
`recovery/disposition.next`, verifies and links the immutable receipt without
replacement, synchronizes `recovery/dispositions`, removes the stage, and
synchronizes `recovery`. Until that completes, the artifact remains
recovery-protected.

These grammars, their golden fixtures, parsers, corruption matrices, crash
points, model, benchmarks, and fuzz targets are **Planned in #21**. Issue #19
must refuse their physical presence without mutating it.
