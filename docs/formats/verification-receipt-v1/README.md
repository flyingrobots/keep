# Verification Receipt Version 1

This page owns the canonical `keep.verification-receipt/v1` record: the
durable, replayable projection of one `VerificationReport` or
`VerificationRefusal` onto one view. A receipt states what one verification
established or refused, against which subject, view, layout, and target, at
which depth or stage, and under which classification. It contains no
plaintext, key material, or path. Keep does not persist receipts; the
application that asked for the verification does.

The [verification invariant](../../invariants/verification/README.md) owns
the vocabulary the receipt encodes; the
[golden corpus](../../../conformance/verification-receipt/v1/README.md) owns
the frozen bytes.

## Core law

A receipt is admitted only as the exact canonical encoding of one receipt
whose fields obey every semantic law below. `CanonicalVerificationReceipt::encode`
is total and deterministic; `decode` admits framing, the verification
contract version, the checksum, every registered code, every identity slot,
and every semantic law, then requires the bytes to equal the canonical
re-encoding of what it decoded. A report and a refusal are different
outcomes: flipping the outcome kind leaves the other fields contradicting it,
so neither direction is admitted.

## Record

Every integer is unsigned, big-endian, and fixed-width. The record is exactly
384 bytes.

<!-- markdownlint-disable MD013 -->

| Offset | Width | Field | Canonical value |
| ---: | ---: | --- | --- |
| 0 | 16 | magic | `KEEP:VERIFY:RCPT` |
| 16 | 2 | version | `1` |
| 18 | 2 | record length | `384` |
| 20 | 4 | flags | `0` |
| 24 | 4 | verification contract version | `1` |
| 28 | 2 | outcome kind | `report:1`, `refusal:2` |
| 30 | 2 | depth | registered depth code: established (report) or the stage refused; the requested depth for `unsupported` |
| 32 | 2 | subject kind | `blob:1`, `layout:2` |
| 34 | 2 | view kind | `reference:1`, `durable:2` |
| 36 | 2 | refusal class | `none:0`, `missing:1`, `corrupt:2`, `ambiguous:3`, `unsupported:4` |
| 38 | 2 | evidence kind | `none:0`, then per class below |
| 40 | 2 | layout present | `0` or `1` |
| 42 | 2 | supported minimum depth | registered depth code for `unsupported`, else `0` |
| 44 | 2 | supported maximum depth | registered depth code for `unsupported`, else `0` |
| 46 | 2 | target present | `0` or `1` |
| 48 | 60 | subject identity slot | canonical `BlobId` binary (59 bytes, one zero pad byte) or `LayoutId` binary (60 bytes) |
| 108 | 60 | layout slot | canonical `LayoutId` binary when present, else zero |
| 168 | 60 | target slot | canonical `BlobId` binary plus one zero byte when present, else zero |
| 228 | 8 | catalog generation | positive for a durable view, `0` for the reference view |
| 236 | 32 | catalog digest | exact for a durable view, zero for the reference view |
| 268 | 8 | liveness generation | positive when the durable view has published retention, else `0` |
| 276 | 32 | manifest digest | exact when the liveness generation is positive, else zero |
| 308 | 8 | evidence index | zero-based chunk or boundary index for an indexed evidence kind, else `0` |
| 316 | 8 | chunks verified | the report's count; `0` for a refusal |
| 324 | 28 | reserved | zero |
| 352 | 32 | checksum | BLAKE3-256 under `keep.verification-receipt-checksum/v1\0` over bytes `0..352` |

<!-- markdownlint-enable MD013 -->

Depth codes are the one-based positions in `VerificationDepth::ALL`:
`framing:1`, `checksum:2`, `chunk-identity:3`, `layout-identity:4`,
`complete-blob-identity:5`, `catalog-reachability:6`,
`retention-closure:7`.

Evidence kinds for `missing`: `blob:1` (no committed layout names the
blob), `layout:2` (the exact layout is not committed), `chunk:3` (the
present layout names a chunk the view lacks; indexed). For `corrupt`:
`chunk-identity:1` (indexed), `layout-identity:2`, `blob-identity:3`,
`profile-boundary:4` (indexed). The receipt keeps the kind, the layout, and
the index; the expected and observed identities stay in the ephemeral
refusal.

## Semantic laws

- A report carries refusal class `none`, evidence kind `none`, evidence
  index `0`, and a zero supported range; it names its layout and target; a
  blob subject's target is the subject and a layout subject's layout is the
  subject.
- A refusal names no target and verified no chunks.
- `missing` and `corrupt` carry a zero supported range; `missing` evidence
  `blob` and `layout` name no layout and no index, `chunk` names both;
  every `corrupt` evidence names its layout, and only the indexed kinds
  carry an index.
- `ambiguous` names no evidence, layout, or index.
- `unsupported` names no evidence or layout; its supported range is
  registered and ordered, and the requested depth lies outside it.
- The reference view binds no generation or digest; a durable view binds a
  positive catalog generation, and a zero liveness generation binds a zero
  manifest digest.
- An absent layout or target slot is zero; a blob slot's pad byte is zero.

## Deterministic refusal order

Length; magic; version; record length; flags; contract version; checksum;
outcome kind; then the remaining registered codes in field order; reserved
bytes; the depth code; the subject slot; the view; the layout slot; then
the outcome's semantic laws. `tests/verification_receipt.rs` holds one
mutation per structural field to this order.

## Evidence

`tests/verification_receipt.rs`: the three golden fixtures match a
handwritten oracle and the production encoder; each decodes from the
fixture file as from another process and re-encodes canonically; every
reference-store report and refusal at every depth projects and round-trips,
as does a `Missing` refusal projected onto the frozen durable view; every
structural field has one exact first refusal; a refusal never decodes as a
report and a report never as a refusal. The `verification_receipt` fuzz
target is seeded from the corpus. Requirement `KEEP-VERIFY-007`.

## Nonclaims

A receipt proves that one verification reported or refused as stated, not
that the view still holds, that the store is durable, or that a later
verification agrees. It carries identities, generations, and digests only;
it reveals no content. Durable views that produce `Framing`, `Checksum`,
`CatalogReachability`, and `RetentionClosure` reports are `KEEP-VERIFY-006`,
planned with the durable read surface.
