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

## Canonical catalog ordering

Catalog entries sort by record kind and then by the exact meaningful identity
bytes. Chunk and layout identities remain different types even if their
digests happen to match. Duplicate logical keys are refused rather than
resolved by "last write wins."

The catalog includes record offsets, lengths, payload lengths, and record
checksums. Those values let a reader reject an entry that does not name one
complete record before it materializes a payload. The enclosing segment
digest separately binds the immutable file.

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

## Flush, sync, link, unlink, rename, and directory sync

`flush` moves language or library buffers toward the operating system. File
synchronization asks the platform to make file bytes and required metadata
durable. Rename changes directory entries atomically. Directory
synchronization makes that namespace change durable. None substitutes for
another.

Every immutable artifact is synchronized before it is hard-linked into its
pool. Every writable staging handle is closed first, and complete verification
uses a new read-only handle. Hard-link creation is atomic and no-clobber: it
cannot overwrite an existing digest-derived name. The pool directory is
synchronized before the staging link is removed, and the staging directory is
synchronized after that removal. A crash between link and unlink leaves two
names for the same verified immutable bytes, which recovery can classify
without guessing.

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

## Observation before recovery

Store opening performs no repair. It produces either one verified reader
snapshot, a typed recovery inventory and plan, or a refusal. This separation
prevents a read-only process from mutating evidence and lets tests inject
crashes into recovery itself.

Reusable staging, valid orphan, truncated tail, corrupt seal, stale
generation, and ambiguity remain distinct because they authorize different
future actions. In particular, a valid orphan is preserved but invisible; it
is not silently promoted or deleted.

## Deferred capabilities

Version 1 does not define retention roots, deletion, garbage collection,
compaction, encryption, compression, multiple writers, network filesystems,
Windows durability, background scheduling, or application namespaces.
Adding any of those requires evidence that preserves this protocol's exact
identity, visibility, and refusal laws.
