# Chunk Identity Corpus v1

This directory is the implementation-independent executable corpus for
`ChunkId` version 1. It freezes physical chunk identity without defining a
layout, storage location, retention claim, or chunk-boundary profile.

Run the independent checker with:

```bash
python3 conformance/chunk-id/v1/check_vectors.py
```

The checker requires Python 3.10 or newer and `b3sum` 1.8.5 or a compatible
implementation on `PATH`.

## Identity

For one nonempty chunk `C` of length `N`, where
`1 <= N <= 2^32 - 1`, version 1 defines:

```text
chunk_digest_v1(C) = BLAKE3-256(
    ASCII("KEEP:CHUNK:DATA\0")
    || u16be(1)
    || u8(1)
    || C
    || u32be(N)
)
```

The data magic is exactly 16 bytes. Version `1` and algorithm `1`
(`BLAKE3-256`) are explicit. The fixed-width length suffix commits to the
exact byte count while preserving one-pass hashing.

`ChunkId` contains the admitted length and digest. It has no public text or
binary codec in this slice. A future layout format must define its boundary
encoding without moving this identity.

## Vectors

`identities.tsv` contains three independently reproducible witnesses:

- the one-byte lower bound;
- project-authored text spanning ordinary bytes and a final newline;
- the registered CDC profile's hard maximum filled with zero bytes.

Recipes are bounded and deterministic. `repeated-byte-v1` repeats one byte;
`hex-repeat-v1` repeats one nonempty decoded hexadecimal pattern.

The checker validates canonical TSV framing, recipe bounds, declared lengths,
and exact BLAKE3-256 output. Rust tests consume the same table through the
public `ChunkId::hash_bytes` path.
