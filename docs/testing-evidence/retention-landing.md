# Retention recovery landing ledger

Owner: `@flyingrobots`. Change kinds: approved behavior change for incomplete stages, bug fixes for source binding and execution-failure reporting, and documentation reconciliation. This is the single closure ledger and evidence receipt for the bounded #99 landing.

## Baseline and approved contract

Baseline inspection found clean local `51bae7f0bdabaeec0f9700358649bc2657b00aa1` and pushed `821c60d0ba48ae4963cdeb8719a2598aafcc0249`. Local `aa6999c` and `51bae7f` preserve the unfinished framing correction; they remain in history. Target main was `379b24141bbcc4b883bd169e746ed76c77f7699c`.

Maintainer decision A supersedes automatic incomplete-stage disposal for this landing: incomplete stages require explicit disposition before any recovery mutation, while demonstrated corruption keeps its precise refusal. Finding no contradiction does not prove canonical completion. Automatic disposition and stronger completion feasibility are deferred to a focused follow-up; no new namespace, journal or wire format is introduced.

Decision B supports cooperating writers under Keep authority in a managed namespace. Concurrent raw namespace mutation is outside writer-lock isolation. Existing no-follow, identity, byte, namespace and corruption checks remain required; neither a handle nor a metadata check makes pathname mutation conditional on inode identity.

Decision C distinguishes pre-effect refusal from execution failure. Planning refusal initiates no recovery mutation. Execution errors preserve their cause and report the failed boundary, completed earlier steps, known effects and uncertain effects/durability of the failing capability. Failure is not rollback; retry requires fresh observation.

## Review reconciliation

