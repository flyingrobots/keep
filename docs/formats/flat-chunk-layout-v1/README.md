# Flat Chunk Layout Version 1

- Status: Frozen version-1 specification; production codec implemented
- Format coordinate: `keep.flat-chunks/v1`
- Format version: `1`
- Layout codec: `1`
- Related issue: [#9](https://github.com/flyingrobots/keep/issues/9)
- Implementation issue: [#10](https://github.com/flyingrobots/keep/issues/10)
- Reconstruction issue: [#13](https://github.com/flyingrobots/keep/issues/13)
- Depends on:
  [ADR-0001](../../adr/0001-exact-logical-byte-identity.md),
  [ADR-0002](../../adr/0002-separate-identity-from-physical-storage.md),
  and [ADR-0003](../../adr/0003-deterministic-content-defined-chunking-profiles.md)

This page specifies the canonical durable plan that maps one logical
`BlobId` to an ordered, bounded sequence of exact `ChunkId` values. It is a
language-independent binary protocol.

Keep exposes validated layout admission, canonical `LayoutId` and record
encoding, and bounded record decoding. The checked-in fixtures and independent
oracle prove exact production codec behavior. The non-durable reference CAS
implements bounded ingestion and verified reconstruction; durable storage
remains a separate future boundary.

## Core law

A valid version-1 flat layout commits to:

- one target `BlobId`, including its exact logical length;
- one registered `StorageProfileId`;
- one ordered sequence of version-1 `ChunkId` values;
- one explicit logical offset and exact positive length per entry; and
- no physical segment, path, object key, catalog generation, or retention
  fact.

Concatenating the exact verified bytes named by the entries in order MUST
produce the exact bytes named by the target `BlobId`, or reconstruction MUST
refuse. Structural layout validation cannot prove that content claim without
the chunk bytes; verified reconstruction performs that final comparison.
Verified reconstruction MUST also reproduce the declared spans by replaying
the bound storage profile over those bytes, or it MUST refuse the profile
claim.

Repeated `ChunkId` values are valid when equal bytes occur more than once.
Entry order and logical offsets belong to the layout. They do not become part
of `ChunkId`.

## Canonical `LayoutId`

Version 1 assigns layout codec `1` to `keep.flat-chunks/v1`. For exact
canonical plan bytes `P`, including the 32-byte record checksum:

```text
layout_digest_v1(P) = BLAKE3-256(
    ASCII("KEEP:LAYOUT:ID\0\0")
    || u16be(1)
    || u16be(1)
    || P
    || u64be(length(P))
)
```

The first integer is the layout identity-envelope version. The second is the
layout codec. This is the envelope accepted by ADR-0002. The plan length is
checked before hashing and cannot exceed `MAX_LAYOUT_RECORD_LENGTH`.

Identity-envelope version 1 fixes BLAKE3-256. The `blake3-256` text token is
an exact coordinate component, not algorithm negotiation. A different layout
identity hash requires a new identity-envelope version.

A validated `LayoutId` contains:

- identity-envelope version `1`;
- layout codec `1`;
- exact plan length;
- the 32-byte `layout_digest_v1` result.

### Text coordinate

The only canonical text form is:

```text
keep:layout:v1:flat-chunks-v1:blake3-256:<plan_length>:<digest>
```

The fixed tokens are exact lowercase ASCII. `plan_length` is canonical
unsigned decimal with no sign, whitespace, separators, or leading zeroes
except the value `0` itself. Codec 1 accepts only lengths in the inclusive
range 176 through 46,137,520 for which
`(plan_length - 176) % 44 == 0`. The digest is exactly 64 lowercase
hexadecimal characters. A canonical coordinate is at most 114 ASCII bytes.
The parser accepts at most 128 input bytes so it can distinguish malformed
fields, including an overflowing decimal length, and refuses a longer input
before token parsing. Unknown versions, codecs, algorithms, impossible or
overlong lengths, uppercase hex, and trailing data are refused. The checked-in
[invalid-text corpus](../../../conformance/layout/v1/invalid-layout-id-text.tsv)
fixes exact refusal classes for every text field and canonicality edge.

### Binary coordinate

The canonical binary `LayoutId` coordinate is exactly 60 bytes:

| Offset | Size | Field | Version-1 value or encoding |
| ---: | ---: | --- | --- |
| 0 | 16 | `identity_magic` | ASCII `KEEP:LAYOUT:ID` followed by two zero bytes |
| 16 | 2 | `identity_version` | unsigned big-endian `1` |
| 18 | 2 | `layout_codec` | unsigned big-endian `1` |
| 20 | 8 | `plan_length` | unsigned big-endian canonical record length |
| 28 | 32 | `digest` | raw BLAKE3-256 output |

Parsing either coordinate proves only canonical supported shape. It does not
prove possession, checksum validity, structural validity, or reconstruction
of the named plan. Binary parsing applies the same codec-1 plan-length range
and congruence law as text parsing. The checked-in
[invalid-binary corpus](../../../conformance/layout/v1/invalid-layout-id-binary.tsv)
provides exact byte mutations and refusal classes.

## Canonical plan record

The canonical plan `P` is:

```text
fixed_header || entries || record_checksum
```

All integers are unsigned big-endian fixed-width values. There are no
variable-width integers, serializer defaults, map keys, tags, or optional
field encodings. The record is positional, so duplicate fields are
unrepresentable. Inserting a second encoding of any field changes the byte
length and is refused as noncanonical framing.

The exact record length is:

```text
record_length = 144 + entry_count * 44 + 32
```

Every operation in this expression uses checked arithmetic before allocation
or cursor movement.

### Fixed header

The header is exactly 144 bytes:

<!-- markdownlint-disable MD013 -->

| Offset | Size | Field | Version-1 value or rule |
| ---: | ---: | --- | --- |
| 0 | 16 | `magic` | ASCII `KEEP:LAYOUT:PLAN` |
| 16 | 2 | `format_version` | unsigned big-endian `1` |
| 18 | 2 | `layout_codec` | unsigned big-endian `1` |
| 20 | 4 | `flags` | zero; every nonzero bit is mandatory-to-understand and unsupported |
| 24 | 2 | `header_length` | unsigned big-endian `144` |
| 26 | 2 | `entry_length` | unsigned big-endian `44` |
| 28 | 8 | `record_length` | exact total plan length, including checksum |
| 36 | 4 | `entry_count` | number of 44-byte entries |
| 40 | 1 | `record_checksum_algorithm` | `1`, meaning BLAKE3-256 |
| 41 | 1 | `chunk_hash_algorithm` | `1`, meaning BLAKE3-256 |
| 42 | 2 | `chunk_identity_version` | unsigned big-endian `1` |
| 44 | 59 | `target_blob_id` | exact canonical ADR-0001 binary `BlobId` |
| 103 | 2 | `storage_profile_identity_version` | unsigned big-endian `1` |
| 105 | 1 | `storage_profile_hash_algorithm` | `1`, meaning BLAKE3-256 |
| 106 | 32 | `storage_profile_digest` | raw `StorageProfileId` digest |
| 138 | 6 | `reserved` | all zero |

<!-- markdownlint-enable MD013 -->

The profile coordinate is the typed
`keep:storage-profile:v1:blake3-256:<digest>` identity specified by ADR-0003.
Version-1 admission recognizes only registered profile identities. The
profile record itself is not duplicated inside each layout.

The header binds every entry to version-1 `ChunkId` and BLAKE3-256 once.
Each entry therefore carries the exact positive `ChunkLength` and digest
that constitute a version-1 `ChunkId` without repeating a 16-byte magic or
algorithm coordinate per chunk.

### Entry

Each entry is exactly 44 bytes:

| Relative offset | Size | Field | Encoding |
| ---: | ---: | --- | --- |
| 0 | 8 | `logical_offset` | unsigned big-endian absolute blob offset |
| 8 | 4 | `chunk_length` | unsigned big-endian positive length |
| 12 | 32 | `chunk_digest` | raw version-1 `ChunkId` digest |

Entry `i` begins at:

```text
144 + i * 44
```

That offset is computed with checked arithmetic. No entry contains a physical
location. A future catalog resolves a logical identity to physical evidence
under its own generation and verification rules.

### Record checksum

Let `R` be the exact header and entries, excluding the checksum. The final
32 bytes are:

```text
layout_record_checksum_v1(R) = BLAKE3-256(
    ASCII("KEEP:LAYOUT:SUM\0")
    || u16be(1)
    || u8(1)
    || R
    || u64be(length(R))
)
```

The checksum detects accidental corruption when no expected `LayoutId` is
available. It is not a MAC, signature, authority statement, retention proof,
or substitute for comparing the complete `LayoutId`. When an expected
`LayoutId` is available, Keep verifies both the record checksum and the
identity.

## Bounds

Version 1 defines:

| Bound | Value |
| --- | ---: |
| `LAYOUT_HEADER_LENGTH` | 144 bytes |
| `LAYOUT_ENTRY_LENGTH` | 44 bytes |
| `LAYOUT_CHECKSUM_LENGTH` | 32 bytes |
| `MAX_CANONICAL_LAYOUT_ID_TEXT_LENGTH` | 114 bytes |
| `MAX_LAYOUT_ID_TEXT_INPUT_LENGTH` | 128 bytes |
| `LAYOUT_ID_BINARY_LENGTH` | 60 bytes |
| `MAX_LAYOUT_DEPTH` | 1 record |
| `MAX_LAYOUT_ENTRY_COUNT` | 1,048,576 (`2^20`) |
| `MAX_LAYOUT_RECORD_LENGTH` | 46,137,520 bytes |

`MAX_LAYOUT_RECORD_LENGTH` is exactly:

```text
144 + 1,048,576 * 44 + 32
```

With the registered profile's 262,144-byte hard chunk maximum, a flat
version-1 plan can describe at most 256 GiB of logical bytes. Larger blobs
require a future hierarchical codec with explicit bounds; they MUST NOT bypass
the entry limit or reinterpret codec `1`.

Depth is exactly one record. Entries can name only chunks; they cannot name
another layout, record, collection, or indirect node. A decoder therefore
performs no recursion and rejects every attempt to introduce child-layout
bytes as noncanonical entry or trailing data.

A decoder MUST:

1. read at most the fixed 144-byte header before trusting `entry_count`;
2. reject `entry_count > MAX_LAYOUT_ENTRY_COUNT`;
3. calculate entry bytes and expected record length with checked arithmetic;
4. reject declared or calculated lengths above `MAX_LAYOUT_RECORD_LENGTH`;
5. require the declared length, calculated length, and actual input length to
   be equal before allocating an entry collection; and
6. stream entries and checksum calculation when materialization is
   unnecessary.

The wire bound does not authorize one allocation of the maximum size. Public
APIs MUST document their configured admission cap and allocation behavior.
An implementation may apply a lower configured cap, but that policy cannot
change the canonical bytes or identity of a plan it does admit.

## Structural laws

Validation walks entries once in encoded order with checked `u64` arithmetic.

For an empty target blob:

- `entry_count` MUST be zero; and
- the record MUST contain only its header and checksum.

For a nonempty target blob:

- `entry_count` MUST be positive;
- the first `logical_offset` MUST be zero;
- every `chunk_length` MUST be positive;
- every entry offset MUST equal the checked exclusive end of its predecessor;
- a lower offset is an overlap;
- a higher offset is a gap;
- the sequence MUST be strictly ordered by offset;
- each nonfinal length MUST be between the admitted profile's minimum and
  maximum, inclusive;
- the final length MUST be between one and the profile maximum, inclusive;
  and
- the checked final exclusive end MUST equal the logical length embedded in
  `target_blob_id`.

The structural checks prove contiguity and aggregate length. They do not prove
that chunk digests match bytes, that content-derived boundaries are natural
for the profile, or that concatenated bytes match the target `BlobId`.
Verified reconstruction proves all three content-dependent claims.

## Parse, validate, admit, verify

Implementations MUST keep these states distinct:

1. **Decoded fields** are bounded raw integers and byte arrays. They carry no
   domain trust and cannot construct public validated types.
2. **Validated layout** has exact framing, supported version and codec,
   canonical fixed widths, zero flags and reserved bytes, a valid checksum,
   canonical nested identity encodings, checked entry arithmetic, contiguous
   spans, and exact aggregate length.
3. **Admitted layout** is validated and uses locally supported
   `BlobId`, `ChunkId`, and registered `StorageProfileId` coordinates within
   the configured resource cap.
4. **Verified reconstruction** has loaded every named chunk through a
   verification boundary, compared each observed `ChunkId`, replayed the
   registered profile's boundary detector over the exact reconstructed byte
   stream, compared its emitted spans with the declared entries, and compared
   the complete observed `BlobId` with the target.

Parsing, structural validation, or admission alone MUST NOT be reported as
content verification.

## Deterministic refusal order

A version-1 decoder validates in this order:

1. minimum header availability;
2. magic, format version, codec, flags, fixed lengths, algorithms, and
   reserved bytes;
3. entry-count and record-length bounds using checked arithmetic;
4. exact declared, calculated, and actual length equality;
5. record checksum;
6. canonical nested identity coordinates and registered profile admission;
7. empty/nonempty cardinality;
8. positive entry lengths, profile bounds, encoded-order offset continuity,
   and checked aggregate length; and
9. expected `LayoutId`, when supplied.

The first failed law determines the typed boundary error. Implementations MUST
not allocate from an unbounded count, continue after ambiguous framing, or
silently canonicalize malformed bytes. Entry validation processes entries in
encoded order and stops at the first failed entry law; a later offset cannot
replace an earlier gap or overlap with another error.

## Requirement ledger

<!-- markdownlint-disable MD013 -->

| ID | Requirement | Design evidence | Implementation status |
| --- | --- | --- | --- |
| `KEEP-LAYOUT-001` | Exact magic, version, codec, and big-endian fixed-width grammar | Header table and golden records | Implemented in #10 |
| `KEEP-LAYOUT-002` | `LayoutId` uses the ADR-0002 domain and exact plan length | Identity grammar, fixture coordinates, and text and binary refusal tables | Implemented in #10 |
| `KEEP-LAYOUT-003` | Target `BlobId` and logical length are inseparable | Embedded canonical 59-byte coordinate | Implemented in #10 |
| `KEEP-LAYOUT-004` | One registered `StorageProfileId` governs all entries | Header profile coordinate | Implemented in #10 |
| `KEEP-LAYOUT-005` | Entry `ChunkId` kind, version, algorithm, length, and digest are typed | Header and entry grammar | Implemented in #10 |
| `KEEP-LAYOUT-006` | Offsets are strictly ordered, contiguous, and gap/overlap free | Structural laws and mutation ledger | Implemented in #10 |
| `KEEP-LAYOUT-007` | Entry and aggregate arithmetic is checked | Bounds and structural laws | Implemented in #10 |
| `KEEP-LAYOUT-008` | Empty and nonempty layouts have exact cardinality | Empty and one-chunk golden records | Implemented in #10 |
| `KEEP-LAYOUT-009` | Depth, counts, and record allocation are bounded before allocation | Depth 1, `2^20` entries, and 46,137,520-byte limit | Implemented in #10 |
| `KEEP-LAYOUT-010` | Nonzero flags, reserved bytes, and unknown mandatory coordinates are refused | Mutation ledger | Implemented in #10 |
| `KEEP-LAYOUT-011` | Trailing, truncated, duplicated, and noncanonical framing is refused | Mutation ledger | Implemented in #10 |
| `KEEP-LAYOUT-012` | Record checksum is typed, domain-separated, and checked | Checksum grammar and golden records | Implemented in #10 |
| `KEEP-LAYOUT-013` | Physical locations never participate | Header and entry grammar | Specified |
| `KEEP-LAYOUT-014` | Decoded, validated, admitted, and verified states remain distinct | State model | Implemented through admission in #10 and verification in #13 |
| `KEEP-LAYOUT-015` | Flat v1 never silently becomes hierarchical | Compatibility section and rationale | Specified |
| `KEEP-LAYOUT-016` | Verified reconstruction reproduces the declared spans under the bound storage profile | Verification state and profile-boundary mutation | Implemented in #13 |

<!-- markdownlint-enable MD013 -->

## Compatibility and migration

The canonical record, checksum preimage, `LayoutId` preimage, coordinate
forms, bounds, and refusal rules are compatibility commitments.

Changing any of these requires a new layout codec:

- field order, width, or endianness;
- checksum domain or algorithm;
- chunk identity version or algorithm;
- offset or length semantics;
- entry-count or record-length protocol limits;
- implicit versus explicit offsets;
- profile binding;
- hierarchy or indirection; or
- canonical text or binary identity grammar.

A new codec may coexist beside codec `1`. Migration writes and verifies a new
layout with a new `LayoutId`; it never rewrites or reinterprets an existing
codec-1 identity. `BlobId` remains stable when reconstructed logical bytes are
unchanged.

Hierarchical layouts are explicitly deferred. Codec `1` contains no child
layout references, recursion, depth field, or extension point. Unknown codecs
and nonzero flags are refused rather than guessed.

## Security and privacy

Lengths, chunk boundaries, repeated chunk identities, and the selected storage
profile are visible metadata. Version 1 provides no confidentiality.

BLAKE3 checksums and identities detect accidental or adversarial byte changes
when compared with independently obtained expected values. They do not
authenticate a writer. A malicious source that controls a complete record can
recompute its checksum and `LayoutId`; admission and application authority are
separate concerns.

The fixed header and hard count and length limits prevent attacker-controlled
allocation from preceding validation. Every offset, multiplication, addition,
and host-size conversion is checked. A decoder refuses ambiguous or
unsupported state and never returns partially authenticated bytes as verified.

## Golden and mutation evidence

The implementation-independent
[layout corpus](../../../conformance/layout/v1/README.md) contains:

- an empty plan;
- a one-byte, one-chunk plan;
- a two-chunk plan that exercises the registered profile's hard maximum and
  final runt;
- a four-chunk plan that proves repeated `ChunkId` values remain lawful at
  distinct offsets;
- exact canonical record hex, checksums, `BlobId`, and `LayoutId` coordinates;
  and
- text and binary `LayoutId` refusal tables plus a plan mutation ledger
  covering every fixed header field, entry field, checksum, truncation,
  trailing bytes, duplicate-field insertion, ordering, gap, overlap,
  aggregate mismatch, allocation limit, and later content or storage-profile
  verification failure.

The reasons for the flat grammar, explicit offsets, checksum, bounds, and
hierarchy posture are recorded in the
[colocated rationale](rationale.md).
