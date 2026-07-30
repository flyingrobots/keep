# Retention Records and Publication

This page owns retention values, root-generation records, manifests, heads,
closure admission, and publication for `keep.segment-store/v2`.

## Scalar and identity rules

All integers are unsigned and big-endian. Reserved bytes and unassigned flags
are zero. Decoders reject unknown mandatory flags, nonzero reserved bytes,
truncation, trailing bytes, unsupported versions, noncanonical ordering,
duplicates, overflow, and values above a fixed limit.

Checksums and durable digests use each record's named domain-separated
BLAKE3-256 profile, including the domain string's terminating zero byte. No
digest covers a serializer-owned value.

### Retention namespace

`RetentionNamespace` is an opaque, nonempty byte string of 1 through 255 bytes.
Every byte is admitted and canonical as-is; the value is not Unicode, a path,
an account, a process, or an application identity. No normalization, case
folding, alias, implicit namespace, or alternate encoding exists.

The namespace digest is:

```text
BLAKE3-256("keep.retention-namespace/v1\0" ||
           namespace-length-u16 ||
           namespace-bytes)
```

The 32-byte digest supplies the physical namespace-directory coordinate. The
root-generation record also stores the exact namespace bytes, so a digest
collision or substituted spelling refuses instead of aliasing two authorities.

### Generations

`RootGeneration` and `LivenessGeneration` are positive `u64` values. Generation
1 is initial. A successor is exactly the observed value plus one under checked
arithmetic. Zero and overflow refuse. An empty root set remains a new
`RootGeneration`; namespace identity and generation history are never deleted
or reused in version 2.

The maximum admitted namespace count is 4,096, including current manifest
namespaces, empty generations, and recovery-protected orphan namespace
directories; directory existence alone is not authority. Admission computes
the attempted total with checked arithmetic and refuses above that maximum
before any namespace-generation or manifest bytes are staged. Existing
namespaces may transition while the store is at capacity.

### Reconstruction anchor

One anchor is exactly 119 bytes:

| Offset | Width | Field |
| ---: | ---: | --- |
| 0 | 59 | canonical `BlobId` binary bytes |
| 59 | 60 | canonical `LayoutId` binary bytes |

Anchors are ordered by the lexicographic order of their complete canonical
bytes. The set is sorted, duplicate-free before admission. The maximum
anchor count in one namespace generation is 65,536.

### Realization profile and limits

Version 2 admits one realization profile:

- identity `1`;
- version `1`;
- canonical name `keep.retention-single-canonical-witness/v1`;
- exact witness count `1` for each layout and chunk identity; and
- selection by canonical physical catalog coordinate.

The stored profile coordinate is the `u32` identity, `u32` version, and
BLAKE3-256 digest of its canonical definition bytes. Any unknown or mismatched
coordinate refuses. A future profile requires a successor specification.

Each root generation stores caller-selected limits no greater than these
implementation ceilings:

| Limit | Ceiling |
| --- | ---: |
| anchors | 65,536 |
| closure nodes | 1,048,576 |
| closure depth | 8 |
| encoded bytes inspected | 16,777,216 |
| physical bytes inspected | 1,073,741,824 |

All limits are positive. Cross-field validation and the ceiling check complete
before traversal or materialization.

## Root-generation record

One root-generation file is:

```text
192-byte fixed-width header
namespace bytes
anchor-count × 119-byte anchors
32-byte root digest
32-byte checksum
```

Its total maximum length is 7,799,295 bytes.

<!-- markdownlint-disable MD013 -->

| Offset | Width | Field | Canonical value |
| ---: | ---: | --- | --- |
| 0 | 16 | magic | `KEEP:RET:ROOT2\0\0` |
| 16 | 2 | version | `2` |
| 18 | 2 | header length | `192` |
| 20 | 4 | flags | `0` |
| 24 | 8 | total record length | derived exact length |
| 32 | 8 | root generation | positive |
| 40 | 2 | namespace length | `1..=255` |
| 42 | 2 | anchor width | `119` |
| 44 | 4 | anchor count | `0..=65,536` |
| 48 | 4 | profile identity | `1` |
| 52 | 4 | profile version | `1` |
| 56 | 32 | profile-definition digest | registered exact digest |
| 88 | 8 | closure-node limit | positive and at most ceiling |
| 96 | 2 | closure-depth limit | positive and at most ceiling |
| 98 | 2 | reserved | zero |
| 100 | 8 | encoded-byte limit | positive and at most ceiling |
| 108 | 8 | physical-byte limit | positive and at most ceiling |
| 116 | 32 | predecessor root digest | zero for generation 1 |
| 148 | 32 | anchor-set digest | exact body-anchor digest |
| 180 | 12 | reserved | zero |

<!-- markdownlint-enable MD013 -->

The anchor-set digest is:

```text
BLAKE3-256("keep.retention-anchor-set/v2\0" ||
           anchor-count-u32 ||
           canonical-anchor-bytes)
```

The root digest covers the header and body:

```text
BLAKE3-256("keep.retention-root/v2\0" || header || body)
```

The checksum covers the header, body, and root digest:

```text
BLAKE3-256("keep.retention-root-checksum/v2\0" ||
           header || body || root-digest)
```

The pool coordinate is:

