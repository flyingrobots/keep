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
| `CatalogSnapshot::verify` | Selected catalog generation and digest | `Framing`, `Checksum`, `CatalogReachability` | Exact catalog-to-record bindings; no complete logical or retained closure claim. |

Every other depth returns `VerificationRefusal::Unsupported` with the exact subject, request and supported set; the operation neither downgrades the request nor returns a success report.

`SnapshotBinding` remains unsupported until its separate protocol exists; catalog/retention coordinates must not be mislabeled as that future proof.

## Costs and admission boundary

Each current reporting call is constant time, allocates no heap memory, performs no I/O, does not block, and neither mutates nor synchronizes storage.

These costs exclude prerequisite admission: segment admission verifies all records, checksums and identities with bounded duplicate-detection allocation; layout record admission may allocate bounded layout metadata; catalog admission binds its entries to admitted segment records.

A framing request on already admitted evidence still requires that stronger admission to have succeeded first; these APIs are not shallow raw-byte scans that tolerate deeper corruption.

Records prepared for publication provide the same logical proof over their canonical representation without asserting that the record has been written or made durable.

## Remaining durable contract

Raw durable loading-to-report failure classification, missing/corrupt/ambiguous/operational outcomes with preserved causes, complete-blob and retention reports, aggregate per-subject reports, and the catalog-ceiling memory campaign remain required by the [closure ledger](../../audits/114-durable-verification-scope.md).

No serialization, repair, quarantine, GC execution or new durable report format is introduced here.

[Evidence and calibration](../../testing-evidence/durable-verification.md) distinguish runtime laws, static/API restrictions and unimplemented acceptance obligations.
