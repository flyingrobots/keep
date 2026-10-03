# Durable verification evidence

Status: the scoped implementation and post-readiness review corrections for [#114](https://github.com/flyingrobots/keep/issues/114) are implemented on the PR branch. Final exact-head independent review and required checks remain acceptance gates in the [scope ledger](../audits/114-durable-verification-scope.md); mainline delivery is not claimed. The sections below preserve chronological slice evidence, including superseded intermediate limitations.

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

## Complete blob and admitted-root evidence

Change kind: new feature, based on `d65c845a05a96bdf5f724c79f4a0aa211fdb1ca9`; these APIs did not exist on the parent, so a parent compile failure is not claimed as behavioral RED.

The public `blob_verification` suite checks requested and achieved evidence, exact catalog provenance, canonical root coordinates, incomplete closure, absent blobs, contradictory complete blob identities, profile replay, unsupported-depth refusal, and resource-failure classification with original typed causes.

The specified oracles are the subject/depth contract, the frozen one-zero layout, a one-one logical target paired with one-zero chunk bytes, and the permanent `profile-boundary-mismatch` corpus witness whose required/replayed boundary lengths are 262143 and 262144.

All law bodies are small in-memory experiments; they use admitted segment/catalog records and real verification operations, with no sleeps, random schedule, ambient filesystem, network, clock or product mutation. Resource-ceiling enforcement remains the previously disclosed repository gap; classification tests are not a claim that a per-test sandbox has been implemented.

| Protected claim | Calibration | Observed runtime failure |
| --- | --- | --- |
| Complete logical closure must execute. | Bypass blob closure while retaining callable production code. | Missing-chunk, wrong-identity and false-profile laws refuse the unexpected success. |
| Root closure must execute before certification. | Skip root closure verification. | Missing-layout and resource-limit laws refuse the unexpected success. |
| Each subject admits only its supported depths. | Disable blob/root policy guards independently. | Exact unsupported-depth laws fail. |
| Reports retain catalog provenance. | Clear the catalog coordinate during report construction. | Blob/root report assertions fail with `None` instead of the exact coordinate. |
| Achieved depth cannot be escalated. | Set achieved depth to `SnapshotBinding`. | Blob/root and shallow-evidence assertions fail. |
| Expected and observed identities are not interchangeable. | Swap the identity coordinates in corruption mapping. | Exact complete-blob contradiction assertion fails. |
| The original typed cause survives classification. | Remove the closure cause from refusals. | Missing-chunk, complete-blob and profile-source assertions fail. |
| Resource refusal is not corruption. | Classify the closure limit as a content contradiction. | The operational-limit law fails. |
| Correct bytes still require registered profile replay. | Bypass feed/finish checks while preserving complete hashing. | The permanent false-profile law fails. |

The calibration ran in a separate copied Docker source tree, restoring the changed file after each isolated mutation and cleaning the Keep package's artifacts before recompilation; dependencies were reusable, while changed Keep artifacts were not accepted from cache. Every listed RED compiled and failed in the named runtime law. The candidate source was never mutated.

The first calibration script stopped after a valid blob-closure runtime RED because `rg` was unavailable inside the container; the corrected remainder used `grep`. Both logs and the first valid RED are preserved. Initial empty-catalog setup incorrectly supplied an unreferenced empty segment and failed before verification; its corrected fixture supplies no segments. A separate wrong test-module path failed module resolution and is also excluded from product RED evidence.

Raw receipts include `closure-laws-first.log`, `closure-laws-corrected.log`, `closure-clippy-corrected.log`, `closure-laws-complete.log`, `closure-laws-complete-corrected.log`, `closure-calibration.log`, `closure-calibration-remainder.log`, and each changed source, original source, build output and exit code under `closure-mutants`.

Replay the focused suite with `cargo test --locked --test blob_verification` and its `--release` counterpart in the copied Linux arm64 Rust 1.96.0 Docker environment. Related catalog, segment and closure laws remain unchanged; no tests were deleted or prior success expectations weakened.

This evidence establishes operations over admitted immutable views, not raw filesystem failure classification, publication-selected retention, aggregate reports, the catalog-ceiling memory bound, process-death recovery, or final exact-head review and CI for the complete #114 candidate. Those remaining obligations are still open and implementation continues within the same PR.

After restoring the candidate, the blob, catalog, segment and existing retention-closure suites passed in debug and release, and all ordinary and compile-fail doctests passed. The first post-calibration command then stopped at the nonexistent `source-check` alias; the corrected `cargo xtask source-structure-check` and formatting check passed separately. All-target/all-feature Clippy with warnings denied passed after the focused suite. The corrected structure receipt is `closure-structure-corrected.log`; these local receipts do not replace the full validation chain on the final candidate.

## Durable ingress, selected publication and bounded observation

Change kinds: new verification feature and a focused bug fix to preserve selected-root diagnostic coordinates, based on `b204f83d6aa9e55f3984ea0933890abb00576847`.

The selected-root coordinate regression was observed RED on that exact parent with only the new public diagnostic type/export and regression test transplanted; the production reader remained unfixed and returned a message-only I/O cause, so the downcast observed `None` instead of the exact expected/observed generation/digest refusal.

The corrected reader passes the same runtime assertion; `coordinate-parent-red.log` and `selection-laws-green.log` distinguish actual bug evidence from new-API compile failures.

Public ingress laws check segment framing/checksum/version and resource classification, catalog absence/version and retained-byte limits, moving catalog/retention coordinates, exact final conflict candidates, permission failures, typed publication-decoder contradictions, selected-root absence/checksum/namespace refusal, and exact catalog/retention provenance.

Existing segment framing/identity corruption laws now apply their original exact typed assertions to the cause retained by verification, and the catalog mutation laws additionally require the same exact decoder error through raw catalog verification.

The v1 report matrix covers empty and populated physical segments, chunk/layout records, catalog success/refusal, absent or incomplete blob closure, and complete blob identity/profile replay; the v2 matrix covers admitted and publication-selected roots, closure/refusal, and moving retention heads.

Supported and unsupported depth sets together exercise every depth for each named admitted subject; published retention delegates to the same root-depth operation after exact selection, and raw ingress must complete prerequisite admission even for a shallow request.

The existing root/manifest/head decoders retain their own precise corruption-law suites; typed observation and root adapters classify their causes without reparsing prose, while unknown I/O and allocation/resource failures remain operational.

The deterministic moving-view laws are port-level schedules, not filesystem race or syscall evidence; selected-root success/refusal and unchanged-evidence assertions execute against actual copied Linux ext4 fixtures with the reader fence held.

The ingress calibration campaign independently inverted raw resource/content classification, reversed candidate order, accepted a moving view, dropped known observation classification, bypassed namespace selection, erased retention-head provenance and inverted root-corruption classification; each mutation compiled and failed its intended public runtime assertion.

A separate mutation wrote changed selected-root bytes after reading them; the first successful report's persistent-evidence comparison failed, demonstrating that read-only success is an asserted observable outcome rather than merely a method name.

Calibration artifacts preserve original and mutated source, command, compiler output, runtime failure and exit status under `ingress-mutants`; `cargo clean -p keep` invalidated changed package artifacts between mutants, and mutations occurred only in a separate copied tree.

The catalog-ceiling law admits 1,048,576 distinct chunk entries, verifies exact named sample bytes and reports catalog reachability; the 1 GiB assertion measures incremental tracked live allocations during catalog/head admission, lookup and reporting, excluding fixture construction, pre-admitted segment owners and process RSS.

On Linux arm64 Rust 1.96.0, an isolated measurement probe using the unchanged production implementation and a deliberately zero test threshold recorded 436,207,624 peak tracked bytes; that probe is measurement extraction, not assertion calibration.

The actual calibration added a live allocation of 1,073,741,825 bytes inside production catalog reporting; the unchanged 1 GiB assertion failed at an observed 1,241,513,985 peak tracked bytes, while the unmutated law passed in debug and release.

The memory receipts are `catalog-ceiling-green.log`, `final-ingress-focused.log`, `ceiling-measurement.log` and `ingress-mutants/admission-memory/red.log`; these measurements are not performance comparisons or total-process memory guarantees.

Focused debug/release ingress, selection and catalog laws passed; all-target/all-feature Clippy passed after them, and the complete candidate validation is recorded separately below.

The first broad copied-tree run passed product tests but stopped on two tooling-environment failures: `b3sum` was absent from PATH and the copied Git repository had no commit for a clone-based documentation-integrity test; `ingress-stable-validation.log` preserves those failures and they are not product regression RED evidence.

The singleton report interpretation is explicitly reconciled in the normative page and scope ledger: the original named interfaces each select one subject, and no whole-store aggregate enumeration is claimed.

Final required checks and independent review remain pending for the exact committed/pushed candidate; earlier receipts do not transfer approval to a different head.

## Independent-review corruption mapping closure

The independent Codex review of `6504c869a379a18db907aed79f7fa3a3b375d9e9` requested complete mapping of existing corruption laws through production verification classification; it found no demonstrated production correctness defect and accepted the explicit singleton and incremental-memory scope reconciliation.

The review is recorded at [PR #165 review comment](https://github.com/flyingrobots/keep/pull/165#issuecomment-5964309362), with the full checklist in `codex-review-6504c86.md`.

The follow-up preserves original exact decoder assertions and routes their real decoded failures through the following public production boundaries; the test support only extracts the original cause after checking classification, never substitutes an expected error.

| Existing corruption family | Production verification boundary | Retained oracle |
| --- | --- | --- |
| Complete-segment framing, record identity, nested header/checksum | `verify_segment` | Original `SegmentReadError` fields and nested causes in `tests/segment`. |
| Canonical catalog mutation fields | `verify_catalog_bytes` | Same `CatalogDecodeError` as direct admission in `tests/catalog/mutation_support.rs`. |
| Publication-head fields, width, checksum and coordinates | `collect_verification_view` | Original `PublicationHeadDecodeError` from real decoding in `tests/publication_head.rs`. |
| Retention head and manifest framing, integrity and semantics | `collect_verification_view` | Original decoder error inside the same `RetentionCurrentStateRefusal` observation boundary. |
| Root framing, integrity, anchor identity and semantic admission | Public `VerificationError::from(RetentionRootDecodeError)` used by raw root classification | Original `RetentionRootDecodeError` in `tests/retention_root_decoding.rs`. |
| Layout mutation corpus, policy limit and expected identity | Public `VerificationError::from(LayoutDecodeError)` using the same layout classifier as blob verification | Original first-failure predicate and typed coordinates in `tests/layout_mutations.rs`. |

The root and layout raw-error conversions name unadmitted input subjects, allocate only the error box and confer no verified report or identity; original typed resource failures remain operational.

The mapped laws pass in debug and release, with all-target/all-feature Clippy denying warnings; receipts are `corruption-mapping-final-green.log` and the preceding setup/compiler failures, which remain excluded from behavioral RED evidence.

Three isolated production mutations inverted raw root classification, inverted layout classification, and discarded known retention-observation classification; each compiled and produced the intended runtime failure in the existing corruption suites, recorded with original source and command under `corruption-mutants` and summarized in `corruption-calibration-corrected.log`.

The earlier `6504c86` candidate passed the corrected local workspace debug/release, both feature-mode Clippy, doctests, documentation build, structure and formatting checks, and all four required hosted checks in [run 37087373122](https://github.com/flyingrobots/keep/actions/runs/37087373122).

Those green results do not approve this subsequent mapping delta; the final pushed head requires its own independent confirmation and hosted checks before readiness.

The delta review found a surviving resource-classification oracle: the layout wrapper accepted a `Corrupt` result carrying `ConfiguredEntryLimitExceeded`, so the first inverted-classification receipt showed the configured-cap law still passing while the other layout laws failed.

The corrected wrapper explicitly rejects that resource cause in the corruption arm; `layout-resource-red.log` records the same production inversion failing the targeted configured-cap law with “a configured resource cap must remain operational, never corruption,” and `layout-resource-green.log` records unmutated debug/release and focused Clippy success.

This is a strengthened test oracle; production classification was unchanged and already correct, so the mutation is calibration rather than a claim of a runtime bug on the parent.

## Post-readiness review: shallow root provenance

Change kind: bug fix. CodeRabbit identified that `AdmittedRetentionRoot::verify` attached catalog provenance even when framing/checksum reporting never consulted the catalog. On unfixed `b33c7da4ec6fae68437b49663ca383edf5c297e1`, the new public regression `shallow_root_reports_do_not_claim_an_unconsulted_catalog` failed with a framing report's `Some((generation, digest))` against specified `None`, using an empty catalog that cannot establish the root's closure.

The fix attaches catalog coordinates only after successful retention-closure verification. Shallow reports still name the exact admitted root and requested depth; publication-selected reports retain their actual retention-head provenance. Existing shallow catalog expectations in both direct and filesystem laws were incorrect and are corrected to `None`; the closure expectations remain unchanged. This is a correction to the report's evidence claim, not a change to root admission, retained closure, publication or formats.

Copied Docker runs of `cargo test --locked --test blob_verification` and its release counterpart pass after the fix. The `filesystem_verification_law_tests` library filter passes in both profiles, exercising the publication-selected root boundary and unchanged filesystem witnesses; focused library/integration Clippy passes with warnings denied. These filesystem fixtures are repository-admitted test stores, not a new production-platform or physical durability certification. The parent RED, direct GREEN and filesystem GREEN receipts remain distinct; final full validation belongs to the final pushed candidate after the remaining review findings are resolved.

## Post-readiness review: store-admission classification

Change kind: bug fix. The public verification loader on `26d352202f06ca8bf36d203604f7897d2eaa5161` classified typed version-two record corruption and migration-bound root-identity disagreement as operational failures. New filesystem regressions observed both failures before the fix: a changed `FORMAT` magic retained `VersionTwoRecordRefusal::Marker(InvalidMagic)` inside an operational result, and copied migration records retained exact donor/recipient inode coordinates inside an operational result.

The classifier now admits those known typed contradictions as `Corrupt` for `PublishedView`, retaining the entire original admission cause. Its exhaustive version-two record match leaves host-width overflow operational; unclassified I/O, including an `InvalidData` kind without a recognized contradictory cause, remains operational. No error-message parsing or blanket kind-based corruption inference is used.

The corrected runtime laws cover malformed `FORMAT`, intent and receipt magic, exact root-identity coordinates, and unchanged refused evidence. Public error-conversion laws distinguish host-width exhaustion and unclassified I/O; these conversion cases are simulated causes, not claimed syscall injections. Docker debug/release execution and focused all-feature library Clippy pass. Additional production mutants demonstrate failures for the wrong public subject, lost original source, resource-as-corruption and untyped-I/O-as-corruption. Sources are restored with cache timestamps invalidated before the subsequent GREEN runs; calibration concerns distinct behavioral claims rather than repeating the full campaign per field.

## Post-readiness review: depth comparison API

Change kind: public API correction. `VerificationDepth` no longer implements `Ord` or `PartialOrd`; equality and every operation's explicit supported-depth set remain intact. This removes the misleading ability to treat catalog reachability as ordinally stronger than complete blob identity. The public rustdoc example is a static/API compile-fail contract, not runtime storage evidence.

On unfixed `665ffb3bf65efd94ba46a94c28dfde636828caf1`, the new compile-fail example failed because the forbidden comparison compiled successfully. After removing the ordering derives, the same example passes with the intended unsupported binary comparison. Catalog, segment and blob/root verification laws pass in Docker debug/release, preserving actual runtime depth acceptance/refusal. This corrects an unreleased API; it changes no durable format or verification policy.

## Post-readiness review: exhaustive shared classification

Change kind: refactoring. Layout admission, nested retention-closure layout failures and durable ingress now call one exhaustive `layout_class` match. Its operational variants remain allocation, host entry-count width, host record-length width and configured entry limit; every other existing layout variant remains corrupt. The current-publication observation match explicitly names its remaining operational variants instead of accepting future variants through a wildcard. This is compiler-enforced classification ownership, not a new runtime failure policy.

A temporary Rust probe generated every single-bit mutation at every byte of the frozen empty, one-zero and max-plus-one-zero layout records, plus unchanged records, under a one-entry cap and the maximum cap. It captured the public decoder/error-conversion outcomes on refactor parent `2b28c4a` and the candidate in separate copied Docker source/build trees; the complete ordered receipts compare byte-for-byte equal. This bounded differential evidence covers the generated decoder outcomes, not arbitrary allocation failures or a universal equivalence proof. The probe and raw receipts remain review artifacts; no implementation-shaped test was added to the permanent suite.

Unchanged verification ingress/view, blob/root, segment, retention-root decoding and layout-decoding laws pass in Docker debug/release. All-feature all-target Clippy passes after consolidating identical operational match arms. The initial duplicate-arm lint failure is retained as setup/tooling feedback and is not claimed as behavioral RED. No runtime expectations were rebaselined for this refactor.

## Post-readiness review: diagnostic calibration closure

Change kind: evidence correction; production and permanent tests are unchanged. Independent review of `6802644ccf0d547694ab26644b9c306a43ddaeba` found that the source-removal mutant failed before the new exact diagnostic assertions. That receipt proves source presence, not diagnostic payload accuracy.

Two diagnostic-only producer mutations on copied `6802644` preserve corruption classification and typed sources: the format-marker decoder reports zero magic bytes after rejecting the actual bad magic; root admission swaps the expected and observed identity coordinates after detecting disagreement. The existing public loader laws compile and fail respectively at `original magic for FORMAT` and `original binding coordinates required`. These are direct failures of the diagnostic assertions, not earlier source-presence failures or compilation errors. The originals, mutants, replay commands and failing logs are retained with the review receipts.

Restoring both production files and invalidating source timestamps yields GREEN for `cargo test --locked --lib verification_store_admission_tests` and its release counterpart in a separate copied Docker source/build tree. The stable candidate's broader Docker chain also passes formatting, source structure, all-feature and no-default-feature Clippy with warnings denied, debug/release workspace tests, doctests and documentation build. Local documentation-integrity execution was unavailable because that Rust container lacks Markdown tooling; its failed setup attempt is retained separately, and hosted documentation/workflow integrity passed on `6802644`. All four required hosted jobs passed on that head; none of those results is substituted for required checks on a later pushed head.

## Post-readiness review: unsupported request precedence

Change kind: bug fix. CodeRabbit's global review of `6802644` identified that publication-selected retention verification read root evidence before rejecting unsupported depths. On unfixed `cc1e37b`, the new public `unsupported_retention_requests_refuse_before_missing_evidence` law compiles and fails with `Missing` plus the original filesystem `NotFound` cause where exact `Unsupported` was required. The permanent regression covers every unsupported depth for a missing selected root and an absent namespace; the oracle names the requested namespace, depth, supported set and absence of an operational source.

The filesystem entry point now checks the same supported-depth constant as direct admitted-root verification before selected-root reads or catalog re-admission. Supported requests retain their existing evidence-verification path. Snapshot loading remains separate and can still refuse during admission; this change does not hide those failures. No durable format, writer behavior, recovery protocol or runtime limit changes.

In copied Docker source, the new law and existing filesystem verification laws pass in debug and release; all-feature library/integration Clippy passes with warnings denied. The parent RED directly calibrates the new exact refusal assertion. No existing expectation was changed, no sleep or uncontrolled schedule was introduced, and ordinary test resource-enforcement gaps remain those documented in the testing enforcement ledger. Final candidate checks and independent delta review remain separate gates.
