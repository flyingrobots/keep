# Migration and Recovery

This page owns the version-2 filesystem namespace, format marker, reader fence,
one-way migration, fixed-stage recovery, GC reservation, and
recovery-disposition reservation.

## Exact filesystem namespace

Version 2 preserves the version-1 files and directories and admits these new
coordinates:

```text
reader.lock
FORMAT
migration.intent
migration.intent.next
migration.receipt
migration.receipt.next
FORMAT.next
retention/HEAD
retention/head.next
retention/root.next
retention/manifest.next
retention/roots/<namespace-digest>/<generation>-<root-digest>.root
retention/manifests/<generation>-<manifest-digest>.manifest
gc/intent
gc/receipt
recovery/disposition.next
recovery/dispositions/<artifact-digest>.receipt
```

`retention/HEAD`, every fixed `.next` stage, `gc/intent`, and `gc/receipt` are
optional according to the exact state tables below. Immutable-pool coordinates
are data-dependent but canonically named. Every other root or
protocol-directory entry is an unknown entry and unrecoverable ambiguity.
Operations are capability-relative and never follow links.

## Format marker

`FORMAT` is exactly 96 bytes:

| Offset | Width | Field | Canonical value |
| ---: | ---: | --- | --- |
| 0 | 16 | magic | `KEEP:STORE:V2\0\0\0` |
| 16 | 2 | version | `2` |
| 18 | 2 | record length | `96` |
| 20 | 4 | flags | `0` |
| 24 | 32 | format-definition digest | registered v2 digest |
| 56 | 4 | maximum namespace count | `4,096` |
| 60 | 4 | reserved | zero |
| 64 | 32 | checksum | BLAKE3-256 over bytes `0..64` |

The definition and checksum domains are
`keep.segment-store-definition/v2\0` and
`keep.segment-store-marker-checksum/v2\0`. A missing marker is version 1 only
when the exact version-1 namespace admits. An unsupported, corrupt,
substituted, or same-name/different-digest marker refuses.

The format-definition digest is BLAKE3-256 of its domain followed by the exact
corpus `definition.tsv` bytes. The format-marker digest is BLAKE3-256 of
`keep.store-format-marker/v2\0` followed by all 96 marker bytes.

`CanonicalStoreFormatMarker` produces the registered marker; `AdmittedStoreFormatMarker` admits its framing, checksum, definition, and namespace bound.
`AdmittedStoreMigrationIntent` admits the exact intent framing, checksum, catalog coordinates, predecessor law, definition, and derived store identity.
These record boundaries do not prove the named live inventory or physical root,
detect the store version, or execute filesystem migration.

## Reader fence

`reader.lock` is a persistent regular zero-length file. Its contents and
existence alone prove nothing.

A version-2 reader acquires a kernel-managed shared lock on `reader.lock`
before opening catalog `HEAD` or `retention/HEAD`. The returned `ReaderFence`
owns that lock for the complete snapshot lifetime. Close, drop, or process
death releases only the kernel lock and never deletes the persistent file.

GC acquires the store writer authority and then an exclusive `reader.lock`, in
that fixed order. New readers wait and existing readers drain before GC
revalidation or physical deletion. Catalog and retention publication may
proceed beside readers because they publish immutable successors and delete no
published segment.

## Migration records

Migration is a one-way explicit migration under exclusive writer authority.
Version 1 is never extended in place without durable migration evidence.

`migration.intent` is exactly 256 bytes:

<!-- markdownlint-disable MD013 -->

| Offset | Width | Field | Canonical value |
| ---: | ---: | --- | --- |
| 0 | 16 | magic | `KEEP:MIG:INT2\0\0\0` |
| 16 | 2 | version | `2` |
| 18 | 2 | record length | `256` |
| 20 | 4 | flags | `0` |
| 24 | 8 | catalog generation named by version-1 `HEAD` | positive |
| 32 | 8 | catalog length named by version-1 `HEAD` | exact admitted length |
| 40 | 32 | catalog digest named by version-1 `HEAD` | exact admitted digest |
| 72 | 32 | predecessor catalog digest | zero for generation 1 |
| 104 | 32 | immutable-pool inventory digest | canonical complete inventory |
| 136 | 8 | root device identity | admitted platform value |
| 144 | 8 | root mount identity | admitted platform value |
| 152 | 8 | root file identity | admitted platform value |
| 160 | 32 | target format-definition digest | exact registered v2 digest |
| 192 | 32 | new store identifier | deterministic derivation below |
| 224 | 32 | checksum | BLAKE3-256 over bytes `0..224` |

