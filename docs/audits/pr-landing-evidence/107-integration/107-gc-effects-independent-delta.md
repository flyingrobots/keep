# Independent scoped confirmation: GC evidence correction

Reviewed exact clean successor `f6aca41fd562915c81233206b5fdf0592d15f60d`, delta from `20dec6d2adfd5fcb9e59aa3e784985dada6e3a01`. Both isolated review and author checkouts are clean at the successor. This documentation-only confirmation follows the full independent GC reporting review, published before the fix at [the exact-head review comment](https://github.com/flyingrobots/keep/pull/107#issuecomment-5978759478). It uses the same authorized independent Codex fallback and mandatory agy-review protocol.

The prior P4 is closed. No verified P0–P5 finding remains in this finite successor delta.

## Verification Checklist

| Mandatory area | Verified result |
| --- | --- |
| Complete delta / every changed path | The complete diff changes only `docs/testing-evidence/m4-integration.md`: the evidence paragraph at `:130` and an added review-provenance paragraph at `:144`. No source, tests, manifest, raw receipts, format documentation, rationale, dependencies or configuration change. |
| Production paths / parallel paths | All 14 capabilities, public execution, generic execution/resume, fresh recovery, shared-stage dispatch and cause/progress paths are unchanged. The full path-by-path checklist in `107-gc-effects-independent-review.md` at exact `20dec6d...` carries forward for this identical implementation; no new runtime path needs certification. |
| Merges / inherited invariants | Successor has the exact single parent `20dec6d2adfd5fcb9e59aa3e784985dada6e3a01`; there is no merge or conflict resolution in this delta. Prior merge and identity/no-follow/bounded-admission/source-preservation constraints remain as recorded in the full review. |
| P4 claim against actual assertions | Paragraph `:130` now assigns public execution error, context invalidation, exact live bytes, candidate bytes or absence, retained intent bytes and fresh recovery/Complete to the eight injected schedules. These match `src/adapters/gc/filesystem_gc_effect_tests.rs:142`, `:158`, `:159`, `:168`, `:169`, `:175`, `:187`, `:190`, `:211` and `:218`. Receipt-byte preservation is correctly conditional on a receipt surviving failure. |
| Narrow pre-effect law | The paragraph separately identifies the generic executor, original typed refusal, no failing-call removal effect, exact contradictory candidate, retained intent pathname and no receipt publication. These match `filesystem_gc_effect_tests.rs:297`–`:335`. It expressly excludes claims of live/intent byte preservation, public context invalidation and restart behavior for this law. This closes the demonstrated overstatement without adding unsupported evidence. |
| New provenance claim | Paragraph `:144` accurately identifies the prior head and single P4 finding, links the published review and leaves runtime receipts at their original source coordinates. Read-only GitHub comment lookup confirmed that comment's exact-head and P4 content. The one-file diff independently establishes the documentation-only claim. |
| Constants / numbers / evidence | No runtime or resource constant changes. Eight injected schedules, five directory-sync phases and the separate pre-effect/source-substitution laws remain consistent with the prior source/receipt checklist. No historical test count, timing, launch measurement or numeric receipt is rewritten. All 12 entries in the unchanged `107-gc-source-sha256.txt` were independently verified again and match. |
| Errors / state machine / durability | No source changes: original detailed reports, known versus uncertain unlink outcomes, unconfirmed durability, first-error stop and fresh observation remain exactly as reviewed. No new rollback, power-loss, raw-writer isolation or retrospective sync-certification claim is introduced. |
| Repository standards / prose | Independent successor `git diff --check` passes. Paragraphs use the existing physical-line format. The supplied documentation-worker receipt reports zero Markdown issues; it was inspected, not independently rerun. Existing testing-policy requirements and coverage gaps remain binding and unwaived. |

## Executed versus inspected and limits

Executed independently for this successor: exact clean SHA/parent checks in both checkouts, complete one-file delta inspection, source/test/manifest non-change comparison, all 12 source-hash checks, successor diff whitespace check and read-only lookup of the published prior review comment.

Inspected only: `107-gc-review-doc-fix-markdown.txt`, which lists the selected documentation input and reports zero issues. No Rust command, Docker restart, runtime mutation, host test or new worker/cache was used.

The independently executed 19-law GC run remains pinned to `20dec6d...`: 19 passed, zero failed, 489 filtered, guarded exit 0. Its source and tests are byte-identical at this successor; the receipt is not relabeled as a new successor execution. Prior author RED, debug/release GREEN, three controls, static checks and their exclusions remain exactly as described in the full report. The raw patch-context and launch-command whitespace disclosed there is unchanged; the present documentation-only delta is clean.

All earlier resource limits and measured usage remain historical, not newly measured here: existing Rust 1.96.0 worker, four CPUs/8 GiB, two jobs, incremental off/offline, shared bounded target; 20 GiB aggregate build, 4 GiB runtime, 128 MiB logs, 50 GiB minimum host/VM free and 900-second fail-closed guard. Prior focused run was explicitly released and the parent stopped the idle worker. No workload is active on behalf of this review.

Separate disposition reporting, candidate/intent identity and nested-authority obligations, per-test sandbox/resource/egress and suite-budget policy disposition, whole-candidate crash/fuzz/dependency/final validation and hosted pushed-head CI remain open. Discussion was not exhaustively refreshed again for this one-paragraph evidence correction; the prior full review's exhausted 44-comment/17-review/eight-thread queue remains its pinned discussion evidence, supplemented by confirmation of the published review link. No whole-#107 acceptance, policy waiver, publication or merge authorization follows.

## Scoped verdict

Approve the documentation correction at exact `f6aca41fd562915c81233206b5fdf0592d15f60d`; the prior P4 is closed. The finite GC reporting slice's unchanged implementation and prior full checklist carry forward with all stated limitations.

APPROVE
