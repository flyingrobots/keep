# Verification reports

Status: implementation in progress under [#114](https://github.com/flyingrobots/keep/issues/114); this page describes the currently implemented admitted-evidence reporting operations, not completed durable verification acceptance.

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

## Remaining durable contract

Raw durable loading-to-report failure classification, published-retention namespace selection, aggregate per-subject reports, and the catalog-ceiling memory campaign remain required by the [closure ledger](../../audits/114-durable-verification-scope.md).

No serialization, repair, quarantine, GC execution or new durable report format is introduced here.

[Evidence and calibration](../../testing-evidence/durable-verification.md) distinguish runtime laws, static/API restrictions and unimplemented acceptance obligations.

## Logical refusal contract

Blob and root verification distinguish missing catalog members, demonstrated contradictions, unsupported requests, and operational failures without returning partial reports. Original layout or closure causes retain their typed coordinates. Resource exhaustion is operational, not evidence of corruption.

The report preserves the catalog generation/digest used by catalog, blob and root operations; this provenance is not a live fence. Multiple valid layouts for a blob are representations, not automatically ambiguity: discovery selects the first canonical identity.

The refusal vocabulary contains bounded conflicting candidates, but the admitted immutable-view operations do not manufacture an ambiguity outcome merely to exercise that variant; durable observation classification remains part of the unfinished contract.
