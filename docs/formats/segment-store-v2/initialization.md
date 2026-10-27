# Direct Initialization

This page owns how a `keep.segment-store/v2` store comes into existence. It
adds no byte format: `FORMAT` keeps the grammar in
[Format marker](recovery.md#format-marker), and every other record keeps the
grammar of its own page. [ADR-0010](https://github.com/flyingrobots/keep/pull/199)
records the decision that initialization replaces migration as the only entry
into the store format.

## Committed namespace

An initialized store root holds exactly:

```text
writer.lock
reader.lock
FORMAT
staging/
segments/
catalogs/
retention/roots/
retention/manifests/
gc/
recovery/dispositions/
```

Catalog `HEAD` is absent until the first catalog is published. Each directory
is empty, and `writer.lock` and `reader.lock` are zero-length regular files.
Nothing in the store records the root directory's device, inode, mount, or
path, so a quiescent copy of a committed store is the same store; see
[Portable stores](recovery.md#portable-stores).

A committed store with no catalog `HEAD` is a defined state, not an inferred
one. Writer reopen admits it. The first catalog publication proves the
absence instead of assuming it: it refuses unless `segments` and `catalogs`
are empty, and the retained-closure gate refuses unless `retention/roots` and
`retention/manifests` are empty, as
[catalog publication](catalog-publication.md#forward-protocol) specifies. A
reader refuses it with `FilesystemRetentionSnapshotError::Unpublished` after
admission and the reader fence.

## Forward protocol

`FilesystemVersionTwoAdmission::initialize` runs under exclusive writer
authority. It performs these phases in order:

| Phase | Operation | Durable effect |
| --- | --- | --- |
| `AdmitPlatform` | open the root no-follow and admit the platform profile | none |
| `ObservePrefix` | classify the root without mutation | none |
| `OpenAndLockWriterFile` | create or open `writer.lock`, lock it, synchronize it and the root | `writer.lock` |
| `DiscardIncompleteMarkerStage` | only on resume: remove an incomplete `FORMAT.next`, synchronize the root | stage removed |
| `WriteMarkerStage` | exclusively create `FORMAT.next` and write the 96 canonical bytes | stage bytes |
| `SynchronizeMarkerStage` | synchronize `FORMAT.next` and reverify its bytes and identity | durable stage |
| `SynchronizeRootAfterMarkerStage` | synchronize the root | durable stage name |
| `AdmitReaderFence` | create zero-length `reader.lock`, synchronize the root | `reader.lock` |
| `AdmitDirectory` | create each directory below in order, synchronize its parent | one directory |
| `LinkMarker` | hard-link `FORMAT.next` to `FORMAT` without replacement | `FORMAT` |
| `SynchronizeRootAfterMarker` | synchronize the root | **commit point** |
| `RemoveMarkerStage` | remove `FORMAT.next` | stage removed |
| `SynchronizeRootAfterCleanup` | synchronize the root | durable cleanup |

The directory order is `staging`, `segments`, `catalogs`, `retention`,
`retention/roots`, `retention/manifests`, `gc`, `recovery`, and
`recovery/dispositions`. A directory that already exists is admitted only if it
is a real directory that holds nothing but the canonical children already
created before it. Every creation is followed by a synchronization of its
parent, so each name is durable before the next one is created.

Initialization never writes `FORMAT` in place. `FORMAT.next` is written
before any directory exists and made durable first.
It is the evidence that this root is a Keep initialization in progress. A root
of directories without it is never adopted.

### Commit point

The commit point is the root synchronization that follows `LinkMarker`.
Before it, the store does not admit. Neither the existence of `FORMAT` nor the
presence of the protocol directories proves completion: `FORMAT` is linked
while `FORMAT.next` still names the same inode, and every admission refuses a
root that holds `FORMAT.next`. `FORMAT.next` is removed only after the commit
synchronization returned, so a root holding `FORMAT` without `FORMAT.next` can
be observed only after the commit is durable. Initialization returns writer
authority only after `SynchronizeRootAfterCleanup`.

## States and results

Initialization classifies the root before it creates `writer.lock`, and again
after it holds the lock. Only the locked classification authorizes effects;
the first exists so that a refused directory is never mutated, not even by
creating `writer.lock`. Every state has exactly one result for each entry
point:

| State | Root holds | `initialize` | writer and reader admission |
| --- | --- | --- | --- |
| empty | nothing | create from the start | refuse: no `writer.lock` |
| locked | `writer.lock` | resume at `WriteMarkerStage` | refuse `MissingFormatMarker` |
| incomplete marker stage | + a strict canonical prefix of `FORMAT.next` | discard it, then write it | refuse `InitializationIncomplete` |
| marker staged | + canonical `FORMAT.next` | resume at `SynchronizeMarkerStage` | refuse `InitializationIncomplete` |
| reader fence | + `reader.lock` | resume at `SynchronizeMarkerStage` | refuse `InitializationIncomplete` |
| directory prefix *k* | + the first *k* directories | resume at `SynchronizeMarkerStage` | refuse `InitializationIncomplete` |
| marker linked | + `FORMAT` naming the inode of `FORMAT.next` | resume at `SynchronizeRootAfterMarker` | refuse `InitializationIncomplete` |
| committed | `FORMAT` without `FORMAT.next` | refuse `AlreadyInitialized` | admit |
| anything else | | refuse, no mutation | refuse |

Every state that holds the canonical `FORMAT.next` and no `FORMAT` resumes at
`SynchronizeMarkerStage` and replays the later phases. Each replayed phase is
idempotent: the stage is reopened and verified by its exact bytes and inode,
an existing `reader.lock` or directory is admitted by kind, length, and
membership and its parent is synchronized again, and the link never replaces
an entry. Replay therefore never depends on knowing which synchronization
completed before process death. A synchronization failure returns
`VersionTwoInitializationError::Io` at that phase and leaves the state on the
table, so a retry resumes and repeats the synchronization.

Refusals carry their evidence as `VersionTwoInitializationRefusal`:

- an entry outside the initialization namespace, a directory or `reader.lock`
  without `FORMAT.next`, a directory out of order, or an entry inside a
  protocol directory other than its canonical children is `NotAPrefix`,
  naming the entry by its path;
- a canonical entry of the wrong kind, such as a link where a directory
  belongs, is `WrongKind`, with expected and observed kinds;
- a `FORMAT.next` whose bytes are not a prefix of the canonical marker, or
  that is longer than 96 bytes, is `MarkerStageMismatch`, with expected and
  observed lengths;
- a `FORMAT` beside `FORMAT.next` that is not the same inode with the same
  canonical bytes is `MarkerLinkMismatch`;
- a `writer.lock` or `reader.lock` that is not zero-length is
  `NonEmptyLockFile`, with its observed length; and
- `FORMAT` without `FORMAT.next` is `AlreadyInitialized`.

The marker-linked state admits a populated committed namespace, not only an
empty one. Process death cannot produce one, because admission refuses
`FORMAT.next`. A filesystem that loses the stage removal at power failure
could, after later writes made other entries durable. Removing a second name
for the committed marker's inode loses nothing, so initialization completes the
cleanup instead of refusing a committed store forever.

### Retention absence

The initialization prefixes require `retention/roots` and
`retention/manifests` to be empty and `retention` to hold nothing else. Missing
retention state therefore never means "nothing is retained" when an artifact
contradicts it: a stray root, manifest, head, or stage refuses as
`NotAPrefix`. After commit, retention publication and the
retained-closure gate own that law.

### Concurrent attempts

Writer authority is the root directory lock and then the `writer.lock` lock,
both nonblocking. Of two concurrent attempts, one acquires them. The other
refuses with `WriterLock { source: Busy }`, or, if it acquires them after the
first committed, with `AlreadyInitialized`. Both classifications run under the
lock, so the second attempt never resumes the first one's phases.

### Roots of the retired version 1

Version-1 admission and initialization are deleted
([#198](https://github.com/flyingrobots/keep/issues/198)). A former version-1
root (`writer.lock`, `staging`, `segments`, `catalogs`, and `HEAD`) is not an
initialization prefix, so `initialize` refuses it unchanged and every open
path refuses it as `MissingFormatMarker`. A root holding only an empty
`writer.lock` is the `Locked` prefix of this protocol and resumes.

## Process-death matrix

`cargo xtask durability-crash-matrix --sequence version-two-initialization`
runs the production initializer in a subprocess and kills it before, during,
and after each boundary. The parent then checks the exact inventory and bytes,
the admission refusal or result, and runs `initialize` again as recovery. It
then requires writer reopen, publishes a first catalog through the version-2
writer, and reads the blob back.

| ID | Boundary | During |
| --- | --- | --- |
| `KEEP-CRASH-081` | `OpenAndLockWriterFile` | after the lock |
| `KEEP-CRASH-082` | `WriteMarkerStage` | a 48-byte strict prefix |
| `KEEP-CRASH-083` | `SynchronizeMarkerStage` | before |
| `KEEP-CRASH-084` | `SynchronizeRootAfterMarkerStage` | before |
| `KEEP-CRASH-085` | `AdmitReaderFence` | after |
| `KEEP-CRASH-086` | `AdmitDirectory` | after each of the nine directories |
| `KEEP-CRASH-087` | `LinkMarker` | after |
| `KEEP-CRASH-088` | `SynchronizeRootAfterMarker` (commit point) | before |
| `KEEP-CRASH-089` | `RemoveMarkerStage` | after |
| `KEEP-CRASH-090` | `SynchronizeRootAfterCleanup` | before |
| `KEEP-CRASH-091` | `DiscardIncompleteMarkerStage`, inside recovery | after |

The matrix has 41 cases. It is process-death evidence. It does not claim
power-loss behavior, which depends on the filesystem honoring each
synchronization.
