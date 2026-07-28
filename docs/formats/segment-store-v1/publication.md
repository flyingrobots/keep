# Publication and Reader Visibility

This page owns the physical namespace, visibility states, writer exclusion,
forward publication order, and reader snapshots for
`keep.segment-store/v1`.

## Physical namespace

The version-1 filesystem adapter owns this relative namespace:

```text
writer.lock
HEAD
head.next
staging/current.seg
staging/current.cat
segments/<segment-digest>.seg
catalogs/<generation>-<catalog-digest>.cat
```

Digest components are exactly 64 lowercase hexadecimal characters.
`generation` is exactly 16 lowercase hexadecimal digits. All operations are
capability-relative and refuse symlinks, nonregular files, alternate
spellings, unknown entries in protocol-owned directories, and replacement of
the opened store root.

The filesystem must expose case-sensitive, byte-preserving directory names.
The capability probe refuses case-folding or normalization aliases before a
store root is initialized or admitted.

The names are physical adapter coordinates, not stable public handles.
`writer.lock` persists; its existence and contents prove nothing.

The writer never overwrites an immutable-pool name. It atomically hard-links
one fully synchronized staged artifact into the same-filesystem pool. If the
destination already exists, the link operation leaves it unchanged. The
result is not trusted from the link outcome or destination name.

After either a new link or an existing-name result, the writer
reopens the pool entry without following links and verifies it completely.
It compares the pool entry against the pre-link verified bytes and digest.
Only that post-link verification advances the protocol. A mismatch is
unrecoverable ambiguity and never an idempotent-success receipt.

Before that link, the writer closes every writable staging handle and reopens
the synchronized artifact read-only for complete verification. No writable
handle or writable protocol path remains once the immutable-pool link exists.

An uninitialized root has no admitted generation. Initialization and the first
publication use generation 1 with an all-zero predecessor digest. A missing
head in a root containing protocol artifacts requires recovery; it is never
silently interpreted as an empty store.

## Crash-safe initialization

Before mutation, the initializer admits the opened root only when the
capability probe passes and its protocol namespace is either empty or a
partial canonical initialization set containing only `writer.lock`,
`staging`, `segments`, and `catalogs`. Every existing name must have the
required regular-file or directory kind and must pass no-follow admission.
Any artifact, unknown name, wrong kind, or alias is unrecoverable ambiguity.

The initializer creates `writer.lock` exclusively when absent, or reopens the
existing regular file without following links (`KEEP-CRASH-031`). Its contents
prove nothing. The initializer acquires the exclusive advisory lock before
creating any directory. Under that lock, it creates each absent directory
exclusively in this order:

1. `staging` (`KEEP-CRASH-032`);
2. `segments` (`KEEP-CRASH-033`); and
3. `catalogs` (`KEEP-CRASH-034`).

An exact existing directory is idempotent success for its creation step. Each
retry reopens and verifies every existing canonical name. A crash before the
final synchronization leaves a recoverable partial initialization set; the
next initializer reacquires the lock, verifies the observed subset, and
continues without deleting or renaming it.

After all four canonical names exist, the initializer synchronizes the store
root (`KEEP-CRASH-035`). Only the completed root synchronization admits an
uninitialized store and returns an initialization receipt. No reader treats a
partial set or an unsynchronized complete set as an admitted store.

## State and visibility

<!-- markdownlint-disable MD013 -->

| State | Physical evidence | Visible to a new current reader | Law |
| --- | --- | --- | --- |
| Uninitialized | no admitted head | no | not an empty published generation |
| Reusable stage | exact header and complete unsealed records | no | may be resumed only by explicit recovery |
| Truncated tail | incomplete or ambiguous staged framing | no | preserve and refuse until explicitly discarded |
| Sealed stage | complete verified seal in staging | no | immutable bytes, not yet durable pool evidence |
| Valid orphan | verified immutable artifact not selected by current head | no | preserve; neither promote nor delete silently |
| Published | selected through one verified head and catalog | yes | exact immutable reader snapshot |
| Retired | selected by an older catalog but not the current head | no for new readers | existing pinned readers may finish; no deletion in v1 |
| Corrupt sealed | seal, checksum, digest, or framing disagreement | no | refuse; never truncate or reinterpret |
| Stale generation | candidate does not extend current generation exactly | no | refuse expected/observed generation |
| Unrecoverable ambiguity | conflicting or insufficient durable evidence | no | represent uncertainty and refuse |

<!-- markdownlint-enable MD013 -->

## Writer exclusion

Version 1 is a one-writer/many-reader protocol.

The writer opens `writer.lock` without following links and acquires one
exclusive kernel advisory lock before recovery planning. It holds the lock
through publication success or typed failure.

