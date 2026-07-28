# Durable Segment Store Version 1

`keep.segment-store/v1` is the accepted physical protocol for Keep's first
durable segment store. It specifies bytes, publication, crash states, reader
visibility, and recovery as one contract.

ADR-0005 records the cross-cutting decision. This page is a protocol
commitment, not a claim that the production adapter has shipped. Production
implementation belongs to issues #15 and #16; executable crash evidence
belongs to issue #17.

## Core law

For a given logical content identity, Keep returns exactly the bytes named by
that identity or refuses.

Durable version 1 adds these physical laws:

- file existence proves no identity, completeness, publication, retention, or
  durability claim;
- only one verified publication head selects one verified immutable catalog;
- only that catalog maps logical identities to immutable physical records;
- a segment is immutable after its complete seal is synchronized;
- a generation is published only after all referenced immutable artifacts are
  synchronized and the new head replacement is directory-synchronized; and
- uncertain, conflicting, malformed, unsupported, or corrupt state is
  represented and refused, never guessed or silently repaired.

Physical locations and digests are not `BlobId`, `ChunkId`, or `LayoutId`.
Rechunking, migration, and later compaction may move physical evidence without
moving logical identity.

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

## Physical namespace

The version-1 filesystem adapter owns this relative namespace:

```text
writer.lock
HEAD
head.next
staging/current.seg
staging/current.cat
segments/<segment-digest>.seg
catalogs/<generation>-<catalog-digest>.cat
```

Digest components are exactly 64 lowercase hexadecimal characters.
`generation` is exactly 16 lowercase hexadecimal digits. All operations are
capability-relative and refuse symlinks, nonregular files, alternate
spellings, unknown entries in protocol-owned directories, and replacement of
the opened store root.

The filesystem must expose case-sensitive, byte-preserving directory names.
The capability probe refuses case-folding or normalization aliases before a
store root is initialized or admitted.

The names are physical adapter coordinates, not stable public handles.
`writer.lock` persists; its existence and contents prove nothing.

The writer never overwrites an immutable-pool name. It atomically hard-links
one fully synchronized staged artifact into the same-filesystem pool. If the
destination already exists, the link operation leaves it unchanged. The
writer reopens it without following links and permits idempotent reuse only
when its complete canonical bytes equal the staged bytes. Any disagreement is
unrecoverable ambiguity.

Before that link, the writer closes every writable staging handle and reopens
the synchronized artifact read-only for complete verification. No writable
handle or writable protocol path remains once the immutable-pool link exists.

An uninitialized root has no admitted generation. Initialization and the first
publication use generation 1 with an all-zero predecessor digest. A missing
head in a root containing protocol artifacts requires recovery; it is never
silently interpreted as an empty store.

## State and visibility

<!-- markdownlint-disable MD013 -->

| State | Physical evidence | Visible to a new current reader | Law |
| --- | --- | --- | --- |
| Uninitialized | no admitted head | no | not an empty published generation |
| Reusable stage | exact header and complete unsealed records | no | may be resumed only by explicit recovery |
| Truncated tail | incomplete or ambiguous staged framing | no | preserve and refuse until explicitly discarded |
| Sealed stage | complete verified seal in staging | no | immutable bytes, not yet durable pool evidence |
| Valid orphan | verified immutable artifact not selected by current head | no | preserve; neither promote nor delete silently |
| Published | selected through one verified head and catalog | yes | exact immutable reader snapshot |
| Retired | selected by an older catalog but not the current head | no for new readers | existing pinned readers may finish; no deletion in v1 |
| Corrupt sealed | seal, checksum, digest, or framing disagreement | no | refuse; never truncate or reinterpret |
| Stale generation | candidate does not extend current generation exactly | no | refuse expected/observed generation |
| Unrecoverable ambiguity | conflicting or insufficient durable evidence | no | represent uncertainty and refuse |

<!-- markdownlint-enable MD013 -->

## Writer exclusion

Version 1 is a one-writer/many-reader protocol.

The writer opens `writer.lock` without following links and acquires one
exclusive kernel advisory lock before recovery planning. It holds the lock
through publication success or typed failure.

