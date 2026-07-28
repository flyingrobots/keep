# ADR-0005: Durable Segment Store Protocol

- Status: Accepted
- Date: 2026-07-28
- Owners: Keep segment, catalog, publication, and recovery boundaries
- Related issue:
  [#14](https://github.com/flyingrobots/keep/issues/14)
- Depends on: ADR-0002, ADR-0004, issues #10 and #13

## Context

Keep's first two milestones prove exact logical identity, canonical chunking
and layout, bounded streaming ingestion, authenticated reconstruction, and
exact range reads. Those laws do not establish durable storage. A process can
currently prove which bytes it observed without proving what survived a
crash, what a new process may safely admit, or which physical records form one
published view.

Durability cannot be added as a final `fsync` call around an otherwise private
file layout. The bytes, forward write protocol, crash states, reader
visibility, and recovery behavior constrain one another. Treating any one of
them separately would permit states whose safety cannot be proved:

- a complete-looking file that was never durably published;
- a catalog that names a segment whose directory entry did not survive;
- a new head that names a partial or noncanonical catalog;
- a staging tail mistaken for a sealed segment;
- a stale lock file mistaken for a live writer;
- a valid orphan silently promoted into the current generation; or
- a reader combining entries from different catalog generations.

Keep therefore needs one versioned physical protocol before the production
segment writer, catalog publisher, and recovery engine can be implemented.

## Decision

Keep adopts `keep.segment-store/v1`, specified in the
[version-1 format and state protocol](../formats/segment-store-v1/README.md).
The decision is one inseparable contract with three parts:

1. canonical immutable segment, catalog-generation, and publication-head
   bytes;
2. an explicit write, flush, sync, no-clobber link, unlink, head-replacement,
   and directory-sync protocol; and
3. deterministic classification and recovery of every interrupted state.

A version-1 segment contains a fixed header, zero or more complete typed
records, and a fixed seal. A record stores either exact chunk bytes bound to a
version-1 `ChunkId` or one canonical flat-layout record bound to its
`LayoutId`. Every record has a domain-separated checksum. The segment seal
binds its complete prefix to a domain-separated physical segment digest and
seal checksum. A sealed segment is immutable.

A catalog generation is a canonical, sorted, duplicate-free map from logical
record identity to a physical segment digest, record offset, record length,
payload length, and record checksum. Locations are physical evidence, not
public stable identity. A fixed publication head names exactly one catalog
generation, catalog length, and catalog digest.

Publication is successful only after all referenced sealed segments and the
new catalog have been written, verified, hard-linked without replacement into
immutable pools, and made durable through directory synchronization. The
staging links are then removed durably; the new head is written,
synchronized, atomically replaced, and followed by root-directory
synchronization. `flush` and file synchronization are separate required
steps. Publication is an explicit fallible operation and never occurs from
`Drop`.

Version 1 uses one writer and any number of readers. The writer holds one
kernel-managed exclusive advisory lock across recovery planning and the
complete publication transaction. Lock-file existence has no meaning; the
lock file is persistent, and process death releases the kernel lock. A held
lock is never stolen or deleted as "stale."

A reader validates one head, opens the exact catalog it names, verifies the
complete catalog, and retains that immutable catalog generation for the
reader's lifetime. Concurrent publication cannot change that view. Files that
are staged, sealed but unpublished, orphaned, or retired from the current
generation are not discoverable through a normal current-generation read.

Opening the store is observational. It may return a typed recovery plan but
must not mutate, truncate, delete, promote, or rewrite physical state.
Executing recovery is explicit and fallible. Recovery preserves valid orphans,
refuses corrupt sealed state, and represents ambiguity rather than guessing.
The required classifications are:

- reusable staged material;
- valid orphan;
- truncated tail;
- corrupt sealed state;
- stale generation; and
- unrecoverable ambiguity.

The initial durable adapter is supported only where its capability probe can
establish regular no-follow files; case-sensitive, byte-preserving directory
names; same-filesystem atomic replacement; atomic no-clobber hard-link
creation; durable file and directory synchronization; and process-scoped
kernel advisory locking. Unknown, networked, path-aliasing, or otherwise
weaker filesystems are refused. This ADR makes no unsupported power-loss or
hardware-cache claim.

The exact grammars, bounds, checksum preimages, physical naming rules,
visibility states, crash points, recovery table, and platform contract are
normative in the version-1 specification. Golden bytes and transition
evidence live in the
[implementation-independent corpus](../../conformance/segment-store/v1/README.md).

## Alternatives considered

- **One file per chunk.** Rejected because file existence cannot establish
  record completeness, verification, retention, or publication. It also
  multiplies directory operations and sync costs, makes atomic multi-record
  publication unavailable, and turns filesystem enumeration into an
  accidental catalog.
- **An embedded database as the durable protocol.** Rejected for version 1
  because database pages, journals, checkpoints, locking, and corruption
  behavior would become Keep's actual on-disk contract without exposing the
  exact crash and recovery state machine Keep must audit. A future adapter may
  use a database only behind the same semantic ports and with equivalent
  evidence.
- **Git objects, trees, and refs.** Rejected because Git representation,
  process behavior, and ref semantics are not Keep logical identity or
  retention law. Git also imports application and transport policy into a
  lower storage layer.
- **One monolithic append log.** Rejected because an ever-growing mutable file
  couples old readable evidence to new tail writes, complicates bounded
  recovery, prevents immutable retirement units, and makes compaction and
  catalog publication harder to reason about.
- **Presence-based stale-lock deletion.** Rejected because a present path does
  not prove a live owner and deleting a lock file can create two writers.
  Version 1 trusts only successful kernel lock acquisition.
- **Automatic repair while opening.** Rejected because choosing, truncating,
  promoting, or deleting durable evidence during observation can hide
  ambiguity. Recovery is planned, explicit, and separately committed.
- **Readers following the newest files they can enumerate.** Rejected because
  enumeration order and file existence do not define one atomic generation.
  Readers follow one verified head to one verified immutable catalog.

## Consequences

- Issues #15, #16, and #17 must implement this protocol rather than inventing
  independent segment, catalog, or crash behavior.
- Version-1 byte grammars, checksum domains, bounds, sort order, crash-point
  identifiers, and publication order are compatibility commitments. Changing
  them requires a new protocol version.
- Segment and catalog adapters must parse, validate, then admit untrusted
  bytes. Production domain and port APIs must not expose serializer-owned
  values or physical paths as identity.
- Sealed segments and published catalog generations are immutable. Later
  compaction publishes new physical locations without changing `BlobId`,
  `ChunkId`, or `LayoutId`.
- The fixed-name staging protocol is intentionally one-writer. Multi-writer,
  distributed locking, and network filesystems require another decision.
- No version-1 garbage collector may delete a valid orphan or retired segment
  until retention, immutable liveness snapshots, reader safety, and recovery
  are specified by later milestones.
- The golden corpus freezes designed protocol bytes; it does not claim that a
  production writer, reader, catalog, or recovery engine has shipped.