<!-- markdownlint-enable MD013 -->

The checksum domain is `keep.store-migration-intent-checksum/v2\0`. The
receipt's intent digest is BLAKE3-256 of
`keep.store-migration-intent/v2\0` followed by all 256 intent bytes.

The [migration inventory](migration-inventory.md) defines its domain and law:
each migration inventory entry is exactly 56 bytes, and the fixed maximum is
2,097,152 entries. The intent therefore binds the exact catalog generation,
length, and digest named by the admitted version-1 `HEAD`.

The deterministically derived store identifier is:

```text
BLAKE3-256("keep.store-identifier/v2\0" ||
           catalog-generation-u64 ||
           catalog-length-u64 ||
           catalog-digest ||
           predecessor-catalog-digest ||
           immutable-pool-inventory-digest ||
           target-format-definition-digest)
```

Integer fields use their fixed-width big-endian bytes. Root device, mount, file
identity, caller identity, path, and time do not enter the identifier. The
migration intent separately binds the physical root coordinates so in-place
recovery refuses a substituted store.

`migration.receipt` is exactly 256 bytes:

<!-- markdownlint-disable MD013 -->

| Offset | Width | Field | Canonical value |
| ---: | ---: | --- | --- |
| 0 | 16 | magic | `KEEP:MIG:REC2\0\0\0` |
| 16 | 2 | version | `2` |
| 18 | 2 | record length | `256` |
| 20 | 4 | flags | `0` |
| 24 | 32 | migration-intent digest | exact durable intent |
| 56 | 32 | store identifier | exact intent value |
| 88 | 32 | format-marker digest | exact verified marker |
| 120 | 32 | initial retention-state digest | exact no-payload digest below |
| 152 | 32 | initial GC-state digest | exact no-payload digest below |
| 184 | 32 | disposition namespace digest | exact no-payload digest below |
| 216 | 8 | completed synchronization mask | every mandatory bit set |
| 224 | 32 | checksum | BLAKE3-256 over bytes `0..224` |

<!-- markdownlint-enable MD013 -->

Its checksum domain is `keep.store-migration-receipt-checksum/v2\0`. Unknown
synchronization bits, a missing mandatory bit, or any mismatch with the intent
refuses.

The three initial-state fields are the no-payload digests
`BLAKE3-256("keep.initial-retention-state/v2\0")`,
`BLAKE3-256("keep.initial-gc-state/v2\0")`, and
`BLAKE3-256("keep.empty-disposition-set/v2\0")`. At completed migration,
absence of `retention/HEAD` is the canonical empty retention state only while
all retention stages and pools are empty. Any retention artifact routes through
recovery instead. Direct version-2 initialization is undefined.

The byte-exact offset tables and golden fixtures are requirements
`KEEP-MIGRATION-002` and `KEEP-MIGRATION-007`; no production writer exists
until those planned items become implemented evidence.

## One-way migration protocol

Migration performs these ordered steps:

1. Admit and completely recover the exact version-1 store.
2. Revalidate its head, catalog, pools, root identity, and writer authority.
3. Publish `migration.intent` from `migration.intent.next` through the
   no-replacement fixed-stage protocol.
4. Create and verify persistent `reader.lock`.
5. Create the exact `retention`, `retention/roots`,
   `retention/manifests`, `gc`, `recovery`, and
   `recovery/dispositions` directories.
6. Synchronize every created parent and the store root.
7. Publish `FORMAT` from `FORMAT.next` through the fixed-stage protocol.
8. Reopen and verify the complete version-2 view.
9. Publish `migration.receipt` from `migration.receipt.next` through the
   fixed-stage protocol.

The [migration crash-point specification](migration-crash.md) owns that
protocol and spans `KEEP-CRASH-053` through `KEEP-CRASH-073`.

Migration never rewrites or deletes admitted version-1 immutable bytes and
provides no automatic downgrade.

Version-1 admission refuses once any migration stage, `migration.intent`,
`reader.lock`, `FORMAT`, or version-2 directory is present. Once the canonical
intent is durable, only version-2 migration recovery may continue.

## Partial migration recovery

The migration recovery boundary admits only these ordered prefixes:

<!-- markdownlint-disable MD013 -->

