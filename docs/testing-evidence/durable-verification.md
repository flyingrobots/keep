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

This slice reports already admitted catalog evidence only; it does not yet provide raw durable loading-to-verification error classification, segment/blob/retention reports, all required durable depth operations, an aggregate report, or the catalog-ceiling memory campaign.

Full required validation, independent exact-head review, hosted checks, and the final #114 PR remain pending until the complete candidate is stable.

No existing tests were deleted or expectations weakened; individual laws state their deletion criteria beside their oracles.
