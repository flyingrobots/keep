# ADR-0003: Deterministic Content-Defined Chunking Profiles

- Status: Accepted
- Date: 2026-07-19
- Owners: Keep chunking and storage-profile layers
- Related issue: [#7](https://github.com/flyingrobots/keep/issues/7)
- Depends on: [ADR-0001](0001-exact-logical-byte-identity.md),
  [ADR-0002](0002-separate-identity-from-physical-storage.md)

## Context

Content-defined chunking (CDC) lets nearby versions reuse chunks after edits.
Boundaries are storage facts, not logical identity: changing a profile may
produce a new layout, but cannot move `BlobId` when the bytes are unchanged.

An algorithm name such as “FastCDC” is not an interoperability contract.
Implementations differ in table values, integer width, masks, normalization,
boundary inclusion, buffering, and end-of-stream behavior. Keep therefore owns
an implementation-independent scalar boundary algorithm and gives every
boundary-affecting parameter a canonical, typed `StorageProfileId`.

The design follows the
[2016 FastCDC paper](https://www.usenix.org/conference/atc16/technical-sessions/presentation/xia),
but its exact mechanics are Keep protocol, not compatibility with every FastCDC
implementation.

## Decision

Keep defines boundary algorithm `1`, named `keep.fastcdc-gear64/v1`, and the
first registered profile `fastcdc-64k-v1`.

### Standard Gear table

Algorithm 1 uses one immutable 256-entry table `G` of unsigned 64-bit integers.
The table is generated without host-dependent state. For each byte value `i`
from 0 through 255:

1. construct exactly 64 bytes, each equal to `i`;
2. compute the 16-byte MD5 digest of those 64 bytes;
3. interpret digest bytes 0 through 7 as one unsigned big-endian `u64`.

This reproduces the standard table used by FastCDC implementations; the pinned
[table generator](https://github.com/nlfiedler/fastcdc-rs/blob/2e47aa3146c6dbae34896997eebd162b280a7052/examples/table64.rs)
is an inspectable reference. MD5 is used only to generate fixed constants. It
does not authenticate content, profiles, or stored chunks.

The table is named by a typed BLAKE3-256 checksum over:

```text
ASCII "KEEP:GEAR:TABLE\0" || u16be(1) || u8(1) || T || u64be(2048)
```

`T` is all 256 table entries in index order, each encoded as `u64be`; algorithm
value `1` means BLAKE3-256. The canonical checksum is:

```text
4194fca74d7987cb8243f26dd6496e8ffc55ba2b139737476add84cc5bc69da7
```

An independent generator reproduced the table bytes and checksum with Python's
standard-library MD5 plus `b3sum` 1.8.5. An implementation must refuse an
unknown table checksum; it must not swap in a convenient table with the same
dimensions.

### Scalar boundary algorithm

The input is one finite byte sequence. Source read partitions, empty reads, and
host buffer sizes are not part of that sequence and cannot affect boundaries.

At the beginning of every chunk, reset `h` to the profile seed. Algorithm 1
currently admits only seed zero. There is no eviction window. The first
`minimum` bytes of a candidate are appended without updating `h` or testing a
boundary. This is cut-point skipping.

After the minimum is present, emit immediately if it is also the maximum.
Otherwise, process the next source byte `b` as a probe at coordinate `p`, where
`p` is the candidate's current byte length:

```text
h = (h << 1).wrapping_add(G[b]) modulo 2^64
mask = if p < target { short_mask } else { long_mask }
```

If `(h & mask) == 0`, emit the exclusive span `S[0..p]`. The probe byte `S[p]`
is not part of the emitted chunk: carry it into the next candidate, reset the
hash state, and treat that byte as the next chunk's first byte. This exclusive
cut convention is normative.

If the mask does not match, append the probe byte. When the candidate reaches
`maximum`, emit it immediately without reading or hashing another byte. This is
a hard maximum, including for adversarial inputs that never satisfy a mask.

At end of stream, empty input emits zero chunks and every positive residual
candidate is emitted, including a final runt shorter than `minimum`. A
zero-length chunk is never emitted.

The result is an ordered partition of the input: concatenating emitted chunks
must reproduce every input byte exactly once. Every non-final chunk has length
in `[minimum, maximum]`; the final chunk has length in `[1, maximum]`.

### Source-partition invariance

Streaming implementations must preserve the candidate, hash, and carried byte
across reads. They may not treat a short read as EOF. EOF is a separate input
event, and only EOF may flush a runt. Processing the same bytes as one read,
one-byte reads, irregular reads, or reads split at every expected boundary must
produce identical chunk offsets and lengths.

Profile sizes are `u32`; total coordinates use checked `u64` arithmetic. A
detector borrows feed slices and retains counters, the fingerprint, and at most
one carried byte. A materializer retains at most `maximum` candidate bytes plus
that byte. Reader adapters must declare a separate finite buffer; no layer may
hide a whole-blob allocation.

### Canonical storage-profile record

A version-1 profile record is exactly 96 bytes. All integer fields are unsigned
big-endian.

<!-- markdownlint-disable MD013 -->

| Offset | Size | Field | Version-1 value or rule |
| ---: | ---: | --- | --- |
| 0 | 16 | `magic` | ASCII `KEEP:CDC:PROFILE` |
| 16 | 2 | `format_version` | `1` |
| 18 | 2 | `record_length` | `96` |
| 20 | 2 | `boundary_algorithm` | `1`, `keep.fastcdc-gear64/v1` |
| 22 | 2 | `flags` | `0` |
| 24 | 32 | `gear_table_checksum` | raw typed checksum bytes |
| 56 | 8 | `seed` | algorithm 1 admits `0` |
| 64 | 4 | `minimum` | validated positive byte length |
| 68 | 4 | `target` | validated byte length |
| 72 | 4 | `maximum` | validated byte length |
| 76 | 1 | `normalization` | `2`, meaning NC2 dual-mask judgment |
| 77 | 1 | `state_width` | `64` (`0x40`) |
| 78 | 2 | `reserved` | `0` |
| 80 | 8 | `short_mask` | nonzero `u64` |
| 88 | 8 | `long_mask` | nonzero `u64` |

<!-- markdownlint-enable MD013 -->

Validation occurs before admission or allocation. Version 1 requires
`0 < minimum <= target <= maximum`, the exact record length, all reserved bits
zero, the supported algorithm, normalization and state width, seed zero, and a
known table checksum. Short input, trailing bytes, invalid ordering, unknown
mandatory values, and noncanonical encodings are typed failures.

`StorageProfileId` is BLAKE3-256 over the exact 96 record bytes. Its only text
form is:

```text
keep:storage-profile:v1:blake3-256:<64 lowercase hexadecimal characters>
```

Parsing is strict: fixed tokens are exact lowercase text; uppercase hex,
whitespace, prefixes, missing digits, and trailing data are rejected. Parsing
proves canonical shape, not that the implementation supports the profile.
Version 1 admission supports only the exact registered record and identity
below. A structurally well-formed but unregistered record remains unsupported;
callers cannot supply arbitrary masks or resource bounds.

### First registered profile

`fastcdc-64k-v1` has these exact parameters:

| Parameter | Value |
| --- | ---: |
| Minimum | 16,384 bytes (`0x00004000`) |
| Target | 65,536 bytes (`0x00010000`) |
| Maximum | 262,144 bytes (`0x00040000`) |
| Normalization | NC2 (`2`) |
| Seed | `0` |
| Short mask | `0x0000d90707537000` |
| Long mask | `0x0000d90313530000` |

The masks come from the spread-bit mask family used by the pinned
[FastCDC scalar implementation](https://github.com/nlfiedler/fastcdc-rs/blob/2e47aa3146c6dbae34896997eebd162b280a7052/src/v2020/mod.rs).
The canonical profile-record digest appears in its text coordinate:

```text
keep:storage-profile:v1:blake3-256:aafa6f05bdc8894306abd41ec6f2b3b76cde995f2598fa3fd547d81fbe1a34eb
```

An independent generator reproduced the exact record and digest with `b3sum`
1.8.5. The accepted boundary corpus freezes the behavioral edge cases.

The profile is a general-purpose baseline, not a claim of optimality. Source
trees, large text, opaque binary, and compressed data use the same byte law when
it is selected. Path, media type, Git state, and caller identity are absent.

Issue #12 will measure 4/16/64 KiB and 64/256/1,024 KiB candidates. Both stay
unregistered until workload evidence justifies more durable coordinates.

Future workload-specific profiles require benchmark evidence, a new canonical
record and `StorageProfileId`, and explicit upstream selection. The core must
never silently infer or substitute a profile from file metadata.

### Layer separation and evolution

`BlobId` names exact logical bytes and excludes this profile. A future layout
format will bind its ordered chunk spans and the exact `StorageProfileId` used
to derive them. Rechunking identical bytes under another admitted profile may
move `LayoutId`; it cannot move `BlobId`.

Profiles are immutable protocol coordinates. Changing any table, seed, size,
mask, width, normalization rule, or boundary convention creates another record
and ID. Unknown profiles are refused. A friendly registry name is not identity
and must never reinterpret an existing ID.

### Security posture

Gear state is a boundary heuristic, not a cryptographic digest. Version 1 is
public and unkeyed; seed zero provides reproducibility, not unpredictability.
An attacker may craft unusual boundary distributions, but the hard maximum
bounds chunk memory and scan length. Chunk and blob verification use separate
cryptographic identities.

A keyed or privacy-oriented boundary scheme would need a separate threat model
and algorithm coordinate. Hiding a seed while publishing the same profile ID
would destroy independent reproducibility and is forbidden.

## Conformance requirements

The executable corpus freezes:

- the 2,048 canonical table bytes and typed table checksum;
- the 96-byte first-profile record, digest, and text coordinate;
- boundaries for empty, tiny, minimum-adjacent, repetitive, deterministic
  random, source-like, large-text, binary, and forced-maximum inputs;
- equivalent results across whole, one-byte, irregular, and boundary-adjacent
  source partitions;
- exclusive probe-byte carry, exact target-mask transition, EOF runt, and hard
  maximum behavior.

## Performance posture

After minimum skipping, the scalar hot path performs one table lookup, one
wrapping shift/add, one mask test, and checked cursor movement per probe byte.
There is no eviction window, division, allocation, or cryptographic hash in the
boundary loop. The selected 64 KiB target trades metadata volume against
nearby-state reuse; M2 benchmarks must measure throughput, allocation, peak
memory, boundary distribution, and reuse before any “optimal” claim.

The [2020 FastCDC paper](https://doi.org/10.1109/TPDS.2020.2984632) describes
additional two-byte-at-a-time optimization. It is not part of this scalar
contract. A vectorized or multi-byte implementation is admissible only if it
reproduces every scalar boundary vector exactly.

## Alternatives considered

### Fixed-size chunking

Rejected as the canonical reuse layout. It is simple and fast, but an insertion
near the front shifts every later boundary. It remains useful as a benchmark
baseline or an explicitly different profile family.

### Rabin-fingerprint CDC

Rejected for version 1. Rabin CDC is established and shift-resistant, but its
eviction window and polynomial arithmetic create a larger scalar hot path and
interoperability surface. Keep chooses Gear-based FastCDC for the first measured
path; benchmarks may justify another algorithm coordinate later.

### Git pack deltas

Rejected as a chunking contract. Git's official
[pack format](https://git-scm.com/docs/pack-format) defines representation
deltas against selected bases. It does not define a bounded streaming partition
whose independent chunks can be globally reused, verified, and range-planned.

### Adopt git-cas defaults

Rejected. The current
[git-cas CDC implementation](https://github.com/git-stunts/git-cas/blob/432c5d9effb12c9f66536f1386791bb4421f3cea/src/infrastructure/chunkers/CdcChunker.js)
uses 32-bit Buzhash, a 64-byte eviction window, JavaScript number behavior, and
different default sizes. It is useful prior art, but copying its defaults would
not create Keep's cross-language canonical protocol.

### Depend directly on a FastCDC crate

Rejected as the protocol definition. A crate may be used behind conformance
tests, but dependency versions expose several algorithm variants, seed rules,
and buffering APIs. Keep must retain a small independently implementable law and
must not let a dependency upgrade move durable boundaries.

## Non-goals

This ADR does not implement the chunker, define `ChunkId` or the durable layout
codec, select compression or encryption, select a profile from path metadata,
or claim that `fastcdc-64k-v1` is optimal. Those require their own executable
evidence and, where identity or format changes, their own ADRs.

## Consequences

- Implementations can derive identical boundaries without sharing code.
- Streaming reads remain partition-invariant and memory-bounded.
- Malicious no-match input cannot exceed the profile's hard maximum chunk.
- Profile evolution is explicit and cannot silently move logical identity.
- The accepted profile and independent vectors are a durable compatibility
  obligation.