The baseline inventory includes all 40 inline threads with their complete comments, 73 review records and 133 top-level comments, verified against paginated API results. These are discovery coordinates, not runtime assertions. Inline IDs below omit the common `PRRT_kwDOTdinZc6` prefix. The independent review at [5946292798](https://github.com/flyingrobots/keep/pull/99#issuecomment-5946292798), its primary reconciliation at [5946293023](https://github.com/flyingrobots/keep/pull/99#issuecomment-5946293023), review-body outside-diff findings and later activity/self-findings are included. Resolution flags alone do not close an obligation.

| Obligation / sources | Invariant and baseline evidence | Disposition / remaining implementation | Concrete exit |
| --- | --- | --- | --- |
| Short-stage destruction: g09p5; independent review; partial-prefix activity comments | Existing corruption checks preserve many contradictory prefixes; planner still schedules automatic discard | OPEN: decision A; refuse incomplete stages and disable lower filesystem discard capabilities; retain diagnostic laws | Direct/publication recovery preserves all stages, including maximum-namespace future-entry counterexample; no discard path executes |
| Incomplete pinning / earlier-prefix discard: oR9Xz, g09qR | Discard stores metadata without a retained handle | APPROVED DEFERRAL of disposal, not a completed pinning implementation; no automatic deletion under A/B | Normative contract and requirements mark disposal deferred; retained-stage refusal tests pass |
| Source-stage removal binding: maintainer inspection | `FilesystemRetentionStage::remove` verifies destination but not source immediately before unlink | OPEN bug fix: retain handle and verify source as well as pool evidence | Replaced source after observation refuses before unlink with exact identity cause and unchanged evidence; RED on baseline |
| Execution partial effects: maintainer C; prior global recovery gap reports | `RetentionRecoveryError` records previous completed steps only; capabilities can fail after link/rename/unlink | OPEN bug fix: finite capability boundary/effect inventory, typed failing-capability status, fresh observation | Pre-effect, post-effect, sync failure and restart laws pass with exact causes and honest effects |
| Namespace and directory admission: g09p8, g09qo; independent review | Recovery starts with pinned-directory and recovery-census admission | IMPLEMENTED, retain; final stable runtime suite verifies both direct and publication paths | Namespace/directory refusal laws pass unchanged |
| Pool inode binding: g09qD; independent review | Observation binds exact pool bytes and identity to stage | IMPLEMENTED, retain | Byte-equal pool substitution refuses before head finalization |
| Recovery file synchronization: g09qI; independent review | Recovered stages synchronize before publication | IMPLEMENTED, retain; process/syscall evidence is not power-loss proof | Existing syscall-order and restart tests pass |
| Head coordinates / root history / successor entries: g09qX, g09qc; global 5950156938 | Planner compares head coordinates, successor root generation and unrelated manifest entries | IMPLEMENTED, retain | Exact history/binding/entry-set refusals and legitimate committed cleanup pass |
| Predecessor and live closure: g09qi, g09qv; global 5951046189 | Selected roots reopen; forward and recovery paths share bounded live closure admission | IMPLEMENTED, retain | Missing/corrupt predecessor, catalog and segment laws pass |
| Typed planner/storage/observation failures: g0-Vj, g0-Vr, g0-VQ, g0-VL, g0-VH | Dedicated storage causes, missing-root diagnostic and filesystem corruption laws exist | IMPLEMENTED, extend only for C | Original error variants/sources and corruption preservation remain observable |
| Reader capability, binding, coordinates and errors: g2O5E/g2pTv, g2pT1, g2O5N/g2pT6, g2pT_, g2O4x | Reader uses pinned root, migration binding, catalog-length coordinates, typed catalog error; fence is crate-private | IMPLEMENTED; duplicate sources grouped | Stable reader suite and public API checks pass; no new isolation claim |
| Reader/model test oracles: g2O40, g2O42, g2O5B, g2O5Q, g2pUF | Exact checksum/errno/identity and collector/model refusal laws implemented | IMPLEMENTED, retain | Existing boundary tests pass |
| Fixture, codec bounds, constructor and migration error consistency: g0-U3, g0-VB, g0-VW, g2O4u, g2O5S, oTAeP | Shared admission/phase vocabulary, codec bounds and typed migration errors exist | IMPLEMENTED, retain | Stable workspace checks pass |
| Executor first failure: g0-Vc | Existing test covers empty prior completed-step prefix | IMPLEMENTED, clarify under C: empty prefix says nothing about failing capability effects | New partial-effect law distinguishes those facts and verifies stop-on-error |
| Recovery process-death state oracle and successor claims: g2O5Y, g0-Ut | Reader checks recovered generation/root; initial and successor publication-phase coverage expanded | IMPLEMENTED; update partial-write expectations under A | Complete-stage restart success retained; partial writes refuse without deletion |
| Merge integration: independent review; globals 5946442838, 5946468733, 5947325527 | Main merged without rewriting history; mount stability, migration crash coordinates and Clippy duplicate arms corrected | IMPLEMENTED; previous case-count assertion is not durability evidence | Stable candidate builds, merge invariants and required crash suites pass |
| Documentation and outside-diff comments: g0-Uw, g2O4s, oR9X3; review-body lib.rs/recovery.md/crash CLI | Status, count and boundary claims previously corrected; A/C now supersede discard promises | OPEN reconciliation across normative docs, requirements, API, PR description and historical receipts | No current claim of automatic disposition, rollback or raw-mutation isolation; historical evidence clearly scoped |
| Current-head acceptance | Earlier agy review requested changes; earlier CI applies only to earlier heads | OPEN: exact-head agy under A/B/C, complete checklist, required checks | Exact pushed SHA approved and green; human merge approval remains required |

Independent-review duplicate findings map to the rows above. Its requested rebase and claimed 300-line hard limit were rejected in the original reconciliation: preserve history with normal merges; 300 lines is a review threshold, 500 the hard file limit. No repeated hardening pass is authorized after the landing exits pass.

The separate catalog publisher admission issue #150 remains tracked outside this recovery landing; its public unchecked-constructor scope is not discharged by these tests. It is not a new recovery-disposition requirement.

## Landing evidence

Pending implementation and validation. Historical per-field receipts remain preserved as evidence of their named assertions at their recorded commits; they do not prove current automatic-discard safety, exhaustive prefix completion, or the approved landing complete.
