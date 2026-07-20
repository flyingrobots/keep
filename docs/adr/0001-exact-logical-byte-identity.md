# ADR-0001: Exact Logical Byte Identity

- Status: Accepted
- Date: 2026-07-19
- Owners: Keep identity layer
- Related issues: [#2](https://github.com/flyingrobots/keep/issues/2),
  [#6](https://github.com/flyingrobots/keep/issues/6)

## Context

Keep's core law is:

> For a given content identity, Keep must return exactly the bytes named by
> that identity—or refuse.

That law requires a stable logical identity before storage layout, compression,
encryption, retention, or physical placement exist. A raw digest is
insufficient as a protocol: it does not identify the semantic domain, hash
algorithm, identity version, or logical byte length, and it gives decoders no
canonical external representation to enforce.

The identity calculation must also accept an unknown-length byte stream in one
pass with bounded memory. Requiring a length prefix would force a pre-scan,
seek, spool, or intermediate hash for such a stream.

## Decision

Version 1 defines `BlobId` as the identity of one exact, finite logical byte
sequence. It uses a 256-bit BLAKE3 digest over a typed canonical preimage.

No Unicode normalization, newline conversion, metadata, pathname, media type,
chunk boundary, compression setting, encryption setting, storage coordinate,
or application meaning participates. Bytes are bytes.

### Canonical hash preimage

For payload `P` of length `N`, where `0 <= N <= 2^64 - 1`:

```text
blob_digest_v1(P) = BLAKE3-256(
    data_magic       ||
    identity_version ||
    hash_algorithm   ||
    P                ||
    logical_length
)
```

The exact byte grammar is:

| Offset | Size | Field | Encoding |
| ---: | ---: | --- | --- |
| 0 | 16 | `data_magic` | ASCII `KEEP:BLOB:DATA` followed by two zero bytes |
| 16 | 2 | `identity_version` | unsigned big-endian integer `1` |
| 18 | 1 | `hash_algorithm` | unsigned integer `1`, meaning BLAKE3-256 |
| 19 | `N` | `P` | exact logical content bytes |
| `19 + N` | 8 | `logical_length` | `N` as an unsigned big-endian integer |

The total preimage length is `27 + N` bytes. The fixed 8-byte suffix makes the
single variable-width field self-delimiting from the end. Appending the length
allows a caller to hash an unknown-length stream in one pass: initialize with
the fixed prefix, hash each byte exactly once while checked-counting it, then
hash the final count.

Length overflow is a typed failure. A chunk that would overflow the `u64`
counter is rejected before either the counter or hasher state changes.

### `BlobId` value

A validated version-1 `BlobId` contains:

- identity version `1`;
- hash algorithm `1` (BLAKE3-256);
- logical byte length `N`;
- the 32-byte `blob_digest_v1(P)`.

Version and algorithm are explicit on the wire. The Rust version-1 type may
store them implicitly because no other values are admissible.

The empty byte sequence is valid and distinct from raw `BLAKE3("")` because
the typed prefix and zero length suffix participate.

### Canonical text representation

The only accepted text form is:

```text
keep:blob:v1:blake3-256:<length>:<digest>
```

Rules:

- the five fixed tokens are lowercase and exact;
- `length` is canonical unsigned decimal: `0`, or a nonzero digit followed by
  zero or more digits;
- leading zeroes, signs, whitespace, separators, and values above `u64::MAX`
  are rejected;
- `digest` is exactly 64 lowercase hexadecimal characters;
- uppercase hexadecimal, alternate alphabets, prefixes, and trailing data are
  rejected;
- parsing validates structure but cannot prove possession of the named bytes.

`Display` emits only this form. `FromStr` accepts only this form.

### Canonical binary representation

The canonical binary form is exactly 59 bytes:

| Offset | Size | Field | Encoding |
| ---: | ---: | --- | --- |
| 0 | 16 | `id_magic` | ASCII `KEEP:BLOB:ID` followed by four zero bytes |
| 16 | 2 | `identity_version` | unsigned big-endian integer `1` |
| 18 | 1 | `hash_algorithm` | unsigned integer `1`, meaning BLAKE3-256 |
| 19 | 8 | `logical_length` | unsigned big-endian byte length |
| 27 | 32 | `digest` | raw BLAKE3-256 output |

Short input, long input, wrong magic, unsupported version, unsupported
algorithm, and any other malformed field are distinct typed decoding failures.
Because the form has a fixed size, all extra bytes are trailing data and are
rejected.

### Equality and ordering

Equality compares the complete validated value: version, algorithm, logical
length, and digest. Version 1 stores only length and digest because its version
and algorithm are invariant.

Ordering is a deterministic indexing convenience, not semantic precedence. It
is the lexicographic ordering of canonical binary forms, equivalently the tuple
`(version, algorithm, logical_length, digest)` with integer fields compared
numerically.

### Verification

Parsing an identity proves only that its representation is canonical and
supported. Verification requires hashing candidate bytes with this preimage and
comparing the complete computed `BlobId` to the expected value.

Implementations must distinguish at least:

- malformed identity;
- unsupported identity version;
- unsupported hash algorithm;
- byte-stream read failure;
- logical length overflow;
- computed identity mismatch.

Keep never repairs, substitutes, or returns candidate bytes after a mismatch.

## Golden vectors

The authoritative machine-readable vectors live in
`conformance/golden-file-worldline/v1/identities.tsv`. These values were
generated before the production `BlobId` implementation by constructing the
specified preimage and hashing it with the independently installed `b3sum
1.8.5` executable.

<!-- markdownlint-disable MD013 -->

| Case | Logical length | Canonical digest |
| --- | ---: | --- |
| Empty | 0 | `c0074a279c09f9d019dc10e4c821f79f1450cfb8541ab4627132ab9f3c75e33f` |
| UTF-8 `Keep exact bytes.\n` | 18 | `af75d70e4993121254ac71f16c5edd02410a36f94d795e4d6064ed3122b7967d` |
| Byte ramp `00` through `ff` | 256 | `e782f90f48483f6a8520c9b05eca57ace1647374dd9456b9e41aadccacd10f12` |
| 1 MiB repeated byte ramp | 1,048,576 | `25399c3df18ecd403c8cacf50a44409e005ca71452e4ad367bd14423c1f86e20` |

<!-- markdownlint-enable MD013 -->

The conformance corpus also contains nearby worldline states A and B, their
canonical text and binary identities, malformed encodings, truncations, and
single-bit content mutations. A single-bit content mutation must compute a
different identity; a single-bit encoded-identity mutation must either decode
to a different supported identity or be rejected. It must never verify the
original bytes as the original identity.

## Allocation and performance implications

- Identity calculation is `O(N)` and hashes each content byte once.
- The incremental state is constant-sized; callers choose their bounded read
  buffer.
- No content-sized allocation, seek, pre-scan, or temporary spool is required.
- Text formatting writes directly to the caller's formatter.
- Binary encoding is a fixed `[u8; 59]` value.
- Text parsing is bounded by the canonical form's maximum length and need not
  allocate.
- Verification reads the candidate stream once and compares the validated
  version-1 length and digest.

BLAKE3 was selected for high software throughput, tree-hash parallelism when a
future measured path warrants it, wide platform support, and an established
Rust implementation. Version 1 deliberately uses the ordinary portable
streaming API; SIMD selection remains an implementation detail and cannot move
identity.

## Compatibility law

The following operations must not move `BlobId` when the exact logical bytes do
not change:

- rechunking;
- repacking;
- compression or decompression;
- encryption, re-encryption, or key rotation;
- storage-tier migration;
- segment compaction;
- catalog rebuild;
- copying between Keep implementations.

A future identity version or algorithm is a new typed coordinate. Keep must not
reinterpret a version-1 identity under different rules.

## Alternatives considered

### Raw `BLAKE3(P)`

Rejected as Keep's public identity. It is fast and content-only, but its 32
bytes carry no domain, version, algorithm, or length contract. Using the same
digest-shaped primitive across unrelated protocols invites type confusion and
makes strict decoding impossible.

### Git object identity

Rejected. Git hashes `"blob " || decimal_length || NUL || P` using the
repository's selected object algorithm. It couples Keep identity to Git's
protocol, algorithm transition, decimal framing, and repository compatibility
surface. Keep must work without Git and must preserve identity across physical
backends.

### Length prefix before content

Rejected for version 1. It is canonical but requires knowing the length before
hashing. Unknown-length streams would need a pre-scan, seek, spool, or
intermediate digest. A fixed-width length suffix is equally injective for the
single variable field and preserves one-pass operation.

### Hash content, then hash a typed envelope around the content digest

Rejected for version 1. This supports unknown-length input but performs an
unnecessary second hash and makes the identity a commitment to an intermediate
digest rather than directly to the typed content stream. The length suffix
achieves the same streaming property with one BLAKE3 state.

### Algorithm chosen by configuration

Rejected. Configuration-dependent identity would let identical bytes acquire
ambiguous identities without a visible protocol transition. Algorithm agility
is explicit through the encoded version and algorithm fields; unsupported
coordinates are refused.

## Consequences

- Keep gains a small, independently implementable identity protocol.
- Unknown-length streams remain one-pass and bounded.
- External identities are self-describing enough to refuse unsupported rules.
- The logical length is authenticated and available without reading content.
- A `BlobId` does not prove retention, durability, location, or semantic type.
- Version-1 identities are permanent compatibility commitments once released.
