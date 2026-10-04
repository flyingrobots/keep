# Store initialization audit

This page owns the T-13.1 verdict from the originally checked roadmap at `1a586d83d5750083172d440f90e7b786d540ff0e`, lines 565–566. It inspects main `6051abb25a9fd33ae7ee0de5614514b709a4d82a`. T-13.2 through T-13.4 require separate verdicts; this page does not close recovery or the process-death matrix.

## T-13.1 — Ordered, idempotent, writer-locked initialization

**Acceptance/definition of done: not fully evidenced.** The original task names `initialize_store` and `KEEP-RECOVERY-002` through `-004`; it has no additional per-task definition-of-done fields. The implementation performs the named transitions and preserves the identity refusal, but the cited replacement experiment does not demonstrate that the authority-producing path respects that refusal. No current production bug is alleged.

| Criterion | Inspected implementation and evidence | Verdict and limits |
| --- | --- | --- |
| `KEEP-RECOVERY-002`: platform, writer lock, ordered directories, synchronization receipt, precise first failure | `store_initialization.rs` calls the semantic port in the prescribed order and returns only after root sync. `tests/store_initialization.rs` injects each phase failure and checks its exact phase and absence of later transitions. | Runtime protocol evidence passes debug/release. These are contract-boundary port interactions, not test-harness case counts or actual syscall-failure injection. |
| `KEEP-RECOVERY-003`: admitted platform, canonical partial namespace, no evidence replacement, exclusion and published reopen | `filesystem_store_initializer.rs`, `filesystem_initialization_storage.rs` and `filesystem_initialization_namespace.rs` retain authority through sync, require canonical types/membership and distinguish initialization from published reopen. `filesystem_platform_profile.rs` checks existing child device/mount identity and writable non-casefolded ext4 properties before initialization. | Filesystem initializer and platform-policy laws pass debug/release. Initializer fixtures deliberately bypass production platform probing; policy-property tests are simulated observations, not mounted-filesystem experiments. Public writer tests separately confirm exclusion, preserved bytes and no-follow behavior. |
| `KEEP-RECOVERY-004`: do not return authority after opened/selected lock identity disagreement | `FilesystemWriterLock::acquire` locks the opened file, calls `verify_current_identity`, propagates its failure and only then constructs the guard. The cited unit law calls the checker directly; the public replacement law substitutes after a first guard already exists and proves root-lock exclusion. | The current code is correct by inspection, but the cited runtime suite survives ignoring the production identity refusal. [Issue #169](https://github.com/flyingrobots/keep/issues/169) owns the missing acquisition-boundary experiment and its calibration. |

The separate feature-gated publisher-admission bypass remains owned by [#150](https://github.com/flyingrobots/keep/issues/150). It is not relabeled as a new initialization defect or duplicated here. Production platform admission, observer helpers, repository-task fixtures and returned writer authority are distinct boundaries.

## Surviving mutation and required correction

In an isolated Docker copy of the inspected main source, replacing only `verify_current_identity(&directory, expected_identity)?;` with `let _ = verify_current_identity(&directory, expected_identity);` leaves the private replacement checker law, public catalog writer-lock laws, initialization port laws and filesystem initializer laws GREEN. The mutation still performs the observation but allows writer authority to escape after a mismatch. A checker-only assertion cannot detect its caller discarding the refusal.

An earlier mutant removed the call entirely and encountered dead-code lint during public integration compilation. That attempt is retained separately and is not runtime calibration evidence. The ignored-result mutant compiled and executed the cited tests successfully. Original source was restored with timestamps invalidated, and public lock laws passed again in debug/release.

The correction must exercise deterministic entry replacement between opening the file and the post-acquisition guard through the actual authority-producing boundary, assert exact refusal and retained evidence, and reject the surviving mutant. It need not invent a production fix or freeze helper structure. Deletion or replacement of the old checker-only test must name the stronger law and its limits.

## Execution and scope

Copied source and a dedicated build directory were used for main `6051abb`. Debug/release runs passed `store_initialization`, `filesystem_store_initializer_tests`, `filesystem_writer_lock`, `filesystem_platform_profile` and the public `catalog_writer_lock` target. No host Rust execution, ambient sleeps or source-string assertion was used as runtime evidence. The mutation receipt is limited to the named debug suites; it is not a claim that every repository test would survive.

These experiments do not prove physical power-loss persistence, arbitrary out-of-band namespace isolation or every platform's syscall behavior. The roadmap checkbox remains unchanged; integration of #169 and a reviewed replacement receipt are still needed to close this evidence gap.
