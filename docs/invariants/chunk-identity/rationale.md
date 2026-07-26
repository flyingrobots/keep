# Chunk Identity Rationale

## Decision

Keep defines `ChunkId` version 1 as a private-field value containing a
positive `u32` byte length and BLAKE3-256 over the typed
`KEEP:CHUNK:DATA` preimage documented in [the invariant](README.md).

The identity excludes CDC profile, blob identity, stream offset, layout,
representation, storage location, and retention state. This decision governs
the chunk-identity invariant and the initial public `ChunkId` API.

The first public surface is deliberately small:

- `ChunkId::hash_bytes` calculates identity from one borrowed slice;
- `ChunkId::length` exposes the typed committed length;
- no raw-digest constructor or accessor is public;
- no text or binary codec is defined before the layout boundary needs one.

`FastCdc` calculates the same identity incrementally while detecting spans.
It emits `ChunkSpan` values through a synchronous callback so Keep does not
allocate a result collection proportional to blob length.

The detector hashes contiguous feed ranges rather than calling BLAKE3 once per
byte. A typed detector failure is terminal: the detector retains the original
error, and both later `feed` calls and the consuming `finish` call return it.
This makes partial progress visible without permitting an ambiguous retry.

## Alternatives rejected

### Reuse `BlobId`

Rejected because a chunk is a physical-layout coordinate, not a logical blob.
Separate Rust types and hash domains prevent substitution even when one blob
happens to contain exactly one chunk with the same bytes.

### Hash raw chunk bytes

Rejected because raw `BLAKE3(C)` does not commit to Keep's identity kind,
version, algorithm coordinate, or canonical length rule.

### Include the CDC profile

Rejected because the same exact chunk bytes should retain one `ChunkId`
regardless of which admitted profile found them. The future layout binds the
ordered spans and `StorageProfileId`.

### Include blob identity or stream offset

Rejected because identical physical chunks must remain reusable across blobs
and positions. Blob membership and order belong to the layout.

### Use a `u64` chunk length

Rejected for version 1 because admitted profile lengths are `u32`, and
`ChunkId::hash_bytes` materializes a single borrowed slice. A wider field
would expand the durable identity grammar without an admitted use.

### Publish a codec now

Rejected because issue #9 owns the canonical layout format and issue #10 owns
its boundary codec. Defining an isolated `ChunkId` wire format first could
force redundant framing or conflict with the layout's canonical grammar.

### Return a collected vector from `FastCdc::feed`

Rejected because a caller could feed an entire blob and force hidden memory
growth proportional to its chunk count. Callback emission keeps Keep's
working state constant while allowing the caller to choose collection,
streaming, or future storage behavior explicitly.

## Consequences

`ChunkId` is a compatibility commitment even though the crate is unreleased.
Changing its domain, field widths, suffix order, or BLAKE3 algorithm requires
a new identity version.

The callback API does not yet provide a fallible storage sink. A future
ingestion port may add that boundary without changing identity or boundary
semantics. This slice makes no durability, recovery, retention, or throughput
claim.