Readers do not take the writer lock. They rely on immutable segments,
immutable catalogs, and atomic head replacement.

A lock acquisition failure reports writer busy. The writer never deletes,
renames, truncates, or replaces the lock file to break a purported stale
owner. Process death releases the kernel lock. Filesystems without proven
process-scoped exclusion are unsupported.

## Forward publication protocol

The writer starts with an expected current generation and catalog digest. It
acquires the lock, validates the current head and catalog again, and refuses
stale expectations before creating a stage. If the current verified head
already equals the complete proposed generation, catalog length, and catalog
digest, retry returns an explicit already-published receipt after
synchronizing the root directory. A different observed generation or digest
is a stale-generation refusal.

Every write handles short writes and interruption. Every flush, file sync,
hard link, unlink, head replacement, and directory sync is explicit and
fallible. An error returns no publication receipt. `Drop` performs cleanup
only and cannot publish.

### Seal each new segment

1. Create `staging/current.seg` exclusively
   (`KEEP-CRASH-001`).
2. Write the complete 64-byte header (`KEEP-CRASH-002`).
3. Append each complete record and checksum (`KEEP-CRASH-003`, with an
   occurrence counter for tests).
4. Flush the complete record prefix (`KEEP-CRASH-004`).
5. Synchronize the reusable record prefix (`KEEP-CRASH-005`).
6. Append the complete seal (`KEEP-CRASH-006`).
7. Flush the sealed bytes (`KEEP-CRASH-007`).
8. Synchronize the sealed staging file (`KEEP-CRASH-008`).
9. Reopen and verify the complete staged segment, then atomically hard-link it
   without replacement to the exact digest-derived immutable-pool name
   (`KEEP-CRASH-009`).
10. Synchronize `segments` (`KEEP-CRASH-010`).
11. Unlink `staging/current.seg` (`KEEP-CRASH-011`).
12. Synchronize `staging` (`KEEP-CRASH-012`).

After step 12 the segment is a durable valid orphan. It remains invisible
until a published catalog names it.

### Publish the catalog generation

1. Create `staging/current.cat` exclusively
   (`KEEP-CRASH-013`).
2. Write the complete canonical generation (`KEEP-CRASH-014`).
3. Flush it (`KEEP-CRASH-015`).
4. Synchronize it (`KEEP-CRASH-016`).
5. Reopen and verify it, then atomically hard-link it without replacement to
   the exact generation-and-digest immutable-pool name
   (`KEEP-CRASH-017`).
6. Synchronize `catalogs` (`KEEP-CRASH-018`).
7. Unlink `staging/current.cat` (`KEEP-CRASH-019`).
8. Synchronize `staging` (`KEEP-CRASH-020`).

After step 8 the catalog is a durable valid orphan. It remains invisible
until the publication head names it.

### Replace the publication head

1. Create `head.next` exclusively (`KEEP-CRASH-021`).
2. Write the complete 128-byte head (`KEEP-CRASH-022`).
3. Flush it (`KEEP-CRASH-023`).
4. Synchronize it (`KEEP-CRASH-024`).
5. Reopen and verify the head and its complete transitive catalog view, then
   atomically replace `HEAD` with `head.next` (`KEEP-CRASH-025`).
6. Synchronize the store root (`KEEP-CRASH-026`).

Only completion of step 6 returns a `#[must_use]` publication receipt.

The normative pre-state, interrupted-state class, post-state, and recovery
posture for every identifier are frozen in
[`transitions.tsv`](../../../conformance/segment-store/v1/transitions.tsv).

## Reader snapshot

A new reader:

1. opens `HEAD` without following links and reads exactly 128 bytes;
2. validates its complete framing and checksum;
3. opens the exact digest-derived catalog name;
4. verifies catalog length, checksum, digest, generation, predecessor
   coordinate, ordering, and duplicates;
5. verifies every referenced segment and record before admission; and
6. retains that immutable catalog generation for the snapshot lifetime.

The reader never rescans `HEAD` during one operation and never combines
catalogs. If head replacement races the initial read, atomic replacement gives
the reader either the complete old head or complete new head. Any other
observation is an unsupported-platform or corruption refusal.

