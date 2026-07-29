# Durable Segment Store Version 1 Rationale

This note records the format-local choices behind
`keep.segment-store/v1`. ADR-0005 owns the cross-cutting decision to bind
format, publication, crash states, and recovery into one protocol.

## Three artifacts instead of one mutable file

Segments own immutable record bytes. Catalog generations own the current map
from logical identity to physical evidence. The publication head owns the
single mutable choice of current generation.

Keeping those concerns separate makes the smallest mutable durable artifact
128 bytes. Readers can verify and retain one immutable catalog while a writer
constructs the next generation. Later compaction can move physical records by
publishing a new catalog without changing logical identity or rewriting old
segments in place.

## Fixed widths and big-endian integers

Every field has one offset and one width. Positional records make duplicate
fields unrepresentable, avoid serializer defaults, and permit bounds to be
checked before allocation. Big-endian fixed-width integers match Keep's
existing identity and flat-layout protocols and give byte ordering the same
direction as numeric ordering where a field is compared lexically.

## Typed records inside bounded segments

One file per chunk would turn the directory into an implicit catalog and
require one durable directory entry for every chunk. One monolithic log would
leave all historic evidence coupled to one mutable tail. Bounded immutable
segments amortize synchronization while preserving finite verification,
recovery, retirement, and compaction units.

Version 1 admits only chunk and flat-layout records because those are the two
logical storage records already specified and implemented. The record header
has no generic extension payload. An unknown kind or nonzero flag is
mandatory-to-understand and refused.

The 60-byte identity slot fits the existing canonical binary `LayoutId`.
Chunk records use the first 36 bytes for their checked length and digest and
require the remaining bytes to be zero. A fixed slot keeps every record
header the same size without making chunk and layout identities
interchangeable.

## Record checksums and sealed-segment digests

Each record checksum localizes corruption and binds the typed identity header
to the exact payload. A segment digest binds the complete header, every
record and record checksum, and the nonrecursive prefix of the seal. The seal
checksum then protects the complete seal including that digest.

The digest is a physical coordinate, not logical content identity. Two
segments may lawfully store the same logical records in different record
orders and therefore have different segment digests. Catalog entries retain
logical identity separately.

The seal includes redundant lengths and counts so a decoder can compare
declared, calculated, and actual framing before trusting offsets. Redundancy is
checked evidence, not permission to select whichever value looks plausible.

Complete-segment admission also refuses a repeated logical identity inside one
segment. Even byte-identical duplicates would create multiple physical
locations for one catalog key and make later location selection needlessly
ambiguous. The reader records one identity coordinate per policy-admitted
record, sorts that temporary bounded index only to detect duplicates, and
retains the segment's physical record order for iteration.

Seal admission is deliberately phased for readers. Fixed seal coordinates and
the seal checksum are established before its count bounds the record walk.
Every record checksum and logical identity is then admitted before the
physical segment digest. The admitted segment is published to the caller only
after all phases succeed, while ordinary record corruption still reports the
more local record boundary instead of collapsing into a digest-only refusal.

## Canonical catalog ordering

Catalog entries sort by record kind and then by the exact meaningful identity
bytes. Chunk and layout identities remain different types even if their
digests happen to match. Duplicate logical keys are refused rather than
resolved by "last write wins."

The catalog includes record offsets, lengths, payload lengths, and record
checksums. Those values let a reader reject an entry that does not name one
complete record before it materializes a payload. The enclosing segment
digest separately binds the immutable file. Admission first scans the complete
segment grammar and records its top-level spans, so record-shaped bytes inside
a payload cannot become a second physical record merely because a catalog
points at them.

Catalog generations form a checked predecessor chain. Generation 1 uses an
all-zero predecessor digest; every later generation increments by exactly one
and embeds the previous catalog digest. This does not create application
history or retention authority. It only prevents a writer from publishing
over a stale physical generation without noticing.

## Fixed-name staging

