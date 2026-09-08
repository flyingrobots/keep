# Retention Publication

This page owns closure admission and the generation transition for
`keep.segment-store/v2`. The record grammars it publishes are owned by
[Retention records](retention.md); restart behaviour for a retained stage is
owned by [Migration and recovery](recovery.md).

## Closure admission

Before publication, Keep pins one completely verified catalog generation and
applies the exact deterministic traversal, counter units, failure order,
authenticated reconstruction, and canonical digest defined by
[Closure verification](closure.md). Any closure failure refuses the entire
transition. Keep never omits one failed member and continues with a smaller
live set.

Preflight verifies steps 3 and 4 without I/O; preparation binds that proof to
the current manifest and derives exact canonical successors.
`execute_retention_publication` revalidates current authority, executes all 17
ordered durability phases, and returns the complete receipt only after cleanup;
exact already-committed retry revalidates authority and performs no mutation.

Version-2 catalog publication holds the same writer authority and proves every
current retained closure against its candidate catalog before replacing the
catalog `HEAD`.

## Generation transition

A transition supplies a namespace, an expected state of absent or one exact
`RootGeneration`, a complete canonical anchor set, the exact realization
profile coordinate, and admitted limits.

Under exclusive writer authority, publication:

1. completes recovery of every fixed retention stage;
2. admits the current retention head, manifest, and selected namespace root;
3. compares expected and observed generations;
4. verifies the candidate closure against one pinned catalog;
5. writes and synchronizes `retention/root.next`;
6. for a new namespace, exclusively creates and verifies its exact digest-named
   directory, then synchronizes `retention/roots`;
7. links and verifies the root pool entry, then synchronizes its directory;
8. writes and synchronizes `retention/manifest.next`;
9. links and verifies the manifest pool entry and synchronizes its directory;
10. writes and synchronizes `retention/head.next`;
11. atomically replaces `retention/HEAD` and synchronizes `retention`;
    `root.next` and `manifest.next` remain durable until the retention head
    commits, then are removed and `retention` is synchronized again; and
12. returns a consequential `#[must_use]` receipt.

The receipt binds the namespace, expected and observed generations, committed
root generation and digest, global manifest generation and digest, profile
coordinate, anchor-set and closure digests, catalog generation and digest, and
every durable publication outcome.

A stale transition preserves expected and observed generations. A
byte-identical retry returns **already committed** only while that exact root
successor remains current; otherwise it returns the precise stale state.

A reader holds one shared `ReaderFence` and double-collects the catalog and
retention heads around complete transitive admission. It accepts only the same
coordinates before and after for both heads. Any generation, length, digest, or
checksum change discards the view and retries within a bounded attempt limit;
exhaustion refuses. The accepted view observes one complete root generation for
its snapshot lifetime.