```text
retention/roots/<namespace-digest>/
  <root-generation-16-lower-hex>-<root-digest-64-lower-hex>.root
```

Names with alternate width, case, suffix, generation, or digest refuse.

## Global retention manifest

One manifest binds every admitted namespace to its exact root generation and
canonical digest:

```text
160-byte fixed-width header
entry-count × 72-byte entries
32-byte manifest digest
32-byte checksum
```

Each entry is:

| Offset | Width | Field |
| ---: | ---: | --- |
| 0 | 32 | namespace digest |
| 32 | 8 | root generation |
| 40 | 32 | root digest |

Entries are sorted by namespace digest and duplicate-free. The maximum entry
count is 4,096 and the maximum manifest length is 295,136 bytes.

<!-- markdownlint-disable MD013 -->

| Offset | Width | Field | Canonical value |
| ---: | ---: | --- | --- |
| 0 | 16 | magic | `KEEP:RET:LIVE2\0\0` |
| 16 | 2 | version | `2` |
| 18 | 2 | header length | `160` |
| 20 | 4 | flags | `0` |
| 24 | 8 | total record length | derived exact length |
| 32 | 8 | liveness generation | positive |
| 40 | 2 | entry width | `72` |
| 42 | 2 | reserved | zero |
| 44 | 4 | entry count | `0..=4,096` |
| 48 | 32 | predecessor manifest digest | zero for generation 1 |
| 80 | 32 | entry-set digest | exact canonical entries |
| 112 | 48 | reserved | zero |

<!-- markdownlint-enable MD013 -->

The entry-set, manifest, and checksum domains are respectively:

```text
keep.retention-manifest-entries/v2\0
keep.retention-manifest/v2\0
keep.retention-manifest-checksum/v2\0
```

The manifest pool coordinate is:

```text
retention/manifests/
  <liveness-generation-16-lower-hex>-<manifest-digest-64-lower-hex>.manifest
```

## Retention head

`retention/HEAD` and `retention/head.next` use one exact 144-byte record:

| Offset | Width | Field | Canonical value |
| ---: | ---: | --- | --- |
| 0 | 16 | magic | `KEEP:RET:HEAD2\0\0` |
| 16 | 2 | version | `2` |
| 18 | 2 | record length | `144` |
| 20 | 4 | flags | `0` |
| 24 | 8 | liveness generation | positive |
| 32 | 8 | manifest length | exact admitted length |
| 40 | 32 | manifest digest | exact pool digest |
| 72 | 32 | predecessor manifest digest | zero for generation 1 |
| 104 | 8 | reserved | zero |
| 112 | 32 | checksum | BLAKE3-256 over bytes `0..112` |

The checksum domain is `keep.retention-head-checksum/v2\0`.

## Closure admission

Before publication, Keep pins one completely verified catalog generation and
derives the complete closure for every anchor:

1. Resolve and admit the exact layout record named by `LayoutId`.
2. Require its embedded `BlobId` to equal the anchor `BlobId`.
3. Resolve and admit every ordered chunk identity required by that layout.
4. Verify each physical record, identity, checksum, digest, and catalog
   coordinate under the stored realization profile.
5. Enforce the stored limits with checked counters and a visited set.
6. Reconstruct and authenticate the complete blob identity.

A missing or corrupt closure member, ambiguous catalog claim, unsupported
profile, limit breach, cycle, unknown mandatory edge, identity mismatch, or
ordering error refuses the entire transition. Keep never omits one failed
member and continues with a smaller live set.

Version-2 catalog publication holds the same writer authority and proves every
current retained closure against its candidate catalog before replacing the
catalog `HEAD`.

## Generation transition

A transition supplies a namespace, an expected state of absent or one exact
`RootGeneration`, a complete canonical anchor set, the exact realization
profile coordinate, and admitted limits.

Under exclusive writer authority, publication:

1. completes recovery of every fixed retention stage;
2. admits the current retention head, manifest, and selected namespace root;
3. compares expected and observed generations;
4. verifies the candidate closure against one pinned catalog;
5. writes and synchronizes `retention/root.next`;
6. for a new namespace, exclusively creates and verifies its exact digest-named
   directory, then synchronizes `retention/roots`;
7. links and verifies the root pool entry, then synchronizes its directory;
8. writes and synchronizes `retention/manifest.next`;
9. links and verifies the manifest pool entry and synchronizes its directory;
10. writes and synchronizes `retention/head.next`;
11. atomically replaces `retention/HEAD` and synchronizes `retention`;
    `root.next` and `manifest.next` remain durable until the retention head
    commits, then are removed and `retention` is synchronized again; and
12. returns a consequential `#[must_use]` receipt.

The receipt binds the namespace, expected and observed generations, committed
root generation and digest, global manifest generation and digest, profile
coordinate, anchor-set and closure digests, catalog generation and digest, and
every durable publication outcome.

A stale transition preserves expected and observed generations. A
byte-identical retry returns **already committed** only while that exact root
successor remains current; otherwise it returns the precise stale state.

A reader holds one shared `ReaderFence` and double-collects the catalog and
retention heads around complete transitive admission. It accepts only the same
coordinates before and after for both heads. Any generation, length, digest, or
checksum change discards the view and retries within a bounded attempt limit;
exhaustion refuses. The accepted view observes one complete root generation for
its snapshot lifetime.