| State | Required response |
| --- | --- |
| no migration artifact | admit exact version 1 |
| intent stage only | finalize an exact stage or explicitly discard an incomplete pre-effect stage |
| durable intent only | verify intent and continue |
| intent plus a canonical prefix of v2 names | verify each name and continue |
| complete v2 shape without marker | verify directories and write marker |
| marker without receipt | reopen full v2 view and publish receipt |
| exact receipt with optional exact receipt stage | clean the stage and admit complete migration |

<!-- markdownlint-enable MD013 -->

A partial migration retry revalidates the intent and every existing byte,
continues idempotently at the first absent canonical step, and never replaces
an existing entry. A missing predecessor, changed version-1 coordinate,
out-of-order name, wrong file kind, substituted byte, conflicting receipt,
unknown entry, or changed root identity is unrecoverable ambiguity.

Process death before durable canonical intent leaves version 1 plus at most its
non-authoritative stage. Process death after durable intent leaves
recovery-required version-2 migration state.

## Retention publication recovery

At restart, a fixed retention stage is classified from its exact framing and
transitive evidence:

The forward protocol guarantees that `root.next` is durable before a new
namespace directory is created. A new digest-named directory is created
exclusively, verified as the exact regular directory rather than a link, and
followed by synchronization of `retention/roots` before the immutable root is
linked. An existing exact directory is idempotent; any wrong kind, substituted
namespace, or unexpected entry refuses. Directory existence alone never proves
a retained root.

<!-- markdownlint-disable MD013 -->

| Fixed stage | Complete evidence | Recovery |
| --- | --- | --- |
| `root.next` | canonical successor root, matching namespace and closure proof | finalize its immutable pool link and retain the stage |
| `manifest.next` | canonical successor manifest naming only admitted roots | finalize its immutable pool link and retain both stages |
| `head.next` | canonical successor head naming the staged manifest | finalize the head, synchronize it, then remove retained stages |

<!-- markdownlint-enable MD013 -->

A pre-effect incomplete stage may be removed only when every later-ordered
effect is absent and all earlier evidence admits exactly. Recovery pins that
regular file, removes it, synchronizes `retention`, and returns a typed discard
report. Any later effect, stale generation, mismatched digest, missing
transitive member, reappeared stage, conflicting pool entry, or other
corruption is a typed refusal. A complete valid orphan remains
recovery-protected until explicit disposition.

The retention crash points are:

| Identifier | Boundary |
| --- | --- |
| `KEEP-CRASH-036` | root stage write |
| `KEEP-CRASH-037` | root stage synchronization |
| `KEEP-CRASH-038` | new namespace-directory creation or exact admission |
| `KEEP-CRASH-039` | namespace-pool synchronization after creation |
| `KEEP-CRASH-040` | immutable root link |
| `KEEP-CRASH-041` | root namespace-directory synchronization |
| `KEEP-CRASH-042` | manifest stage write |
| `KEEP-CRASH-043` | manifest stage synchronization |
| `KEEP-CRASH-044` | immutable manifest link |
| `KEEP-CRASH-045` | manifest pool synchronization |
| `KEEP-CRASH-046` | retention-head stage write |
| `KEEP-CRASH-047` | retention-head stage synchronization |
| `KEEP-CRASH-048` | retention-head atomic replacement |
| `KEEP-CRASH-049` | committed retention namespace synchronization |
| `KEEP-CRASH-050` | retained root-stage removal |
| `KEEP-CRASH-051` | retained manifest-stage removal |
| `KEEP-CRASH-052` | retention cleanup synchronization |

`RetentionPublicationPhase::ALL` freezes this exact order as a typed public
vocabulary. Storage execution and process-death evidence remain unimplemented.

Each point requires before, during, and after process-death evidence. Restart
must establish exact catalog visibility, retention head, namespace generation,
orphan classification, stage disposition, and recovery report.

## GC and recovery-disposition recovery

The [GC and disposition record specification](gc.md) owns the exact
`GcRetirementIntent`, `GcRetirementReceipt`, and
`RecoveryDispositionReceipt` grammars and state transitions. Issue #21 owns
their executable parser, corruption, crash, recovery, and fuzz evidence.
Issue #19 admits only the absent `gc/intent`, `gc/receipt`,
`recovery/disposition.next`, and disposition-receipt pool. Any presence is
unsupported mandatory state and refuses without mutation.
