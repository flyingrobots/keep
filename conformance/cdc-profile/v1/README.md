# FastCDC 64 KiB Profile Corpus v1

This directory is the implementation-independent executable corpus for Keep's
first deterministic content-defined chunking profile. It freezes the physical
layout decision without defining chunk identity or changing logical `BlobId`.

Run the independent checker with:

```bash
python3 conformance/cdc-profile/v1/check_vectors.py
```

The checker requires Python 3.10 or newer and `b3sum` 1.8.5 or a compatible
implementation on `PATH`. It uses Python's standard-library MD5 only to
regenerate the public, fixed Gear table; MD5 is not a content identity or
security primitive.

## Profile

The canonical profile name is `fastcdc-64k-v1`:

| Parameter | Value |
| --- | ---: |
| Boundary algorithm | scalar Gear64/FastCDC (`1`) |
| Gear state width | 64 bits |
| Seed | `0` |
| Minimum size | 16,384 bytes |
| Target size | 65,536 bytes |
| Maximum size | 262,144 bytes |
| Normalization | level 2 |
| Short-region mask | `0x0000d90707537000` |
| Long-region mask | `0x0000d90313530000` |

All arithmetic on the fingerprint is wrapping unsigned 64-bit arithmetic.
The fingerprint update for byte `b` is:

```text
fingerprint = ((fingerprint << 1) + gear[b]) mod 2^64
```

For a non-empty remaining source:

1. If its length is at most `minimum`, emit it as the final chunk.
2. Limit the candidate chunk to `min(remaining, maximum)` bytes.
3. Start `p` at `minimum`, with the fingerprint reset to `seed`.
4. For each `p < min(target, limit)`, update with byte `source[p]` and
   probe the short-region mask.
5. Continue through each `p < limit`, probing the long-region mask.
6. On a matching probe at `p`, emit the half-open range `[0, p)`.
7. If no probe matches, emit `[0, limit)`.
8. Reset the fingerprint for the next chunk.

The probe byte at `p` participates in the boundary fingerprint but is not part
of the emitted `[0, p)` chunk. It becomes byte zero of the next chunk, within
that chunk's skipped prefix and under freshly reset state. Empty input emits no
chunks. An EOF remainder may be shorter than `minimum`; every other chunk is
between `minimum` and `maximum`, inclusive.

## Gear table

`gear-table.bin` is authoritative protocol material: exactly 256 unsigned
64-bit integers in byte-value order, each encoded big-endian. The reproducible
recipe for entry `i` is the first eight bytes of:

```text
MD5(byte(i) repeated 64 times)
```

The typed checksum preimage is:

```text
ASCII("KEEP:GEAR:TABLE\0")
|| u16be(1)
|| u8(1)                         # BLAKE3-256
|| gear-table.bin
|| u64be(2048)
```

Its BLAKE3-256 digest is recorded in `profile.tsv`. Runtime implementations
must consume or embed bytes identical to the authoritative fixture; the MD5
recipe exists for independent verification, not runtime table generation.

## Canonical profile record

`profile-record.bin` is exactly 96 bytes:

| Offset | Width | Field |
| ---: | ---: | --- |
| 0 | 16 | ASCII `KEEP:CDC:PROFILE` |
| 16 | 2 | format version `1`, u16be |
| 18 | 2 | record length `96`, u16be |
| 20 | 2 | boundary algorithm `1`, u16be |
| 22 | 2 | flags `0`, u16be |
| 24 | 32 | typed Gear table checksum |
| 56 | 8 | seed, u64be |
| 64 | 4 | minimum size, u32be |
| 68 | 4 | target size, u32be |
| 72 | 4 | maximum size, u32be |
| 76 | 1 | normalization level |
| 77 | 1 | state width in bits |
| 78 | 2 | reserved zero, u16be |
| 80 | 8 | short-region mask, u64be |
| 88 | 8 | long-region mask, u64be |

`StorageProfileId` is BLAKE3-256 of those 96 bytes, rendered exactly as:

```text
keep:storage-profile:v1:blake3-256:<64 lowercase hexadecimal digits>
```

Every parameter that can move a boundary is therefore covered by the profile
identity. A future profile gets a different record and identity while `BlobId`
continues to name the same logical bytes.

## Source recipes

Recipe names are protocol, not checker implementation details:

- `empty-v1` requires parameter `-` and count zero.
- `repeated-byte-v1` repeats the one byte encoded by its two lowercase hex
  digits exactly `count` times.
- `alternating-v1` repeats its two-byte lowercase-hex pattern `count` times.
- `file-repeat-v1` repeats the exact bytes at its safe corpus-relative path
  `count` times.
- `xorshift64-v1` parses its parameter as one nonzero big-endian `u64` seed.
  For each output byte, update state in order with `x ^= x << 13`,
  `x ^= x >> 7`, and `x ^= x << 17`, reducing to unsigned 64 bits after each
  step, then emit the low eight bits. `count` is the number of output bytes.

Mutation `offset` and `span_length` are byte coordinates in the named base.
`insert-v1` inserts decoded `value_hex`; `delete-v1` removes the span and uses
`-` for value; `xor-v1` XORs equal-length decoded value bytes with the span.
Every recipe also declares the exact resulting `logical_length`.

## Protocol tables

- `profile.tsv` records the authoritative table checksum and profile identity.
- `sources.tsv` defines bounded, deterministic primitive source recipes.
- `mutations.tsv` defines early insertion, deletion, and XOR variants.
- `boundaries.tsv` records absolute exclusive chunk-end offsets.
- `inputs/source-block.txt` is project-authored source-like fixture material.

All TSV files are UTF-8, tab-delimited, canonical lowercase where applicable,
and terminated by exactly one final LF. `-` denotes an inapplicable field, not
an empty string. Do not reorder rows, normalize source bytes, or regenerate
expected boundaries with Keep's production chunker.

The checker independently validates the table recipe and typed checksum,
profile record and identity, recipe bounds, exact boundaries, reconstruction,
chunk size laws, maximum-size forcing, and invariance under whole, bytewise,
fixed, cyclic, and boundary-adjacent source partition schedules.

Named witnesses freeze boundary details that broad random cases can obscure:

- `probe-byte-carry` proves a matching probe byte is excluded from one chunk
  and becomes the first byte considered by the next reset chunk;
- `short-mask-match` cuts strictly before `target` under the short mask;
- `target-long-transition` matches the long mask, but not the short mask,
  exactly at `p == target`;
- `natural-cut-runt` ends with a sub-minimum EOF chunk after a natural cut; and
- the checker injects named empty-interleaved feed schedules and proves empty
  partitions neither flush state nor move a boundary.