One writer permits one fixed segment stage, one fixed catalog stage, and one
fixed next-head file. Fixed names avoid clocks, random numbers, process IDs,
and collision retry policy in the durable protocol. Recovery must classify or
explicitly dispose of an existing stage before another transaction begins.

The immutable pool names include verified digests. Naming is an adapter
detail, but its canonical lowercase grammar makes recovery deterministic and
prevents multiple spellings of one physical artifact.

That grammar requires case-sensitive, byte-preserving directory names.
Case-folding or normalization aliases would let a noncanonical spelling reach
canonical bytes before recovery could refuse the path, so version 1 rejects
such filesystems instead of weakening path identity.

## Flush, sync, link, unlink, rename, and directory sync

`flush` moves language or library buffers toward the operating system. File
synchronization asks the platform to make file bytes and required metadata
durable. Rename changes directory entries atomically. Directory
synchronization makes that namespace change durable. None substitutes for
another.

Every immutable artifact is synchronized before it is hard-linked into its
pool. Every writable staging handle is closed first, and complete verification
uses a new read-only handle. Hard-link creation is atomic and no-clobber: it
cannot overwrite an existing digest-derived name. Because path resolution can
race the pre-link read, the resolved pool entry is reopened and completely
verified after both new-link and existing-name outcomes. Only after that check
is the pool directory synchronized; the staging link is then removed, and the
staging directory is synchronized after that removal. A crash between link and
unlink leaves two names for the same verified immutable bytes, which recovery
can classify without guessing.

The publication head is the only replaced protocol file. It is replaced only
after every referenced immutable artifact is durable. Success is returned
only after the root-directory sync following head replacement.

## Persistent advisory lock

The writer lock file persists and its contents carry no authority. Successful
exclusive kernel lock acquisition is the only ownership evidence. Process
death releases the lock without requiring presence-based stale-file cleanup.

This deliberately excludes multi-host and filesystems whose advisory locks do
not provide the required exclusion. Weakening the lock would violate
one-writer publication rather than improve availability.

## Platform admission before publication

The writer lock proves process-scoped exclusion; it does not prove
case-sensitive names, single-host ownership, hard-link and replacement
semantics, or directory durability. `FilesystemCatalogPublisher::open`
therefore consumes an opaque `FilesystemPlatformAdmission` that owns the
acquired lock. The proof is bound to that exact root authority and cannot be
constructed from metadata or caller assertion.

Issue #16 exposes no public proof producer. Its crate-internal transition tests
use an explicitly test-only unchecked value, while issue #17 owns the
crash-tested initializer and platform checks that may return production
admission. This staging prevents an incomplete probe from turning successful
syscalls into a durability claim.

Rejected alternatives were treating `FilesystemWriterLock` as sufficient,
accepting a caller-selected boolean or platform enum, and approving any
filesystem whose individual operations returned success. Each would let an
unsupported platform manufacture the authority that the proof is meant to
represent.

## Observation before recovery

Store opening performs no repair. It produces either one verified reader
snapshot, a typed recovery inventory and plan, or a refusal. This separation
prevents a read-only process from mutating evidence and lets tests inject
crashes into recovery itself.

Reusable staging, valid orphan, truncated tail, corrupt seal, stale
generation, and ambiguity remain distinct because they authorize different
future actions. In particular, a valid orphan is preserved but invisible; it
is not silently promoted or deleted.

Explicit discard fingerprints are computed only after the canonical stage
name selects its format maximum and a limit-plus-one stream proves the complete
observed bytes fit. Oversized evidence is preserved and refused before hashing
rather than turned into unbounded recovery work.

## Deferred capabilities

Version 1 does not define retention roots, deletion, garbage collection,
compaction, encryption, compression, multiple writers, network filesystems,
Windows durability, background scheduling, or application namespaces.
Adding any of those requires evidence that preserves this protocol's exact
identity, visibility, and refusal laws.
