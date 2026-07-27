# Flat Chunk Layout Corpus Origin

## Provenance

- Created: 2026-07-27
- Governing specification:
  `docs/formats/flat-chunk-layout-v1/README.md`
- Layout identity envelope:
  `docs/adr/0002-separate-identity-from-physical-storage.md`
- Blob witnesses:
  `conformance/golden-file-worldline/v1/identities.tsv`
- Chunk witnesses: `conformance/chunk-id/v1/identities.tsv`
- Storage-profile witness: `conformance/cdc-profile/v1/profile.tsv`
- Independent digest executable: `b3sum` 1.8.5
- Byte-assembly runtime: Node.js 24.18.0 `Buffer` fixed-width big-endian
  operations

## Construction

The fixtures were constructed before any production layout encoder or decoder
existed.

The one-off construction process:

1. assembled the 144-byte header from literal magic bytes and fixed-width
   unsigned big-endian integers;
2. copied existing canonical binary `BlobId` witnesses or independently
   calculated the typed ADR-0001 preimage for the deterministic source;
3. copied the accepted `StorageProfileId` digest;
4. copied independently checked version-1 `ChunkId` lengths and digests;
5. assembled each 44-byte entry in logical order;
6. sent the exact typed checksum preimage to the external `b3sum` executable;
7. appended the resulting raw 32-byte checksum;
8. sent the exact ADR-0002 `LayoutId` preimage to `b3sum`; and
9. rendered the final record bytes as lowercase hex for review.

Node.js assembled bytes but did not supply a BLAKE3 implementation. `b3sum`
was the only digest oracle. No Keep production type, serializer, encoder,
decoder, or chunk detector generated expected layout bytes.

The `profile-boundary-mismatch` mutation's replacement `ChunkId` digests were
independently calculated from 262,143 and 2 zero bytes through the same typed
version-1 preimage and external `b3sum` oracle.

## Review controls

The checked-in tables expose every source recipe, identity, entry coordinate,
record length, checksum, and layout identity needed for an independent
implementation to reproduce the fixtures.

The issue #10 implementation satisfies the corpus review controls through an
independent field checker and companion conformance tests that:

- decode the hex fixtures without using the production layout decoder;
- reconstruct every field at its documented offset;
- recompute both checksum and `LayoutId` through an independently admitted
  BLAKE3 capability;
- execute every text and binary `LayoutId` refusal vector;
- apply every mutation in `mutations.tsv`; and
- cross-check production encoding only after the independent values pass.

Regenerating a fixture because production output differs is forbidden. Resolve
the disagreement against the specification and independent digest oracle.
