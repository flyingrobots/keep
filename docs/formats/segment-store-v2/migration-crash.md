# Migration Crash Points

This page owns fixed-record publication and process-death boundaries for the
one-way `keep.segment-store/v1` to `keep.segment-store/v2` migration.

## Fixed-stage law

Migration never writes canonical fixed names in place:

| Stage | Canonical target |
| --- | --- |
| `migration.intent.next` | `migration.intent` |
| `FORMAT.next` | `FORMAT` |
| `migration.receipt.next` | `migration.receipt` |

For each pair, migration:

1. creates the stage exclusively as a pinned regular file;
2. writes bounded complete bytes, synchronizes, reopens, and verifies them;
3. links the stage to the canonical target without replacement;
4. synchronizes the store root;
5. removes the retained stage; and
6. synchronizes the store root again.

The verified stage is linked without replacement. The canonical target is
immutable. Recovery never truncates, replaces, or repairs it. An exact stage
with an absent target resumes at the link. Exact stage and target bytes resume
at the required synchronization or cleanup. Different bytes, a substituted
inode, a link, or a wrong file kind refuse.

A pre-effect incomplete stage may be removed only when its canonical target and
every later-ordered migration effect are absent and every earlier effect admits
exactly. Recovery pins the stage, removes it, synchronizes the store root, and
returns a typed discard report. Any later effect makes incomplete or corrupt
stage bytes unrecoverable ambiguity.

The fixed stage is not authority. `migration.intent` becomes migration
authority only after its canonical link and store-root synchronization.
`migration.receipt` becomes completion evidence at the equivalent boundary.

## Receipt synchronization mask

`migration.receipt` records the exact pre-receipt mask
`0x00000000000003ff`. Bits are:

| Bit | Completed evidence |
| ---: | --- |
| 0 | canonical migration intent and store root synchronized |
| 1 | `reader.lock` verified, synchronized, and root-synchronized |
| 2 | `retention` created and parent synchronized |
| 3 | `retention/roots` created and parent synchronized |
| 4 | `retention/manifests` created and parent synchronized |
| 5 | `gc` created and parent synchronized |
| 6 | `recovery` created and parent synchronized |
| 7 | `recovery/dispositions` created and parent synchronized |
| 8 | canonical format marker and store root synchronized |
| 9 | complete version-2 view reopened and verified |

Bits 10 through 63 are zero and refuse when set. The mask records only
evidence completed before receipt construction; receipt-stage publication and
its final root synchronizations are established by admission of the canonical
receipt, not claimed by its own bytes.

## Namespace prefix

After durable intent publication, migration creates persistent `reader.lock`
and the exact nested directory prefix in the order specified by
[migration recovery](recovery.md). Each existing name must be the exact pinned
file or directory expected at that position. Each new nested name is followed
by synchronization of its parent. The final store-root synchronization admits
the complete prefix. A wrong kind, link, out-of-order name, or unknown entry
refuses.

## Process-death matrix

| Identifier | Boundary |
| --- | --- |
| `KEEP-CRASH-053` | migration-intent stage write |
| `KEEP-CRASH-054` | migration-intent stage synchronization |
| `KEEP-CRASH-055` | migration-intent canonical link |
| `KEEP-CRASH-056` | store-root synchronization after intent link |
| `KEEP-CRASH-057` | migration-intent stage removal |
| `KEEP-CRASH-058` | store-root synchronization after intent cleanup |
| `KEEP-CRASH-059` | persistent reader-fence creation |
| `KEEP-CRASH-060` | canonical nested directory-prefix creation |
| `KEEP-CRASH-061` | store-root synchronization after namespace creation |
| `KEEP-CRASH-062` | format-marker stage write |
| `KEEP-CRASH-063` | format-marker stage synchronization |
| `KEEP-CRASH-064` | format-marker canonical link |
| `KEEP-CRASH-065` | store-root synchronization after marker link |
| `KEEP-CRASH-066` | format-marker stage removal |
| `KEEP-CRASH-067` | store-root synchronization after marker cleanup |
| `KEEP-CRASH-068` | migration-receipt stage write |
| `KEEP-CRASH-069` | migration-receipt stage synchronization |
| `KEEP-CRASH-070` | migration-receipt canonical link |
| `KEEP-CRASH-071` | store-root synchronization after receipt link |
| `KEEP-CRASH-072` | migration-receipt stage removal |
| `KEEP-CRASH-073` | final store-root synchronization |

Every identifier requires before, during, and after process-death evidence.
`KEEP-CRASH-060` additionally requires one case for every admitted directory
prefix length. Restart must classify exact stages, canonical targets, namespace
prefix, marker, receipt, and cleanup state without depending on a clock,
filesystem iteration order, or file existence alone.

`StoreMigrationPhase::ALL` freezes the 21 boundaries above in exact order.
Fresh writer-locked filesystem execution implements that exact order and has
deterministic in-process storage-fault and corruption laws. The restart
classifier and resuming storage are implemented and proven in-process for
every prefix of the 21 phases (see
[partial migration recovery](migration-recovery.md)).

The before, during, and after process-death matrix runs as
`cargo xtask durability-crash-matrix --sequence migration`. For each of the
68 cases (21 boundaries at three positions, plus one `during` case per
admitted directory-prefix length for `KEEP-CRASH-060`) an isolated child
process publishes the Golden File Worldline version-1 store, executes the
production migration protocol with the selected boundary gated, and is killed
by its process group. The parent then compares the exact root inventory
against an independent expected-state model, reopens the root for recovery
the way a restarted writer would, requires the production planner to report
the plan the
[recovery table](migration-recovery.md#partial-migration-recovery) predicts,
runs that recovery (or the forward retry after an untouched version-1 store
admits), and requires one complete migration with every version-1 byte intact
and a second recovery that reports `Complete`. The
[transitions ledger](../../../conformance/segment-store/v2/transitions.tsv)
records each boundary's pre-state, interrupted class, post-state, and recovery
posture. The matrix proves application process death; host power loss
remains outside its claim.
