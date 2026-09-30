# Verification Receipt Version 1 Corpus

This directory freezes canonical `keep.verification-receipt/v1` records: the
replayable projection of one verification report or refusal onto one view.
The format is specified in
[`docs/formats/verification-receipt-v1/`](../../../docs/formats/verification-receipt-v1/README.md).

## Corpus files

| File | Contents |
| --- | --- |
| `artifacts.tsv` | Case, kind, exact byte length, trailing checksum, and fixture per receipt |
| `reference-complete-blob-report.hex` | A report: the one-zero blob established at `CompleteBlobIdentity` against the reference view through its canonical layout |
| `durable-corrupt-chunk-refusal.hex` | A refusal: chunk 0 of the one-zero layout corrupt at `ChunkIdentity` against the frozen durable view (catalog generation 2, retention generation 1) |
| `reference-unsupported-framing-refusal.hex` | A refusal: `Framing` requested of the reference view, which supports `ChunkIdentity` through `CompleteBlobIdentity` |
| `ORIGIN.md` | Construction provenance and verification boundary |

Hexadecimal fixture files contain one lowercase hexadecimal encoding of the
complete 384-byte record followed by exactly one LF.

## Frozen identities

The subject, layout, and target slots carry the canonical one-zero `BlobId`
and `LayoutId` binaries from
[`conformance/layout/v1/layouts.tsv`](../../layout/v1/layouts.tsv). The
durable view binds the generation-two catalog digest at byte 320 of
[`one-zero-catalog-generation-two.hex`](../../segment-store/v1/one-zero-catalog-generation-two.hex)
and the generation-one manifest digest from
[`segment-store/v2/artifacts.tsv`](../../segment-store/v2/artifacts.tsv).

## Verification

`tests/verification_receipt.rs` reconstructs every fixture from a
handwritten oracle over those inputs, requires the production encoder to
reproduce it byte for byte, decodes each fixture as another process would,
and holds every structural field to one exact first refusal.
`verification_receipt_conformance_contract` in `xtask` admits this
directory's shape and the artifact table. A fixture is never regenerated to
make a production implementation pass.