Readers do not take the writer lock. They rely on immutable segments,
immutable catalogs, and atomic head replacement.

A lock acquisition failure reports writer busy. The writer never deletes,
renames, truncates, or replaces the lock file to break a purported stale
owner. Process death releases the kernel lock. Filesystems without proven
process-scoped exclusion are unsupported.

## Forward publication protocol

The writer starts with an expected current generation and catalog digest. It
acquires the lock, validates the current head and catalog again, and refuses
stale expectations before creating a stage. If the current verified head
already equals the complete proposed generation, catalog length, and catalog
digest, retry returns an explicit already-published receipt after
synchronizing the root directory. A different observed generation or digest
is a stale-generation refusal.

Every write handles short writes and interruption. Every flush, file sync,
hard link, unlink, head replacement, and directory sync is explicit and
fallible. An error returns no publication receipt. `Drop` performs cleanup
only and cannot publish.

A crash during directory synchronization may expose either the namespace
state that preceded the synchronization or the fully synchronized state.
Recovery classifies and verifies both possibilities; it never infers that the
preceding namespace mutation became durable merely because it was issued.

### Seal each new segment

1. Create `staging/current.seg` exclusively
   (`KEEP-CRASH-001`).
2. Write the complete 64-byte header (`KEEP-CRASH-002`).
3. Append each complete record and checksum (`KEEP-CRASH-003`, with an
   occurrence counter for tests).
4. Flush the complete record prefix (`KEEP-CRASH-004`).
5. Synchronize the reusable record prefix (`KEEP-CRASH-005`).
6. Append the complete seal (`KEEP-CRASH-006`).
7. Flush the sealed bytes (`KEEP-CRASH-007`).
8. Synchronize the sealed staging file (`KEEP-CRASH-008`).
9. Reopen and verify the complete staged segment, then atomically hard-link it
   without replacement to the exact digest-derived immutable-pool name
   (`KEEP-CRASH-009`). Reopen the resolved pool entry and complete the required
   post-link verification.
10. Synchronize `segments` (`KEEP-CRASH-010`).
11. Unlink `staging/current.seg` (`KEEP-CRASH-011`).
12. Synchronize `staging` (`KEEP-CRASH-012`).

After step 12 the segment is a durable valid orphan. It remains invisible
until a published catalog names it.

### Publish the catalog generation

1. Create `staging/current.cat` exclusively
   (`KEEP-CRASH-013`).
2. Write the complete canonical generation (`KEEP-CRASH-014`).
3. Flush it (`KEEP-CRASH-015`).
4. Synchronize it (`KEEP-CRASH-016`).
5. Reopen and verify it, then atomically hard-link it without replacement to
   the exact generation-and-digest immutable-pool name
   (`KEEP-CRASH-017`). Reopen the resolved pool entry and complete the required
   post-link verification.
6. Synchronize `catalogs` (`KEEP-CRASH-018`).
7. Unlink `staging/current.cat` (`KEEP-CRASH-019`).
8. Synchronize `staging` (`KEEP-CRASH-020`).

After step 8 the catalog is a durable valid orphan. It remains invisible
until the publication head names it.

### Replace the publication head

1. Create `head.next` exclusively (`KEEP-CRASH-021`).
2. Write the complete 128-byte head (`KEEP-CRASH-022`).
3. Flush it (`KEEP-CRASH-023`).
4. Synchronize it (`KEEP-CRASH-024`).
5. Reopen and verify the head and its complete transitive catalog view, then
   atomically replace `HEAD` with `head.next` (`KEEP-CRASH-025`).
6. Synchronize the store root (`KEEP-CRASH-026`).

Only completion of step 6 returns a `#[must_use]` publication receipt.

An existing `head.next` always routes through recovery before step 1. The
writer never truncates, replaces, or silently removes it to make the exclusive
create succeed.

The normative pre-state, interrupted-state class, post-state, and recovery
posture for every identifier are frozen in
[`transitions.tsv`](../../../conformance/segment-store/v1/transitions.tsv).

## Reader snapshot

A new reader:

1. opens `HEAD` without following links and reads exactly 128 bytes;
2. validates its complete framing and checksum;
3. opens the exact digest-derived catalog name;
4. verifies catalog length, checksum, digest, generation, predecessor
   coordinate, ordering, and duplicates;
5. verifies every referenced segment and record before admission; and
6. retains that immutable catalog generation for the snapshot lifetime.

The reader never rescans `HEAD` during one operation and never combines
catalogs. If head replacement races the initial read, atomic replacement gives
the reader either the complete old head or complete new head. Any other
observation is an unsupported-platform or corruption refusal.

Version 1 never deletes immutable artifacts. A reader holding an older
verified generation can therefore finish while a new generation is
published. Retention and GC must define later deletion safety before this can
change.
