# Segment Bytes

This page owns the canonical primitives, record framing, and immutable segment
grammar for `keep.segment-store/v1`.

## Canonical primitives

All integers are unsigned, fixed-width, and big-endian. Every flag bit is
mandatory-to-understand; version 1 requires all flags and reserved bytes to be
zero. There are no serializer-owned values, variable-width integers, maps,
optional fields, duplicate fields, or trailing bytes.

Algorithm value `1` means BLAKE3-256. Version value `1` means the exact grammar
on this page.

For a domain string `D` and exact bytes `B`, this page uses:

```text
framed_blake3_v1(D, B) = BLAKE3-256(
    D
    || u16be(1)
    || u8(1)
    || B
    || u64be(length(B))
)
```

Every length conversion and addition is checked before allocation, cursor
movement, seek, or comparison.

## Sealed segment

A segment is:

```text
segment_header || records || segment_seal
```

`records` is the exact concatenation of `record_count` complete records.
There is no padding or alignment between records.

### Segment header

The segment header is exactly 64 bytes:

<!-- markdownlint-disable MD013 -->

| Offset | Size | Field | Version-1 value or rule |
| ---: | ---: | --- | --- |
| 0 | 16 | `magic` | ASCII `KEEP:SEGMENT:V1` followed by NUL |
| 16 | 2 | `format_version` | `1` |
| 18 | 2 | `flags` | zero |
| 20 | 2 | `header_length` | `64` |
| 22 | 2 | `record_header_length` | `112` |
| 24 | 2 | `seal_length` | `128` |
| 26 | 2 | `reserved` | zero |
| 28 | 8 | `maximum_record_payload_length` | `67,108,864` |
| 36 | 8 | `maximum_segment_length` | `1,073,741,824` |
| 44 | 4 | `maximum_record_count` | `1,048,576` |
| 48 | 1 | `record_checksum_algorithm` | `1` |
| 49 | 1 | `segment_digest_algorithm` | `1` |
| 50 | 14 | `reserved` | all zero |

<!-- markdownlint-enable MD013 -->

A decoder compares these protocol bounds exactly. A store may enforce lower
configured limits, but it cannot encode those limits into version-1 bytes or
admit bytes above the protocol limits.

### Record

Each record is:

```text
record_header || payload || record_checksum
```

The record header is exactly 112 bytes:

<!-- markdownlint-disable MD013 -->

| Offset | Size | Field | Version-1 value or rule |
| ---: | ---: | --- | --- |
| 0 | 16 | `magic` | ASCII `KEEP:SEG:RECORD` followed by NUL |
| 16 | 2 | `record_version` | `1` |
| 18 | 1 | `record_kind` | `1` chunk, `2` flat layout |
| 19 | 1 | `flags` | zero |
| 20 | 2 | `header_length` | `112` |
| 22 | 2 | `identity_length` | `36` for chunk, `60` for layout |
| 24 | 8 | `payload_length` | exact payload byte length |
| 32 | 8 | `record_length` | `112 + payload_length + 32` |
| 40 | 1 | `record_checksum_algorithm` | `1` |
| 41 | 2 | `identity_version` | `1` |
| 43 | 1 | `identity_algorithm` | `1` |
| 44 | 4 | `reserved` | all zero |
| 48 | 60 | `identity` | kind-specific canonical identity slot |
| 108 | 4 | `reserved` | all zero |

<!-- markdownlint-enable MD013 -->

For record kind `1`, identity bytes 0–3 are the positive `u32be`
`ChunkLength`, identity bytes 4–35 are the version-1 `ChunkId` digest, and
identity bytes 36–59 are zero. `payload_length` equals the embedded chunk
length. The payload is the exact chunk bytes.

For record kind `2`, all 60 identity bytes are the canonical binary
`LayoutId` from `keep.flat-chunks/v1`. `payload_length` equals the plan length
embedded in that identity. The payload is the exact canonical flat-layout
record.

Unknown kinds, kind/identity-length mismatch, unsupported identity
coordinates, zero chunk length, length disagreement, or nonzero unused slot
bytes are refused.

Let `H` be the complete 112-byte record header and `P` the exact payload. The
final 32 bytes are:

```text
record_checksum = framed_blake3_v1(
    ASCII("KEEP:SEG:RECORD:SUM\0"),
    H || P
)
```

Decoding verifies framing and checksum but makes no content-verification
claim. Record admission additionally recomputes the kind-specific logical
identity from `P` and compares the complete observed identity with the
header.

Complete-segment admission walks exactly the seal's declared record count,
admits every record under caller-selected record and nested-layout limits,
refuses trailing record bytes, and refuses duplicate logical identities.
Record iteration preserves physical order; the temporary duplicate-detection
index does not define a canonical record order.

The reader admits the seal's fixed coordinates and checksum before using its
record count, then admits every nested record, and verifies the physical
segment digest last. No payload is exposed until all phases succeed. This
ordering preserves fail-closed outer framing while allowing record checksums
to localize ordinary record corruption.

### Segment seal

The segment seal is exactly 128 bytes:

<!-- markdownlint-disable MD013 -->

| Offset | Size | Field | Version-1 value or rule |
| ---: | ---: | --- | --- |
| 0 | 16 | `magic` | ASCII `KEEP:SEGMENT:END` |
| 16 | 2 | `seal_version` | `1` |
| 18 | 2 | `flags` | zero |
| 20 | 2 | `seal_length` | `128` |
| 22 | 2 | `reserved` | zero |
| 24 | 4 | `record_count` | exact record count |
| 28 | 4 | `reserved` | zero |
| 32 | 8 | `bytes_before_seal` | header plus complete records |
| 40 | 8 | `segment_length` | exact complete file length |
| 48 | 8 | `record_bytes` | exact concatenated record bytes |
| 56 | 1 | `seal_checksum_algorithm` | `1` |
| 57 | 1 | `segment_digest_algorithm` | `1` |
| 58 | 6 | `reserved` | all zero |
| 64 | 32 | `segment_digest` | physical digest defined below |
| 96 | 32 | `seal_checksum` | seal checksum defined below |

<!-- markdownlint-enable MD013 -->

Let `S` be the segment header and complete records. Let `Q` be seal bytes
0–63. Then:

```text
segment_digest = framed_blake3_v1(
    ASCII("KEEP:SEGMENT:DIGEST\0"),
    S || Q
)

seal_checksum = framed_blake3_v1(
    ASCII("KEEP:SEGMENT:SEAL:SUM\0"),
    seal[0..96]
)
```

The digest is a physical immutable-segment coordinate. It is not a logical
content identity, retention claim, authentication tag, or proof of
publication.
