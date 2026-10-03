# Durable verification evidence

Status: partial implementation for [#114](https://github.com/flyingrobots/keep/issues/114); the full durable verification acceptance contract remains open in the [scope ledger](../audits/114-durable-verification-scope.md).

## Catalog report slice

Change kind: new feature; subject: Keep's public runtime reporting API, with separately identified static/API construction restrictions.

The source baseline is `945f06c7e24d7754c06ce81d9f57c519fbc46902` plus the catalog-report implementation and public laws committed with this record.

The specified oracle is the subject-specific verification contract and the frozen generation-two catalog/head corpus, whose catalog digest is `ea7d0055fd21f00ed94809ef4e671d72fa2e6a4a5d9ecefb23f3a320a2dad993`.

The tests exercise `CatalogSnapshot::verify`, not test-harness case counts or source text.

| Claim | Law or API evidence | Falsification observed |
| --- | --- | --- |
| The original request is preserved. | `catalog_reports_bind_the_requested_evidence_to_the_selected_generation` | Replacing the report request with `Framing` failed `original request must be retained` for a checksum request. |
| Per-subject evidence is neither escalated nor attached to another generation. | The same public law, checking the complete returned subject/depth projection. | Replacing achieved depth with `CompleteBlobIdentity` failed the report assertion; substituting the successor generation failed with observed 3 versus expected 2. |
| Unsupported requests retain exact coordinates and supported policy. | `catalog_requests_outside_its_evidence_refuse_without_downgrading` | Disabling the support guard failed with `unsupported depth certified`; the successor-generation mutation also failed the exact refusal assertion. |
| Catalog reachability does not certify incomplete blob reconstruction. | `catalog_reachability_does_not_certify_an_incomplete_blob`, using an admitted layout-only catalog whose required chunk is absent. | Disabling the support guard failed with `catalog membership was presented as complete blob verification`. |
| Reporting over already admitted evidence allocates nothing. | `reporting_catalog_evidence_requires_no_additional_allocation` | A deliberately allocated, black-boxed 1,024-byte vector failed `reporting must not allocate` with observed 1,024 versus expected zero. |
| Callers cannot manufacture or deepen a report. | Compile-fail rustdoc on `VerifiedSubject` field mutation and `VerificationReport::established`, plus a compiling accessor example. | The negative examples are rejected by Rust; this is static/API evidence, not runtime RED. |

All mutation failures occurred after successful compilation in the named runtime law.

Each mutant used an isolated copied source tree and its own Cargo target directory; the candidate was not mutated.

The new API is absent on the parent, so no parent compilation failure is presented as runtime RED evidence.

## Execution and replay

Execution used the existing Linux arm64 Docker validation container, pinned Rust 1.96.0, copied repository sources, and owned scratch directories.

The catalog laws are small, deterministic in-memory tests with frozen checked-in inputs and no filesystem, network, clock, spawned thread, or scheduling dependency in their bodies.

No random generation, concurrency, fault schedule, process-death, or power-loss claim is made by this slice.

The runner does not enforce per-test resource ceilings or a measured suite latency SLO; those remain the disclosed repository [enforcement gaps](../testing/enforcement.md).

The report-allocation law measures incremental bytes allocated on the test thread after snapshot admission, not catalog admission memory or process peak RSS.

Replay the product laws with `cargo test --locked --test catalog_verification` and `cargo test --locked --release --test catalog_verification` inside the copied Docker checkout.

Replay construction restrictions with `cargo test --locked --doc`.

Debug and release laws, doctests, all-target/all-feature Clippy with warnings denied, formatting, and the source-structure check passed.

The first structure-check attempt stopped because the copied source lacked Git metadata; after initializing and indexing the copied tree, the structure check passed, with the original failed command log retained.

Raw local artifacts are retained under the issue's audit scratch record: `catalog-first-check.log`, `catalog-release-api-check.log`, `catalog-structure-corrected.log`, `catalog-post-calibration-green.log`, `incomplete-blob-calibration-red.log`, and the `catalog-mutants` source/log directories.

## Remaining acceptance

This slice reports already admitted catalog evidence only; it does not yet provide raw durable loading-to-verification error classification, complete-blob/retention reports, all required durable depth operations, an aggregate report, or the catalog-ceiling memory campaign. The segment and logical-record extension is recorded below.

Full required validation, independent exact-head review, hosted checks, and the final #114 PR remain pending until the complete candidate is stable.

No existing tests were deleted or expectations weakened; individual laws state their deletion criteria beside their oracles.

## Segment and logical-record reporting

Change kind: new reporting feature over already admitted evidence, plus a behavior-preserving relocation of `SegmentDigest` from the codec adapter to domain ownership; the source baseline is `92f92089cf00d1a766fe8775dac28561ff738b27`.

`AdmittedSegment::verify` reports only physical framing/checksum evidence, while `AdmittedSegmentRecord::verify` reports framing/checksum and the exact record kind's chunk or layout identity.

The public [subject/depth matrix](../invariants/verification/README.md) documents costs and rejects ordinal-depth inference, publication claims and complete-blob claims from an isolated layout record.

The physical-coordinate oracle is the frozen empty and one-zero segment digests in `conformance/segment-store/v1/artifacts.tsv`.

Logical subjects use the frozen one-zero bundle, its declared layout identity and the named identity of the one-zero chunk; the report implementation does not derive expected values for these tests.

These small tests use immutable in-memory fixtures and exhaust the finite request vocabulary for physical segments and both logical record kinds; no random seed, scheduler, filesystem fault or generated-space completeness claim applies.

| Protected claim | Deliberate production violation | Observed runtime failure |
| --- | --- | --- |
| Unsupported requests never become success, including an isolated layout without its chunks. | Disable the segment and record support guards. | The physical and logical matrix laws reject unsupported success; the isolated-layout law reports `layout identity was presented as complete-blob evidence`. |
| Physical evidence names the exact frozen segment. | Replace the reported segment digest with zero bytes. | The physical-coordinate assertion and exact refusal-subject assertion fail. |
| Chunk evidence names a chunk subject. | Replace only the chunk report subject with a physical-segment subject. | The public record subject/proof assertion fails. |
| Layout evidence names its canonical layout subject. | Replace only the layout subject with a physical-segment subject, leaving chunk reporting intact. | The later layout branch of the matrix and the isolated-layout law fail. |
| Requests remain distinct from achieved evidence. | Replace the report request with `SnapshotBinding`. | The segment and logical-record request assertions fail. |
| Achieved evidence is never silently inflated. | Replace only the achieved depth with `CompleteBlobIdentity`. | Request assertions pass, then segment and logical proof assertions fail. |
| Refusals preserve the exact supported policy. | Return an empty supported set while retaining the guard. | Exact typed-refusal assertions fail for physical segments and both logical record kinds. |
| Reports contain their promised subject evidence. | Return an empty subject slice. | Physical and logical report laws fail with their missing-subject diagnostics. |
| Reporting allocates no additional heap memory. | Allocate and black-box a 1,024-byte vector in each reporting operation. | Both incremental-allocation assertions report 1,024 bytes rather than zero. |

All listed RED results follow successful compilation and reach the intended runtime check; each mutant uses separate copied source and a separate target directory, leaving the candidate unchanged.

Raw sources, logs and exit receipts live under `segment-mutants`; the exact omitted-subject failure is also preserved in `segment-mutants/no-subject/red.log`.

The initial calibration-copy attempt exhausted the ext4 scratch mount's inode capacity before the remaining mutants could run; `segment-calibration-inode-status.log` records that environment condition, which is excluded from calibration evidence.

Completed and incomplete copied sources were preserved on the container's larger build filesystem, and the remaining in-memory calibrations ran there without changing their inputs or oracle.

The unchanged candidate passes debug/release report laws, existing catalog reports, segment/record/header/seal and memory laws, doctests, all-target/all-feature Clippy, formatting and source structure.

`segment-release-related-validation.log` preserves an attempted nonexistent test-target invocation; `segment-related-validation-corrected.log` runs the actual segment-record/header/memory targets and remaining checks, without counting the invocation error as a product result.

The execution receipts are `segment-first-check.log`, `segment-release-related-validation.log`, `segment-related-validation-corrected.log` and `segment-post-calibration-green.log`.

Per-test resource enforcement remains the previously disclosed repository gap; allocation assertions measure reporting after admission, not total admission memory or process RSS.

No existing runtime expectations were changed or tests removed, and the full durable failure/retention/aggregate and memory-ceiling obligations remain open.
