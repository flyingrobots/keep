# Chunk Identity

This page defines what Keep currently proves when it returns a `ChunkId`.

## Invariant

`ChunkId` names one exact, finite, nonempty chunk. For chunk bytes `C` with
length `N`, where `1 <= N <= 2^32 - 1`, version 1 calculates:

```text
chunk_digest_v1(C) = BLAKE3-256(
    ASCII("KEEP:CHUNK:DATA\0")
    || u16be(1)
    || u8(1)
    || C
    || u32be(N)
)
```

The 16-byte data magic, version, algorithm, bytes, and checked length all
participate. The `KEEP:CHUNK:DATA` domain is distinct from the
`KEEP:BLOB:DATA` domain, so chunk and blob identities cannot be substituted.

A validated `ChunkId` contains the admitted `ChunkLength` and 32-byte digest.
Its fields are private. `ChunkId::hash_bytes` is the only public constructor
in this slice.

## Relationship to chunking

Chunk identity is independent of the boundary profile. `FastCdc` uses the
accepted `fastcdc-64k-v1` profile to find spans, then calculates `ChunkId`
from the exact bytes in each span.

Changing a profile may move chunk boundaries and therefore produce a
different ordered set of chunk identities. It cannot move `BlobId` when the
complete logical bytes remain unchanged.

## Verification boundary

`ChunkId::hash_bytes` verifies only that the supplied bytes produce the
returned identity. `FastCdc` verifies the bytes observed while detecting a
span. Neither operation proves:

- that a span belongs to a validated layout;
- that the complete blob identity matches;
- that bytes are stored, retained, durable, or recoverable;
- that a later read still returns the same bytes.

Those claims require future layout, storage, retention, and verification
evidence.

## Bounds and allocation

`ChunkId::hash_bytes` borrows one nonempty slice, reads it once, and allocates
no heap memory. It refuses lengths that do not fit `u32`.

`FastCdc` borrows feed slices and retains no candidate bytes. Its state is one
BLAKE3 hasher, a Gear fingerprint, typed stream coordinates, and fixed
counters. `FastCdc::RETAINED_STATE_LIMIT_BYTES` bounds that state at 4 KiB,
independent of the total input length. Caller-owned input and callback memory
are outside that bound.

The integration suite measures `size_of::<FastCdc>()`, processes the
one-mebibyte zero witness, and proves the retained state does not grow. This
is deterministic retained-memory evidence, not a process-RSS claim.

## Executable evidence

- `conformance/chunk-id/v1/` independently checks three identity vectors.
- `conformance/cdc-profile/v1/` freezes every expected boundary.
- `tests/streaming_cdc.rs` checks golden boundaries, exact identities,
  reconstruction, partition invariance, profile bounds, adversarial inputs,
  and retained memory.
- `fuzz/fuzz_targets/fast_cdc.rs` compares whole, bytewise, and irregular
  partitions and rehashes every emitted span.

The choice of identity grammar and public surface is recorded in
[the colocated rationale](rationale.md).
