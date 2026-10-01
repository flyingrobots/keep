# Closure Corruption Boundary

- Status: Normative version-2 protocol; executable ingress evidence implemented
- Format coordinate: `keep.segment-store/v2`
- Requirement: [`KEEP-RETENTION-005`](requirements.md#retention-transitions)
- Parent contract: [Closure verification](closure.md)

This page defines where corrupt closure-member bytes refuse. Its primary job is
to keep untrusted byte admission separate from deterministic closure traversal.

## Trust boundary

`verify_retention_closure` accepts a validated `RetentionRoot` and an immutable
`CatalogSnapshot`. It does not accept untrusted bytes, raw segment records,
paths, readers, or caller lookup callbacks.

A record reaches that snapshot only through this proof chain:

1. `ChecksummedSegmentRecord::decode` admits exact framing and its checksum.
2. `ChecksummedSegmentRecord::admit` recomputes the chunk or layout identity
   from the payload and returns an `AdmittedSegmentRecord`.
3. `AdmittedSegment::decode` admits every complete nested record and the
   segment seal and digest.
4. `ChecksummedCatalog::admit` binds each catalog entry to the exact admitted
   record identity, checksum, and top-level location.
5. `ChecksummedPublicationHead::admit` binds the admitted catalog's generation,
   length, and digest into a `CatalogSnapshot`.

Failure at any step makes the next type unconstructible through the public API.
Closure verification therefore has no corruption fallback and never
reinterprets a malformed record as a missing member.

## Exact refusal ownership

The inherited version-1 boundaries retain their typed errors:

- malformed record framing or checksum returns `SegmentRecordDecodeError`;
- chunk payload identity disagreement or malformed layout payload returns
  `SegmentRecordAdmissionError`;
- complete-segment corruption returns `SegmentReadError`;
- catalog location, identity, checksum, or segment disagreement returns
  `CatalogAdmissionError`; and
- publication-head disagreement returns `CatalogSnapshotError`.

`RetentionClosureVerificationError::MissingMember` means the pinned,
fully admitted catalog has no binding for the scheduled logical identity. It
does not mean bytes were present but corrupt.

## Re-verification under authority

A preparation carries a closure verified against a `CatalogSnapshot` the
caller supplied, which may have been read by another process, from another
store, or before a pool entry changed. Filesystem publication therefore does
not trust it. After binding this store's catalog head and the catalog it
selects, current-state verification loads that catalog together with every
segment it names, bounded by the authority's `CatalogRestartPolicy`, admits
each record through the proof chain above, re-runs `verify_retention_closure`
against the freshly admitted snapshot, and requires the same closure digest.

Refusal keeps the original error. A member whose record framing, checksum,
chunk identity, or layout payload refuses surfaces as
`RetentionCurrentStateRefusal::ClosureMemberRefused` whose `source` is the
`CatalogRestartError` that carries the `SegmentReadError` and, beneath it, the
exact `SegmentRecordDecodeError` or `SegmentRecordAdmissionError`; a closure
that no longer verifies surfaces as `ClosureReverificationRefused` with the
`RetentionClosureVerificationError`; a closure that verifies to a different
digest surfaces as `ClosureDigestChanged`. No layer replaces a typed error
with a string, so a caller can walk `source()` from the publication error to
the record that failed. Every refusal happens before any retention stage is
written.

## Executable evidence

- The [closure-member re-verification
  laws](../../../src/adapters/retention/filesystem_retention_member_tests.rs)
  corrupt a chunk payload, corrupt a layout payload, and remove a member
  segment in a migrated store, then prove publication refuses with the exact
  admission or restart error reachable through `source()` and leaves the
  retention namespace untouched.

- The [segment-record framing
  laws](../../../tests/segment_record/framing_laws.rs) cover checksum and
  framing corruption.
- The [segment-record admission
  laws](../../../tests/segment_record/admission_laws.rs) cover content-valid
  checksums whose chunk or layout payload does not match its declared identity.
- The [segment corruption-localization
  laws](../../../tests/segment/identity_laws.rs) prove record refusal precedes
  the outer segment digest.
- The [`segment_format` fuzz
  target](../../../fuzz/fuzz_targets/segment_format.rs) reaches record decoding,
  record admission, and complete-segment admission from deterministic canonical
  seeds owned by the [Rust seed-corpus
  task](../../../xtask/src/fuzz_seed_corpus/segment_seeds.rs).

These proofs establish ingress safety. The filesystem retention authority
also re-verifies every closure member before publication and preserves the
original admission error through its operation-level refusal.
`filesystem_retention_member_tests` downcasts that source to the exact
`SegmentRecordAdmissionError`; this is executable evidence for
`KEEP-RETENTION-005`.
