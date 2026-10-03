# Verification reports

Status: durable verification candidate under [#114](https://github.com/flyingrobots/keep/issues/114); final acceptance is tracked in the [closure ledger](../../audits/114-durable-verification-scope.md).

## Contract

A report names the exact subject, the caller's requested depth and the evidence established for that subject.

An ordinal depth comparison grants no inference about another subject: a catalog membership check cannot certify a blob, and a layout identity cannot certify its missing chunks.

Report fields and construction are private; callers can inspect or copy established evidence but cannot construct or deepen it.

Reports contain no plaintext, keys or filesystem paths and convey no publication, retention authority, reader fence or assurance that physical bytes still exist later.

## Current subject and depth matrix

| Entry point | Subject | Supported depths | Scope of the proof |
| --- | --- | --- | --- |
| `AdmittedSegment::verify` | Exact physical segment digest | `Framing`, `Checksum` | The admitted immutable segment's physical representation; logical claims require record-specific reports. |
| `AdmittedSegmentRecord::verify` for a chunk | Exact `ChunkId` | `Framing`, `Checksum`, `ChunkIdentity` | That record's complete chunk bytes; no blob or profile-boundary claim. |
| `AdmittedSegmentRecord::verify` for a layout | Exact `LayoutId` | `Framing`, `Checksum`, `LayoutIdentity` | Canonical layout bytes and identity; no requirement that referenced chunks exist. |
| `CatalogSnapshot::verify_blob` | Named logical blob in one catalog | `Framing`, `Checksum`, `ChunkIdentity`, `LayoutIdentity`, `CompleteBlobIdentity` | Canonical layout discovery; chunk presence at chunk depth; full profile replay and logical hash at complete depth. |
| `AdmittedRetentionRoot::verify` | Exact namespace, root generation and root digest | `Framing`, `Checksum`, `RetentionClosure` | Closure depth checks every anchor against the supplied catalog and its admitted limits. |
| `CatalogSnapshot::verify` | Selected catalog generation and digest | `Framing`, `Checksum`, `CatalogReachability` | Exact catalog-to-record bindings; no complete logical or retained closure claim. |

Every other depth returns `VerificationRefusal::Unsupported` (wrapped by `VerificationError` for blob/root operations) with the exact subject, request and supported set; the operation neither downgrades the request nor returns a success report.

`SnapshotBinding` remains unsupported until its separate protocol exists; catalog/retention coordinates must not be mislabeled as that future proof.

## Costs and admission boundary

Reporting physical segment, logical record and catalog evidence is constant time and allocation-free; shallow root reporting has the same costs. Blob discovery decodes catalogued layouts in canonical identity order, retaining at most one decoded layout at a time. Chunk verification looks up every referenced member; complete blob verification additionally streams every selected byte through profile replay and complete identity calculation. Retention closure uses its existing checked limits, an ordered member index bounded by the root node limit, and one decoded layout at a time. These operations perform no I/O, mutate no persistent bytes, and synchronize nothing.

These costs exclude prerequisite admission: segment admission verifies all records, checksums and identities with bounded duplicate-detection allocation; layout record admission may allocate bounded layout metadata; catalog admission binds its entries to admitted segment records.

A framing request on already admitted evidence still requires that stronger admission to have succeeded first; these APIs are not shallow raw-byte scans that tolerate deeper corruption.

Records prepared for publication provide the same logical proof over their canonical representation without asserting that the record has been written or made durable.

## Durable ingress and view collection

`verify_segment` admits raw segment bytes before reporting; `verify_catalog_bytes` admits the supplied publication head, catalog and selected segments before reporting.

`FilesystemCatalogSnapshot::load_for_verification` reads the exact selected artifacts under `CatalogRestartPolicy`, and its `verify` and `verify_blob` methods re-admit owned bytes before reporting.

`FilesystemRetentionSnapshot::load_for_verification` holds the existing shared fence and uses bounded before/load/after collection; `verify_retention` reads and verifies only the manifest-selected root for the supplied namespace digest, checks namespace/generation/digest, and establishes the requested root evidence against that same catalog.

Filesystem loading blocks on reads and fence acquisition; the owner retains caller-bounded segment bytes plus protocol-bounded catalog/manifest data, while reporting may rebuild the bounded catalog indexes.

Selected-root verification holds one bounded root buffer, decoded anchors and catalog indexes; closure adds its checked node-bounded member index and one decoded layout at a time, with no whole-blob output buffer.

These operations do not publish, repair, synchronize, run recovery, or acquire writer authority; a retained incomplete stage is not disposed of by verification.

Exhausted moving-view collection returns `Ambiguous` with the actual last before/after catalog and retention coordinates; no partial view or report is returned.

A failed observation is operational unless its retained typed cause establishes a precise missing artifact or content contradiction; classification never parses error prose.

Each named original interface verifies one requested subject and returns one `VerifiedSubject`; traversal of a blob's chunks or a root's anchors establishes that subject's depth, without manufacturing separate reports for its dependencies.

This satisfies the original per-subject interface contract; it is not a whole-store enumeration or aggregate-report API, and the earlier work-in-progress references to a required aggregate operation were broader than the original named interfaces.

## Catalog-ceiling memory boundary

The catalog-ceiling runtime law supplies 1,048,576 distinct chunk records and requires exact sample lookups plus a `CatalogReachability` report within 1 GiB (1,073,741,824 bytes) of incremental tracked live allocations during catalog/head admission, lookups and reporting.

This bound excludes caller-owned encoded segment/catalog buffers, fixture construction, segment admission, allocator bookkeeping and process RSS; it is not a total-process memory promise.

Filesystem owners additionally retain the selected segment bytes up to their explicit `CatalogRestartByteLimit`, protocol-bounded catalog bytes and admission indexes; those owners must be included when sizing a verification process.

No serialization, repair, quarantine, GC execution or new durable report format is introduced here.

[Evidence and calibration](../../testing-evidence/durable-verification.md) distinguish runtime laws, static/API restrictions and final acceptance checks.

## Logical refusal contract

Blob and root verification distinguish missing catalog members, demonstrated contradictions, unsupported requests, and operational failures without returning partial reports. Original layout or closure causes retain their typed coordinates. Resource exhaustion is operational, not evidence of corruption.

The report preserves the catalog generation/digest used by catalog, blob and root operations; this provenance is not a live fence. Multiple valid layouts for a blob are representations, not automatically ambiguity: discovery selects the first canonical identity.

Immutable admitted-view operations have no moving observation to classify; ambiguity is produced by the durable collection path, retaining at most the last conflicting coordinate pair and the original attempt-limit cause.

Raw layout/root decoder errors also convert into `VerificationError` without losing their typed causes; these conversions name unadmitted input subjects and cannot manufacture a report.
