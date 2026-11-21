# Keep PR landing handoff — 2026-10-04

The user has stopped for the day. The merge goal is PAUSED. Resume implementation only when the user asks to continue. This handoff records the state after publishing the disposition RED checkpoint; it is not a merge approval or a claim that the outstanding PRs are complete.

## Resume prompt

Resume the existing goal: merge the existing open PRs into main after another Code Lawyer pass, clean independent review and green required checks. Preserve all work and the original acceptance scope. Start with the committed disposition regression below, not a new general audit. Use the existing closure ledger at `docs/audits/pr-landing-2026-10-03.md` on the audit branch. Do not reinterpret previously scoped approvals as whole-PR approval, discard the RED checkpoint, or restart already-closed findings without new evidence.

Read the applicable global and repository AGENTS instructions, Code Lawyer and agy-review skills. Independent Codex is an explicitly authorized fallback for agy; use an appropriately sized model, read-only review and the same adversarial prompt/checklist. Existing merge authorization persists once all gates pass. Do not bypass protections, hooks or testing-policy requirements. Commit completed work at turn boundaries; keep permanent regressions and substantial evidence in Git. Docker executes copied source, rather than serving as the sole home of tests or fixes.

## Exact saved state

| Artifact | State at handoff |
| --- | --- |
| Implementation branch | `landing/107-integration`; clean worktree, local and pushed head `f21589f00de89fd490463de7d9407dea693f68cc`. |
| Remote checkpoint | [Published branch](https://github.com/flyingrobots/keep/tree/landing/107-integration), [RED checkpoint commit](https://github.com/flyingrobots/keep/commit/f21589f00de89fd490463de7d9407dea693f68cc). This is an intentionally failing regression checkpoint, not merge-ready work. |
| Default workspace branch | `audit/131-completed-roadmap-criteria`; its pre-handoff head is `1f242bfc229002a1a24a9c2388a4c4f5225d44d9`. This handoff is a documentation-only successor on that branch. |
| Main | Last verified live Git ref and GitHub branch API: `3165890e9291cfb5fe10e81a9d7cd151f3e59464`. Refresh before integration. PR metadata reports an older `baseRefOid`; do not mistake it for a newly advanced main. |
| PR #107 | Still targets `main` from `feature/roadmap-m4-tasks`, last verified pushed head `7cdc2cfbef59407f3c38881b39a290cb85501f9b`. Publishing the separate landing branch did not update this PR. |
| PR #133 | Still stacked on #107 at `66c0e4653424cd36e55a868c24d94898a40aca59`; already included by a normal merge in the local/published landing history. It has not been separately merged into main. |
| Checkouts | Run `git worktree list` and reuse the existing author, audit and review checkouts. The author checkout is named `keep-integration-107`; the isolated reviewer checkout is `keep-review-107`. Both share the repository Git database with the default workspace. Do not reset, rebase, amend, force-push or delete worktrees to simplify the history. |
| Workers | `keep-validation` and `keep114-closure-docs` are stopped. No owned build, campaign or independent review remains active. Their exit 137 follows deliberate Docker stops; do not relabel that as an observed OOM. |

## Immediate next work: disposition failure reporting

Disposition is the existing explicit finalize-or-retire operation for a complete retention artifact left by interrupted publication. This work repairs its failure reports; it does not add automatic deletion of incomplete retention stages. The verified scenario is that pool unlink succeeds, then namespace synchronization fails, but the public error contains no failed-capability progress.

The permanent test is `src/adapters/retention/filesystem_retention_disposition_effect_tests.rs`, registered under the existing disposition test module. Its name is `failed_namespace_sync_reports_the_removed_pool_artifact`. The source, diagnostic scaffold, actual failing receipt, launch record, source manifest and reproduction instructions are all committed at `f21589f`; nothing must be reconstructed from a temporary Docker layer or an editing script.

Replay inside a copied, guarded Docker checkout of that commit:

```sh
cargo test --locked --offline --lib failed_namespace_sync_reports_the_removed_pool_artifact
```

Observed result: one test executed and failed at the intended boundary assertion, `None` versus `Some(NamespaceSynchronization)`; 508 other tests were filtered. Before failing, it observes the actual artifact removal, surviving namespace, a decodable retirement receipt and unchanged published head. Later known-effect/durability and uncertainty assertions have not yet executed and must not be claimed calibrated. This is deterministic EIO checkpoint injection over real filesystem effects, not a kernel EIO or power-loss experiment.

The checkpoint contains forwarding dispatch in `filesystem_retention_disposition_execution_storage.rs`, production-inert checkpoints and a test-only selector on the authority, `RecoveryDispositionError::storage_progress()`, new boundary/effect vocabulary and the unchanged old unannotated operations. It compiles and reaches runtime RED. The reporting fix, broader focused tests, final Clippy/format/structure/doc checks and independent review remain unfinished. See the final section of `docs/testing-evidence/m4-integration.md` on the landing branch and its `107-disposition-*` artifacts. The [source finding](https://github.com/flyingrobots/keep/pull/107#issuecomment-5978805029) was posted before implementation.

Continue with this bounded sequence:

1. Annotate the ten existing disposition capabilities, preserving original typed causes and pre-existing shared-stage reports. Distinguish preparation refusal, failed attempted unlink, successful unlink followed by failure, successful namespace sync before a later failure, and namespace removal whose parent sync has not completed. Earlier successful calls are not themselves proof of effects or durability.
2. Cover the finite root/manifest pool-removal and synchronization boundaries with deterministic runtime laws. Keep first-error stopping, retained receipt evidence and fresh-observation/restart outcomes explicit. A second invocation that refuses must not be described as successful recovery. Do not assert speculative rollback or raw-namespace isolation.
3. Keep the committed RED revision; implement GREEN in later commits. Calibrate distinct load-bearing effects, original causes and boundary assertions with reproducible controls, rather than another campaign per field. Run focused checks during editing and the full required chain once the complete candidate is stable.
4. Reconcile evidence, public API docs, normative protocol and the existing ledger. Obtain independent exact-head review, publish the full feedback before fixing findings, and confirm the resulting head after any review delta. Only then proceed to the remaining landing obligations.

## Completed adjacent work: do not repeat it without cause

The GC reporting implementation is `20dec6d2adfd5fcb9e59aa3e784985dada6e3a01`, with documentation-only correction `f6aca41fd562915c81233206b5fdf0592d15f60d`. The independent reviewer ran all 19 focused GC laws successfully and found only a P4 evidence overstatement. That prose was corrected; [exact-successor scoped APPROVE](https://github.com/flyingrobots/keep/pull/107#issuecomment-5978784924) closes the GC reporting slice. Debug/release, workspace Clippy, format, structure and changed Markdown passed at the recorded coordinates. The scaffold now layered above it is not covered by that approval.

Shared stage creation/receipt-effect reporting has [scoped approval at `cdc7b64`](https://github.com/flyingrobots/keep/pull/107#issuecomment-5978541881). The normal #133 integration and verification-classification correction have [approval at `34db571`](https://github.com/flyingrobots/keep/pull/133#issuecomment-5978418730). Earlier compaction identity, execution-progress and bounded-reread closures are recorded in the existing audit ledger and review artifacts. These approvals retain their exact scope and source coordinates.

## Remaining merge queue and gates

The last verified queue has seven open PRs: #179, #141, #133, #107, #102, #94 and #93. Refresh GitHub state before acting. No PR merged in the last work session. #141 is the unfinished completed-roadmap audit; do not count it as a finished implementation. #133 remains stacked on #107. #102, #94 and #93 have recorded prerequisite relationships through #179/#102; use the ledger rather than assuming independent mergeability.

For #107, remaining obligations include disposition execution-failure reporting, direct recovery-observed GC/disposition substitution coverage and nested-directory authority binding, reconciliation of original GC/migration/fuzz acceptance, all review bodies and top-level discussion, testing-policy disposition, complete stable-candidate validation, independent whole-candidate review and required CI on the final pushed PR SHA. The landing branch already includes #133; updating #107 may affect the stacked PR's GitHub state. Account for that deliberately rather than treating this RED checkpoint as a publishable final candidate.

The #106 test-resource waiver was explicitly approved only for that dependency update and expires 2026-10-17. It does not cover #107 or #179. #179's separate waiver request remains unapproved in the recorded conversation; its implementation/CI evidence is otherwise in the ledger. `docs/testing/enforcement.md` requires explicit maintainer approval or actual enforcement. Neither author-written expiry nor aggregate Docker limits waive per-test sandbox, resource-ceiling or suite-budget obligations. Do not ask for an already approved action again, but never invent this missing approval.

Maintainer decisions remain binding: preserve incomplete retention stages before any recovery mutation; keep precise corruption separate from unsupported incomplete state; support cooperating writers under Keep authority without promising isolation against arbitrary raw namespace mutation; preserve identity/no-follow/exact-byte checks; distinguish pre-effect refusal from execution failure, known from uncertain effects, and failed synchronization from rollback. Automatic incomplete-stage disposition remains deferred. Do not add a quarantine namespace, journal or new format as a shortcut.

## Docker and resource handoff

Read the now-matching global Codex, Claude and agy resource rules. Reuse `keep-validation` with `rust:1.96.0`, four CPUs, eight GiB RAM, two Cargo jobs, offline dependencies and incremental compilation disabled. The existing source/target live under `/keep-validation-cache`; the lock is `/var/lock/keep-validation.lock`. The cache backing filesystem is 20 GiB. Keep all builds/reviews serial under that lock; no per-PR image, worker, volume or target proliferation.

The project budgets are aggregate build caches at most 20 GiB, runtime data at most 4 GiB including logs at most 128 MiB, with at least 50 GiB free on host and Docker backing storage before heavy work. Active fail-closed monitoring, explicit output ownership, bounded logs and child-process termination on breach/timeout are mandatory. Named volumes, container memory limits and a written budget do not enforce disk limits.

The audit checkout has the reusable copy runner in `scripts/docker/cargo.sh` and `cargo-worker.sh`. The existing local operational wrapper is `~/.codex/scratch/keep-landing/107-bounded-focused.sh`; inspect it before use, as it names the author checkout and existing worker. It records each launch contract, checks aggregate usage and enforces a 900-second command deadline. This wrapper is operational scratch, not the source of the regression; the committed test can be replayed with the command above in another equivalently guarded environment. Never run Rust tests on the host or mount the host repository writable for validation.

Historical `/build` archives were approximately 7.9 GiB and `/tmp` archives approximately 3.5 GiB at the last measured review, including retained source/evidence. The reusable Cargo target was approximately 906 MiB. The approximately 15.2 GB writable layer includes the bounded cache backing file; do not double-count that file and its mounted contents. These measurements predate the final disposition checkpoint: refresh all usage before another heavy command. Preserve archives and unique evidence; no broad prune or speculative deletion is authorized. Both workers are stopped at handoff.

## Stop condition

Finish the finite ledger under the approved scope, obtain independent review of the exact resulting candidate and verify all required checks on its final pushed SHA. Merge only eligible PRs under the existing authorization and repository protections. Do not start another speculative hardening pass after those conditions are met. Until then, report specific open obligations rather than a completion percentage or “almost done.”
