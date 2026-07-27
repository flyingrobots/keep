# Flat Chunk Layout Version 1 Rationale

## Decision

Keep assigns layout codec `1`, named `keep.flat-chunks/v1`, to a fixed-width
big-endian flat sequence of explicitly offset version-1 `ChunkId` values.
The record embeds the complete canonical binary target `BlobId`, binds one
registered `StorageProfileId`, carries a domain-separated checksum, and
limits the entry count to `2^20`.

This decision governs the canonical record, `LayoutId`, validation stages,
resource bounds, migration posture, and first durable layout compatibility
surface. The production codec implements those laws. Ingestion, storage, and
verified reconstruction remain separate future boundaries.

## Why a flat first codec

A flat ordered plan has one bounded traversal, one offset domain, and no
recursive trust transition. That makes gaps, overlaps, aggregate mismatch,
allocation, and range planning directly auditable.

The `2^20` entry limit bounds the largest record at 46,137,520 bytes and the
registered profile's largest describable blob at 256 GiB. That is a deliberate
version-1 protocol limit, not a claim that every caller should materialize a
46 MiB plan.

Issue #12 will measure real workloads. A hierarchical codec requires its own
depth, fanout, cycle, aggregate, and allocation laws and therefore receives a
new codec coordinate rather than an inactive flag in v1.

## Why explicit offsets

Offsets are derivable from preceding lengths, but encoding them creates a
cross-field invariant that detects gaps, overlaps, reordering, and incorrect
range planning before chunk bytes are loaded.

The redundancy is canonical: exactly one offset is lawful at each entry.
Physical locations remain absent. The offset is a logical coordinate within
the target blob.

## Why fixed-width binary

Fixed-width big-endian integers have one encoding, make bounds inspectable
before allocation, and require no serializer configuration. A positional
record cannot contain duplicate map keys or alternate field order.

The format still rejects inserted duplicate field bytes, trailing bytes, and
changed fixed lengths. A serializer-owned Rust struct, arbitrary Serde output,
JSON, CBOR, and host-native layout are not durable protocol.

## Why bind identities by kind

The full canonical binary `BlobId` appears in the header because it already
has an admitted boundary encoding and includes the logical length.

The profile coordinate uses its identity version, hash algorithm, and digest
instead of duplicating the 96-byte profile record in every layout. Admission
resolves the identity through the registered profile set.

The header binds every entry to version-1 BLAKE3-256 `ChunkId`. An entry then
stores the exact length and digest that form that typed value. Repeating the
chunk magic, version, and algorithm in every entry would add bytes without
adding a new validation boundary.

Verified reconstruction replays the registered profile's deterministic
boundary detector over the reconstructed stream and compares its emitted
spans with the entries. Chunk and blob digest agreement alone cannot prove
that the declared profile produced the plan.

Any future identity kind, version, or algorithm requires a new layout codec.
No version-1 field is reinterpreted.

## Why include a record checksum

`LayoutId` verifies a plan when an independently obtained expected identity is
available. Recovery and inspection may encounter an isolated candidate record
before a catalog supplies that expectation.

The record checksum provides a typed corruption check in that state. Its
domain differs from `LayoutId`, and it covers the header and entries but not
itself. It is not authentication and cannot make an untrusted record
authoritative.

## Why the checksum is inside `LayoutId`

The exact canonical plan includes its checksum. This prevents two records
with different checksum bytes from sharing one `LayoutId`, keeps the durable
record self-contained, and makes checksum-algorithm evolution an explicit
codec change.

## Alternatives rejected

### Encode only ordered lengths and digests

Rejected because gaps, overlaps, and entry reordering would be
unrepresentable rather than independently checkable. Explicit offsets create
the required continuity law and support later exact range planning.

### Embed physical segment coordinates

Rejected because compaction, copying, tier movement, and catalog recovery move
locations without moving logical content or its reconstruction plan.
ADR-0002 assigns physical coordinates to generation-checked catalogs.

### Embed the complete storage-profile record

Rejected because the immutable registered profile already has a typed
identity. Repeating the record increases every layout and creates two copies
whose disagreement would need another precedence rule.

### Use CBOR, JSON, or arbitrary Serde output

Rejected because serializer defaults, map order, duplicate fields, integer
width choices, and library upgrades create a larger canonicality surface.
Keep reserves JSON and CBOR for boundaries with named canonical profiles; this
format needs neither.

### Use variable-width integers

Rejected because fixed-width fields are small relative to chunk digests and
have a one-byte representation. Varints add overlong encodings and parsing
branches without meaningful layout-size savings.

### Omit the checksum and rely only on `LayoutId`

Rejected because recovery may need to classify an isolated record before an
expected identity is available. The checksum supplies corruption evidence
without claiming catalog membership or writer authority.

### Make version 1 hierarchical

Rejected because recursive layouts require explicit depth, fanout, cycle,
aggregate, partial-read, and allocation laws. Adding unused child fields or
flags would create dormant ambiguity. A future hierarchy receives a new codec
and fixtures.

### Remove the entry-count limit and stream indefinitely

Rejected because an unbounded durable record is an unbounded validation and
indexing obligation even when decoding streams. The hard limit makes refusal
portable and reviewable; configured policies may be lower.

### Use one chunk for the empty blob

Rejected because `ChunkId` names only nonempty bytes and FastCDC emits zero
chunks for empty input. The unique empty layout therefore contains zero
entries and the empty `BlobId`.

## Consequences

- A layout is independently reproducible from exact typed inputs.
- Rechunking can move `LayoutId` without moving `BlobId`.
- Range planning can validate logical continuity before storage lookup.
- Verified reconstruction refuses a content-correct plan that falsely claims
  a storage profile whose deterministic boundaries it does not reproduce.
- Large plans remain bounded but may still warrant streaming APIs.
- Blobs above the flat codec's capacity are precisely unsupported rather than
  partially represented.
- Format evolution creates a new codec and migration path instead of changing
  version-1 meaning.
