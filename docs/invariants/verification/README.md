# Verification Reports

This page defines what a Keep verification report proves and what a
verification refusal evidences. The public non-durable `ReferenceStore`
implements the reference form. Durable views implement the same vocabulary
as their surfaces land.

The [rationale](rationale.md) records the governed decisions. The
[requirement ledger](requirements.md) maps each law to its evidence.

## Invariant

A verification names one subject and one requested depth. It returns exactly
one of:

1. a `VerificationReport` that establishes exactly the requested depth;
2. a `VerificationRefusal` that evidences, from a complete view, why the
   requested depth cannot hold;
3. an operational failure from which no content conclusion follows.

A report never states a deeper depth than was requested, and a view that
cannot establish a depth refuses it instead of reporting a shallower one.
Verification never repairs, substitutes, quarantines, or rewrites physical
state.

## Depths

`VerificationDepth` is an ordered enumeration, never a set of boolean flags:

| Depth | Establishes |
| --- | --- |
| `Framing` | every durable record the subject depends on has canonical framing |
| `Checksum` | every such record has a matching checksum |
| `ChunkIdentity` | every chunk the subject names is present and hashes to its `ChunkId` |
| `LayoutIdentity` | the subject's layout produces its canonical `LayoutId` |
| `CompleteBlobIdentity` | the authenticated chunks reproduce the target `BlobId` and replay the registered storage profile |
| `CatalogReachability` | one admitted catalog generation names every record the subject needs |
| `RetentionClosure` | one retained root's closure reaches the subject under one fenced view |

Establishing a depth requires every shallower depth the view supports. A
refusal names the `stage` Keep was establishing when it stopped, and a
lower-stage refusal is always reported before a deeper one.

## Subjects and reports

`VerificationSubject::Blob` verifies through the view's deterministic layout
choice; `VerificationSubject::Layout` verifies one exact committed layout.
`ReferenceStore::verify_admitted_layout` verifies a caller-supplied layout
whose canonical identity becomes the subject.

A `VerificationReport` binds the subject, the depth established, the exact
`LayoutId` it was established through, that layout's target `BlobId`, and
the number of chunks authenticated. Its fields are private and it has no
method that raises its depth.

A report proves nothing beyond its depth: not durability, not retention, not
application meaning, and not that a later verification will agree.

## Refusals

`VerificationRefusal` keeps three kinds of evidence distinct:

- `Missing`: required evidence is absent from a complete view, with
  `MissingEvidence` naming the blob, layout, or exact chunk;
- `Corrupt`: present evidence contradicts the identity it must reproduce,
  with `CorruptionEvidence` carrying the expected and observed `ChunkId`,
  `LayoutId`, or `BlobId`, or the boundary index at which profile replay
  diverged;
- `Ambiguous`: two pieces of admitted evidence conflict, so neither a
  positive nor a negative conclusion follows.

`Unsupported` is the fourth variant: the view cannot establish the requested
depth at all, and says which depths it can.

Absence is evidence only against a complete view. The reference store's
in-memory indexes are complete by construction. A durable view must bind a
complete admitted catalog before it may report `Missing`; until then an
unreadable index is an operational failure, not a refusal.

## Reference store

`ReferenceStore::verify` supports `ChunkIdentity` through
`CompleteBlobIdentity`. It holds no durable framing or checksums, no catalog,
and no retention, so every other depth is refused as `Unsupported`.

Work and memory are bounded by the layout: every chunk is read and hashed
once in a single pass; `LayoutIdentity` and deeper materialize one canonical
layout record bounded by the layout's entry limit; `CompleteBlobIdentity`
replays the registered storage profile with the detector's fixed state. The
view allocates no other adapter-owned memory.

No reference-store path produces `Ambiguous`.

## Receipts

`VerificationReceipt::from_report` and `from_refusal` project a report or
refusal onto a `VerificationView` (the reference store, or one durable
snapshot's catalog and retention coordinates), keeping the subject, depth or
stage, classification, evidence kind and index, and the exact layout and
target, and dropping the expected and observed identities.
`CanonicalVerificationReceipt::{encode, decode}` is the durable, replayable
384-byte form specified on
[the format page](../../formats/verification-receipt-v1/README.md); a
receipt written by one process is admitted by another exactly as meant.

## Nonclaims

A report contains no plaintext, key material, or path. It is an ephemeral
statement about one operation against one view; its receipt is the durable
form and proves no more than the report did.
