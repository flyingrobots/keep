# Catalog and Publication-Head Bytes

This page owns catalog generations, publication-head framing, and the fixed
bounds for `keep.segment-store/v1`. It uses the canonical primitives defined
by the [segment-byte grammar](segment.md#canonical-primitives).

## Catalog generation

A catalog generation is:

```text
catalog_header || sorted_entries || catalog_checksum || catalog_digest
```

### Catalog header

The catalog header is exactly 128 bytes:

<!-- markdownlint-disable MD013 -->

| Offset | Size | Field | Version-1 value or rule |
| ---: | ---: | --- | --- |
| 0 | 16 | `magic` | ASCII `KEEP:CATALOG:V1` followed by NUL |
| 16 | 2 | `format_version` | `1` |
| 18 | 2 | `flags` | zero |
| 20 | 2 | `header_length` | `128` |
| 22 | 2 | `entry_length` | `160` |
| 24 | 8 | `generation` | positive checked generation |
| 32 | 32 | `previous_catalog_digest` | all zero only for generation 1 |
| 64 | 8 | `entry_count` | exact entry count |
| 72 | 8 | `catalog_length` | `128 + entry_count * 160 + 64` |
| 80 | 1 | `catalog_checksum_algorithm` | `1` |
| 81 | 1 | `catalog_digest_algorithm` | `1` |
| 82 | 46 | `reserved` | all zero |

<!-- markdownlint-enable MD013 -->

Generation 1 is the only generation with an all-zero predecessor digest.
Every later generation is exactly one greater than the current verified head
and embeds that head's catalog digest. Overflow is refused.

### Catalog entry

Every entry is exactly 160 bytes:

<!-- markdownlint-disable MD013 -->

| Relative offset | Size | Field | Encoding |
| ---: | ---: | --- | --- |
| 0 | 1 | `record_kind` | `1` chunk, `2` flat layout |
| 1 | 1 | `flags` | zero |
| 2 | 2 | `identity_length` | `36` chunk, `60` layout |
| 4 | 60 | `identity` | same canonical slot as the record header |
| 64 | 32 | `segment_digest` | exact sealed-segment digest |
| 96 | 8 | `record_offset` | absolute offset from segment start |
| 104 | 8 | `record_length` | exact complete record length |
| 112 | 8 | `payload_length` | exact payload length |
| 120 | 32 | `record_checksum` | checksum copied from the named record |
| 152 | 8 | `reserved` | all zero |

<!-- markdownlint-enable MD013 -->

Entries are strictly sorted by `(record_kind, meaningful_identity_bytes)`.
The meaningful bytes are the first `identity_length` bytes of the identity
slot. Duplicate keys are refused. No hash-map or filesystem iteration order
may affect encoded order.

Each location must satisfy checked bounds:

```text
record_offset >= 64
record_length >= 144
record_offset + record_length <= bytes_before_seal
payload_length + 144 = record_length
```

The named segment must exist at the digest-derived immutable-pool name and
verify completely. The record at the declared span must reproduce the entry's
kind, identity, lengths, and checksum exactly.

Catalog admission scans the complete segment grammar from byte 64 through the
declared record count and records every top-level record span before admitting
locations. Each `(record_offset, record_length)` pair must equal one discovered
top-level record span. A location into a record header, payload, checksum, or
segment seal is refused even when those embedded bytes independently resemble
a valid record.

Let `C` be the complete header and sorted entries. The two 32-byte trailer
fields are:

```text
catalog_checksum = framed_blake3_v1(
    ASCII("KEEP:CATALOG:SUM\0"),
    C
)

catalog_digest = framed_blake3_v1(
    ASCII("KEEP:CATALOG:DIGEST\0"),
    C || catalog_checksum
)
```

The catalog digest is a physical generation coordinate and predecessor
witness. It does not establish retention or application history.

## Publication head

The publication head is exactly 128 bytes:

<!-- markdownlint-disable MD013 -->

| Offset | Size | Field | Version-1 value or rule |
| ---: | ---: | --- | --- |
| 0 | 16 | `magic` | ASCII `KEEP:CATHEAD:V1` followed by NUL |
| 16 | 2 | `format_version` | `1` |
| 18 | 2 | `flags` | zero |
| 20 | 2 | `head_length` | `128` |
| 22 | 1 | `head_checksum_algorithm` | `1` |
| 23 | 1 | `catalog_digest_algorithm` | `1` |
| 24 | 8 | `generation` | exact positive catalog generation |
| 32 | 8 | `catalog_length` | exact named catalog length |
| 40 | 32 | `catalog_digest` | exact named catalog digest |
| 72 | 24 | `reserved` | all zero |
| 96 | 32 | `head_checksum` | checksum defined below |

<!-- markdownlint-enable MD013 -->

```text
head_checksum = framed_blake3_v1(
    ASCII("KEEP:CATHEAD:SUM\0"),
    head[0..96]
)
```

A head is admitted only after its checksum, generation, length, exact
digest-derived catalog name, complete catalog bytes, catalog predecessor law,
and every catalog-referenced segment and record are verified.

## Bounds

Version 1 defines:

<!-- markdownlint-disable MD013 -->

| Bound | Value |
| --- | ---: |
| `SEGMENT_HEADER_LENGTH` | 64 bytes |
| `RECORD_HEADER_LENGTH` | 112 bytes |
| `RECORD_CHECKSUM_LENGTH` | 32 bytes |
| `SEGMENT_SEAL_LENGTH` | 128 bytes |
| `MAX_RECORD_PAYLOAD_LENGTH` | 67,108,864 bytes |
| `MAX_RECORD_LENGTH` | 67,109,008 bytes |
| `MAX_SEGMENT_LENGTH` | 1,073,741,824 bytes |
| `MAX_SEGMENT_RECORD_COUNT` | 1,048,576 |
| `CATALOG_HEADER_LENGTH` | 128 bytes |
| `CATALOG_ENTRY_LENGTH` | 160 bytes |
| `CATALOG_TRAILER_LENGTH` | 64 bytes |
| `MAX_CATALOG_ENTRY_COUNT` | 1,048,576 |
| `MAX_CATALOG_LENGTH` | 167,772,352 bytes |
| `PUBLICATION_HEAD_LENGTH` | 128 bytes |
| `MAX_RECOVERY_INVENTORY_ENTRY_COUNT` | `2,097,152` |

<!-- markdownlint-enable MD013 -->

The segment-length bound and record framing impose a lower effective record
count when records carry payloads. A decoder checks both limits. No protocol
bound authorizes one allocation of that size; adapters stream records and
entries and document any lower configured cap.
