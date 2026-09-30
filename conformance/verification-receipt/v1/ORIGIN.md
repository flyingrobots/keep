# Verification Receipt Corpus Origin

The corpus was constructed on 2026-09-30 with
`rustc 1.98.1 (48a229cea 2026-09-01)` and `cargo 1.98.1`.

## Independent inputs

The oracle in `tests/verification_receipt/oracle.rs` imports exact bytes
only from previously accepted fixtures:

- the one-zero `BlobId` (59 bytes) and `LayoutId` (60 bytes) binaries
  already transcribed from `conformance/layout/v1/layouts.tsv` by the
  version-2 segment-store oracle;
- the generation-two catalog digest at byte 320 of
  `conformance/segment-store/v1/one-zero-catalog-generation-two.hex`;
- the generation-one manifest digest from the `one-root-manifest` row of
  `conformance/segment-store/v2/artifacts.tsv`.

It assembles each record from the field table in
`docs/formats/verification-receipt-v1/README.md` and computes the trailing
checksum as BLAKE3-256 under `keep.verification-receipt-checksum/v1\0` over
bytes 0 through 351. It calls no production encoder.

## Materialization boundary

A temporary ignored test wrote the three hexadecimal fixtures and
`artifacts.tsv` from the oracle and was removed immediately after. The
committed oracle is read-only and rejects drift; the production
`CanonicalVerificationReceipt::encode` is required to reproduce every
fixture, not the other way round.
