# Catalog publication admission audit

This page owns the T-12.2 and T-12.3 verdicts from the originally checked roadmap at `1a586d83d5750083172d440f90e7b786d540ff0e`, lines 546–550. The T-12.2 finding inspects main `b50dbd4cb4cee286aea1aa0352152a232197dda1`; the later T-12.3 verdict names its own inspected revision below. T-12.1 remains under review.

## T-12.2 — Writer exclusion and platform-admitted publication

**Acceptance/definition of done: not met across supported features.** The
ordinary publisher constructor consumes a private-field
`FilesystemPlatformAdmission`. Writer locking and the publication state
machine have executable exclusion, ordering, refusal and recovery evidence.
However, `repository-tasks` exposes a public alternate constructor accepting
only `FilesystemWriterLock`. Its admission producer performs lenient root
identity observation without enforcing the production platform profile.

The path is
`src/adapters/filesystem_catalog_publisher.rs::open_unchecked_for_repository_tasks`
through
`src/adapters/filesystem_platform_admission.rs::unchecked_for_repository_tasks`
to the lenient identity probe in `filesystem_platform_profile.rs`. The
existing architecture law checks the ordinary constructor and admission type
text, so it does not cover this alternate public route. The production crash
harness consumes that route after unchecked repository initialization.

A temporary public API probe on an actually unsupported Linux Docker
filesystem first verified production initialization's exact
`AdmitPlatform/Unsupported` refusal. It then created the canonical namespace,
acquired the writer lock, and called the public repository-task constructor.
The constructor returned a publisher. The refusal law failed with
`refused platform acquired public publisher authority` on the exact inspected
main source, using a dedicated build directory. No admission flags were
forged, no bad artifact was published, and no host test was run.

This demonstrates a platform-admission capability bypass, not corruption or
physical power-loss failure. The original platform-admitted publication
criterion and the feature-invariant rule remain binding. Executable
[issue #150](https://github.com/flyingrobots/keep/issues/150) owns the public
boundary regression, correction and affected crash-harness adaptation as one
independently mergeable outcome. Shared harness files alone do not establish
a prerequisite on the separate sealed-stage escape in #146.

## Executed evidence and remaining limits

Eleven catalog integration targets passed 59 laws in Docker debug and 59 in
release on `8d902516e682361882bc5c9902de296ce5c9de85`. The two ordering laws
and sixteen filesystem publisher fixture laws also passed in each mode.
These fixtures use an unchecked test publisher and do not prove production
platform admission. The failure probe ran separately on the inspected newer
main source; passing existing fixture laws does not negate that failure.

The catalog ledger's absent ordering and filesystem publication evidence
owners were corrected by
[PR #149](https://github.com/flyingrobots/keep/pull/149) for #148, integrated
as `82374a995df095106aefe52f87ab3cb26184639d`. The two ordering and sixteen
publisher fixture laws were freshly rerun in Docker debug and release on
its exact target `b50dbd4cb4cee286aea1aa0352152a232197dda1`; the documentation
PR changes no runtime or test code. Those reference corrections do not
resolve T-12.2 or close #150. No fresh crash campaign or host-power-loss
evidence is claimed here.

## T-12.3 — Restart snapshot and model agreement

**Acceptance: restart examples supported; model-agreement evidence incomplete under the binding testing standard.** This verdict inspects main `6051abb25a9fd33ae7ee0de5614514b709a4d82a`. `KEEP-CATALOG-009` and `KEEP-CATALOG-010` are separate promises; passing restart examples does not establish generated model agreement.

`src/adapters/catalog_restart_loader.rs::load_from_directory` reads and admits the selected head, admits its named catalog, checks generation/length/digest, loads the referenced segments and constructs an admitted filesystem snapshot. `tests/catalog_restart.rs` exercises exact frozen payload reconstruction. Its `catalog_restart/refusal_laws.rs` exercises corrupt and unsupported heads, noncanonical catalogs, missing catalog/segment files and conflicting physical-name contents through `FilesystemCatalogSnapshot::load`. These are actual filesystem-backed public outcomes, not harness enumeration. They do not establish arbitrary concurrent out-of-band namespace isolation or physical power-loss recovery.

`tests/catalog_model.rs::generation_transitions_and_lookups_match_a_btree_map` checks only one hand-written sequence: bundle, one chunk, empty. Its `model()` obtains expected identities and payloads through the same production `AdmittedSegment::records()` boundary consumed by catalog construction. The example can detect some lookup defects, but shared decoding errors can agree on both sides. It has no generated history space, independent input-derived map, absent-lookup checks or generated invalid transitions. The separate transition examples establish specified stale/predecessor refusals at their chosen points; they do not close this model-evidence gap.

Testing Standards rules 5 and 6 require generated evidence for agreement claims and an independently grounded oracle. Existing tests have no blanket exemption under `docs/testing/enforcement.md`. Therefore the complete definition of done is not established, even though the historical requirement ledger says implemented and its existing example passes. [Issue #166](https://github.com/flyingrobots/keep/issues/166) owns generated independent model histories, exact runtime assertions, replay/reduction and assertion calibration as one independently mergeable correction. No production catalog defect is alleged by this finding.

Fresh source-specific Docker runs on `6051abb25a9fd33ae7ee0de5614514b709a4d82a` passed `cargo test --locked` and `cargo test --release --locked`, each selecting `catalog_generation`, `catalog`, `publication_head`, `catalog_encoding`, `catalog_locations`, `catalog_transition`, `catalog_snapshot`, `catalog_restart` and `catalog_model` with `--test`. Source and build directories were separate from other candidate branches, and filesystem scratch used the container's owned ext4 mount. This receipt establishes those existing runtime examples only; no new mutation, generated campaign, full crash campaign or resource-ceiling enforcement is claimed. T-12.1 remains under review.