Version 1 never deletes immutable artifacts. A reader holding an older
verified generation can therefore finish while a new generation is
published. Retention and GC must define later deletion safety before this can
change.

## Recovery

Opening is read-only. It returns an admitted reader snapshot, an uninitialized
state, a typed recovery inventory and plan, or a refusal. It never changes
durable bytes.

Recovery inventory reads only protocol-owned canonical names. Across the
store root, `staging`, `segments`, and `catalogs`, recovery counts entries
before retaining or sorting their names. The scan stops with a typed limit
refusal when the count exceeds `MAX_RECOVERY_INVENTORY_ENTRY_COUNT`; because
it stops on the first excess entry, the refusal reports an observed-at-least
count of `2,097,153`, not a host-order-dependent exact total. A configured
limit may be lower but never higher.

After the count is admitted, recovery sorts names by raw canonical bytes and
verifies complete content before classification. Unknown names, symlinks,
conflicting canonical coordinates, duplicate digests, multiple fixed-name
stages, or a head that cannot be proven atomic are unrecoverable ambiguity.

The required classes are:

- **Reusable staged material:** a valid header followed by zero or more
  complete records and no seal. Explicit recovery may resume it without
  rewriting the admitted prefix.
- **Valid orphan:** a completely verified sealed segment or catalog not
  selected by the current head. Recovery preserves it and reports why it is
  invisible.
- **Truncated tail:** an empty stage, partial header, partial record, partial
  checksum, partial seal, or partial catalog/head. It is never truncated,
  padded, or treated as complete automatically.
- **Corrupt sealed state:** complete-looking immutable framing whose declared
  values, checksum, digest, identity, or physical name disagree. Recovery
  refuses it.
- **Stale generation:** a candidate head or catalog that does not extend the
  verified current generation and predecessor exactly. Recovery reports
  expected and observed coordinates.
- **Unrecoverable ambiguity:** evidence permits more than one incompatible
  history or cannot prove one lawful current state. Recovery refuses without
  choosing.

An explicit recovery executor may resume a reusable stage, preserve an
orphan, explicitly discard a named truncated stage, or finalize one fully
verified next-generation head using the same generation comparison and
publication steps. Recovery itself uses the same crash points and must be
idempotent. It may not silently promote the newest artifact, truncate to the
last plausible boundary, rewrite a checksum, delete a valid orphan, or select
by timestamp.

## Deterministic refusal order

Artifact decoders validate:

1. exact minimum fixed-header availability;
2. magic, version, flags, fixed lengths, algorithms, and reserved bytes;
3. count and length bounds with checked arithmetic;
4. declared, calculated, physical-name, and actual length agreement;
5. local checksum;
6. enclosing digest;
7. kind-specific identity and payload verification;
8. ordering, duplicate, offset, and generation laws; and
9. expected head or logical identity, when supplied.

The first failed law determines the typed error. A decoder never allocates
from an untrusted count before bounds and exact total length agree.

## Platform contract

The initial adapter is supported only when it can prove:

- capability-relative no-follow access to regular files and directories;
- case-sensitive, byte-preserving directory names without path aliases;
- atomic same-filesystem no-clobber hard-link creation;
- atomic same-filesystem replacement of one regular file by another;
- file synchronization that covers required data and metadata;
- directory synchronization that makes create, link, unlink, rename, and replacement
  durable;
- process-scoped exclusive advisory locking; and
- one host with one writer.

The adapter refuses network filesystems, shared multi-host mounts, filesystem
types with unknown rename or synchronization semantics, symlinked protocol
paths, and platforms whose directory durability cannot be established.
Windows support is deferred until an adapter and crash harness prove
equivalent semantics.

The protocol cannot compensate for hardware or an operating system that
acknowledges synchronization without honoring it. Documentation and receipts
state this assumption; they do not upgrade it into proof.

## Requirement ledger

<!-- markdownlint-disable MD013 -->

