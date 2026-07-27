# Flat Chunk Layout Corpus v1

This directory is the implementation-independent corpus for
`keep.flat-chunks/v1`. It freezes canonical record bytes, `LayoutId`
coordinates, and mutation expectations independently of the production
encoder and decoder.

Issue [#10](https://github.com/flyingrobots/keep/issues/10) owns the
production codec, corruption tests, property tests, and continuous fuzz
target. Issue [#13](https://github.com/flyingrobots/keep/issues/13) owns the
public verification boundary that compares actual chunk bytes, the complete
blob identity, and the replayed registered storage profile. The decoder still
admits all three verification-phase mutations because structural admission
does not possess chunk bytes.

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
- `invalid-layout-id-text.tsv` supplies hexadecimal input bytes and exact
  refusal classes for malformed or unsupported text coordinates.
- `invalid-layout-id-binary.tsv` supplies exact byte mutations and refusal
  classes for malformed or mismatched binary coordinates.
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

Expected outcomes follow the specification's first-failure order. Inserting a
duplicate flags-width field at offset 24 shifts the canonical header and makes
`header_length` the first invalid field. Swapping the middle entries of
`zeros-long` creates a gap at the first swapped position before the later
offset decrease is observed.

Mutations of opaque identity digests are not all structural errors. A
rechecksummed `BlobId` digest or `ChunkId` digest can form another structurally
valid layout. Those cases remain admitted until verified content proves
`layout.reconstruction-mismatch` or `layout.chunk-mismatch`. The ledger names
that later phase explicitly, and the reference CAS now enforces it.

The `profile-boundary-mismatch` mutation replaces the natural 262,144-byte
hard-maximum boundary for 262,145 zero bytes with structurally valid
262,143-byte and 2-byte chunks. Both replacement `ChunkId` values and the
target `BlobId` name the exact source bytes. Verification MUST refuse the plan
because replaying `fastcdc-64k-v1` emits the original boundary.

## `LayoutId` coordinate mutations

Issue #10 applies every row in `invalid-layout-id-binary.tsv` to the named
case's `layout_id_binary_hex` value. Operations have semantics identical to the
plan mutation protocol. Wider replacements deliberately distinguish an
out-of-bounds plan length, an in-bounds but incongruent length, and a different
valid record length that mismatches the named record.

`invalid-layout-id-text.tsv` stores raw input as hexadecimal so empty input,
whitespace, and other noncanonical bytes remain unambiguous in a
line-oriented table. The production codecs MUST assert the exact typed
outcomes in both files rather than only generic failure.
