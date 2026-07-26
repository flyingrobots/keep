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

## Calculation boundary

`ChunkId::hash_bytes` calculates the identity produced by the supplied bytes.
`FastCdc` calculates identities for the bytes observed while detecting spans.
Neither operation compares against an independently supplied expected identity
or proves:

- that a span belongs to a validated layout;
- that the complete blob identity matches;
- that bytes are stored, retained, durable, or recoverable;
- that a later read still returns the same bytes.

Integrity verification requires recomputing an identity and comparing it with
an independently obtained expected `ChunkId`. The remaining claims require
future layout, storage, retention, and verification evidence.

## Failure transition

`FastCdc::feed` processes incrementally. A typed failure may occur after a
prefix of the feed was accepted and callbacks for that prefix ran. The error
reports that prefix length, and the detector records the original failure.
Later `feed` and `finish` calls return the same error without accepting or
emitting anything; the caller must discard the failed detector.

A callback panic is outside this typed transition because Keep cannot observe
caller unwinding. A caller that catches such a panic must also discard the
detector.

## Bounds and allocation

`ChunkId::hash_bytes` borrows one nonempty slice, reads it once, and allocates
no heap memory. It refuses lengths that do not fit `u32`.

`FastCdc` borrows feed slices and retains no candidate bytes. Its state is one
BLAKE3 hasher, a Gear fingerprint, typed stream coordinates, fixed counters,
and an optional typed failure. `FastCdc::RETAINED_STATE_LIMIT_BYTES` bounds
that inline state at 4 KiB, independent of the total input length. Caller-owned
input and callback memory are outside that bound.

The integration suite separately checks the inline type-size ceiling and uses
an instrumented allocator in an isolated test binary. The allocator observes
zero total, live, and peak heap allocation while `FastCdc` processes 16 KiB,
1 MiB, and 4 MiB deterministic inputs. This bounds detector-owned working
memory without treating process RSS, caller input, or callback output as
detector state.

## Performance evidence

`FastCdc` evaluates the registered scalar boundary rule one byte at a time but
batches BLAKE3 updates over contiguous feed ranges. The
`streaming_cdc` benchmark records whole-slice and one-byte-feed throughput for
minimum and one-mebibyte inputs:

```bash
cargo bench --bench streaming_cdc
```

## Executable evidence

- `conformance/chunk-id/v1/` independently checks three identity vectors.
- `conformance/cdc-profile/v1/` freezes every expected boundary.
- `tests/streaming_cdc.rs` checks golden boundaries, exact identities,
  reconstruction, partition invariance, profile bounds, adversarial inputs,
  and the inline state bound.
- `tests/streaming_cdc_memory.rs` measures zero detector-owned heap allocation
  over increasing representative input lengths.
- `fuzz/fuzz_targets/fast_cdc.rs` compares whole, bytewise, and irregular
  partitions, fails on every unexpected refusal, and rehashes every emitted
  span.
- `benches/streaming_cdc.rs` provides repeatable whole-feed and bytewise-feed
  throughput evidence.

The choice of identity grammar and public surface is recorded in
[the colocated rationale](rationale.md).