| ID | Requirement | Design evidence | Status |
| --- | --- | --- | --- |
| `KEEP-STORE-001` | Segment, record, seal, catalog, entry, and head have exact versioned canonical grammars | Field tables and golden artifacts | Specified in #14 |
| `KEEP-STORE-002` | Every persistent integer is fixed-width big-endian and checked | Canonical primitives and bounds | Specified in #14 |
| `KEEP-STORE-003` | Checksums and physical digests use named domain-separated preimages | Checksum formulas and fixture oracle | Specified in #14 |
| `KEEP-STORE-004` | Sealed segments and published catalogs are immutable | State model and publication protocol | Specified in #14 |
| `KEEP-STORE-005` | Logical identity is separate from physical location | Record and catalog entry grammars | Specified in #14 |
| `KEEP-STORE-006` | Catalog ordering and duplicate refusal are deterministic | Catalog-entry rules | Specified in #14 |
| `KEEP-STORE-007` | Publication has explicit flush, sync, no-clobber link, unlink, head-replacement, and directory-sync order | `KEEP-CRASH-001`–`026` | Specified in #14 |
| `KEEP-STORE-008` | File presence alone proves no state | Core law and visibility table | Specified in #14 |
| `KEEP-STORE-009` | Writer exclusion uses one persistent kernel-managed lock | Writer-exclusion contract | Specified in #14 |
| `KEEP-STORE-010` | Readers observe one complete immutable generation | Reader-snapshot protocol | Specified in #14 |
| `KEEP-STORE-011` | Opening is observational and recovery is explicit | Recovery protocol | Specified in #14 |
| `KEEP-STORE-012` | Required recovery classes remain distinct | Recovery classification ledger | Specified in #14 |
| `KEEP-STORE-013` | Ambiguous or corrupt durable state is refused | Recovery and refusal order | Specified in #14 |
| `KEEP-STORE-014` | Memory, record, segment, and catalog sizes are bounded | Bounds table | Specified in #14 |
| `KEEP-STORE-015` | Unsupported filesystem semantics fail closed | Platform contract | Specified in #14 |
| `KEEP-STORE-016` | No Echo, Graft, Git, or application policy enters the protocol | ADR-0005 and physical namespace | Specified in #14 |
| `KEEP-STORE-017` | Catalog locations equal verified top-level segment-record spans | Catalog-entry admission | Specified in #14 |

<!-- markdownlint-enable MD013 -->

## Compatibility and migration

The byte grammars, magic values, field widths and order, endianness, kinds,
flags, algorithm coordinates, bounds, checksum domains, catalog ordering,
physical-name grammar, generation law, crash-point identifiers, publication
order, and recovery classifications are compatibility commitments.

Changing any of them requires a new store protocol version and an explicit
migration decision. A migration writes and verifies new immutable artifacts,
publishes a new compatible head under its accepted protocol, and never
reinterprets existing bytes in place.

Logical `BlobId`, `ChunkId`, and `LayoutId` remain stable when their exact
logical bytes and canonical plans remain unchanged.

## Security and privacy

Version 1 provides integrity checks, not writer authentication or
confidentiality. Logical identities, lengths, record boundaries, catalog
membership, and physical reuse are visible metadata. An attacker controlling
all bytes can recompute unkeyed checksums.

All path operations are capability-relative and no-follow. Untrusted counts
and lengths are bounded before allocation. Diagnostics retain expected and
observed typed coordinates but do not include plaintext payloads, unbounded
paths, secrets, or terminal control characters.

## Golden evidence

The
[durable segment-store corpus](../../../conformance/segment-store/v1/README.md)
contains:

- an empty sealed segment;
- a one-byte chunk segment bound to an existing independent `ChunkId`;
- catalog generation 1 naming its exact physical record;
- a publication head naming that exact catalog;
- a two-record segment carrying the chunk and its canonical flat layout;
- a two-entry catalog proving chunk-before-layout order and checked offsets;
- a publication head naming that cross-kind catalog;
- exact canonical bytes, checksums, physical digests, lengths, counts, and
  offsets; and
- the complete `KEEP-CRASH-001`–`KEEP-CRASH-026` transition ledger.

The test-only Rust oracle reconstructs every artifact directly from these
tables and formulas. Production implementations must match the frozen corpus
and add parser fuzzing, corruption mutations, crash injection, recovery tests,
and model-based generation tests in issues #15–#17.

The format-local tradeoffs are recorded in the
[colocated rationale](rationale.md).
