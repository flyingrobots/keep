# Flat Chunk Layout Corpus v1

This directory is the implementation-independent design corpus for
`keep.flat-chunks/v1`. It freezes canonical record bytes, `LayoutId`
coordinates, and mutation expectations without supplying a production encoder
or decoder.

Issue [#10](https://github.com/flyingrobots/keep/issues/10) owns the
implementation, independent checker, corruption tests, property tests, and
continuous fuzz target. Until that issue lands, these fixtures are protocol
evidence, not evidence that Keep can ingest, decode, admit, or reconstruct a
layout.

The normative format is the
[Flat Chunk Layout v1 specification](../../../docs/formats/flat-chunk-layout-v1/README.md).

## Files

- `layouts.tsv` binds each case to source bytes, `BlobId`, record length,
  checksum, `LayoutId`, and exact record fixture.
- `entries.tsv` lists every entry's index, logical offset, exact length, and
  version-1 `ChunkId` digest.
- `*.layout.hex` contains one lowercase hexadecimal encoding of the complete
  canonical binary record followed by exactly one LF. Hex is fixture transport;
  decoded bytes are the durable record.
- `mutations.tsv` defines structural and verification mutations for issue #10.
- `ORIGIN.md` records independent construction and review provenance.

All TSV files are UTF-8, tab-delimited, and terminated by exactly one final LF.
Fields use canonical unsigned decimal unless their name ends in `_hex`. `-`
means inapplicable; it is not an empty field.

## Golden cases

The four records exercise distinct laws:

- `empty` binds the empty `BlobId` and contains zero entries;
- `one-zero` binds one byte to one one-byte `ChunkId`;
- `max-plus-one-zeros` binds 262,145 zero bytes to one hard-maximum chunk and
  one final one-byte runt; and
- `zeros-long` binds four repeated hard-maximum chunks, proving that repeated
  `ChunkId` values are lawful at distinct logical offsets.

The chunk digests are existing independent witnesses from
`conformance/chunk-id/v1/identities.tsv`. The registered profile identity is
the accepted ADR-0003 witness.

## Mutation protocol

`mutations.tsv` uses these operations:

- `replace-v1` replaces exactly `span_length` bytes with the same number of
  decoded `parameter` hexadecimal bytes;
- `xor-v1` XORs exactly `span_length` bytes with equal-length decoded
  `parameter` hexadecimal bytes;
- `insert-v1` inserts decoded `parameter` bytes before `offset`;
- `delete-v1` removes exactly `span_length` bytes and requires parameter `-`;
  and
- `swap-v1` swaps the span at `offset` with the equal-length span whose
  canonical decimal offset is `parameter`.

`recompute-v1` recalculates the record checksum after a same-length mutation so
the targeted semantic law, rather than checksum corruption, decides the
result. `preserve-v1` leaves every existing checksum byte unchanged.

Mutations of opaque identity digests are not all structural errors. A
rechecksummed `BlobId` digest or `ChunkId` digest can form another structurally
valid layout. Those cases remain admitted until verified content proves
`layout.reconstruction-mismatch` or `layout.chunk-mismatch`. The ledger names
that later phase explicitly.

## `LayoutId` coordinate mutations

Issue #10 also derives these mutations from every `layout_id_binary_hex` value
in `layouts.tsv`:

| Mutation | Offset | Expected refusal |
| --- | ---: | --- |
| Wrong identity magic | 0 | `layout-id.wrong-magic` |
| Unsupported identity version | 16 | `layout-id.unsupported-version` |
| Unsupported layout codec | 18 | `layout-id.unsupported-codec` |
| Plan-length mismatch | 20 | `layout-id.plan-length-mismatch` when compared with a record |
| Digest mismatch | 28 | `layout-id.mismatch` when compared with a record |
| Truncation | 59 | `layout-id.wrong-length` |
| Trailing byte | 60 | `layout-id.wrong-length` |

Every mutation is one bit unless a wider replacement is necessary to target a
bound. The production decoder MUST assert exact typed outcomes rather than
only generic failure.
