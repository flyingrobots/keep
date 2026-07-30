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
