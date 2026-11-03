# Deposition of Dr. John Watson: PR #211 (`e316d71..ce998c8`)

I came to this as a colleague who missed the whole business and wants three answers: what changed, what's next, and what can be trusted. Citations are `<filepath>#<line>@ce998c8`. The PR body and `gh` output have no file lines, so I cite those by section or command. Anything I infer is marked *conjecture*.

## What I could work out unaided

**What changed: mostly yes.** The PR body names the four merged PRs and the one edit made on top of them. The three CHANGELOG entries are plain, and one even limits itself ("tests and ledger only") (`CHANGELOG.md#21@ce998c8`). The recovery prose is the best writing here. It says what `Removed` and `AlreadyAbsent` mean, and it refuses to let either pass as a durable receipt (`docs/formats/segment-store-v1/recovery.md#299@ce998c8`, `#325@ce998c8`). The rationale explains why the outcome is kept (`docs/formats/segment-store-v1/rationale.md#204@ce998c8`).

**What's next: only with effort.** The ROADMAP has no "start here" section, and a search for such wording finds nothing. It does have a linked checklist grouped by milestone (`ROADMAP.md#83@ce998c8`) and a dependency map that calls F-21 "the most-cited blocker" (`ROADMAP.md#174@ce998c8`). A patient reader can get from there to a sensible first move. The tasks themselves are excellent once found, with requirements, acceptance criteria, scope and test plan (`ROADMAP.md#702-725@ce998c8`).

**What's safe to rely on: the framing is honest, but the roadmap is wrong on one point this PR ships.**

## Real credit

The ROADMAP says "It is a plan. It is not evidence" and "the ledger wins" (`ROADMAP.md#5@ce998c8`, `#12@ce998c8`). That is the right framing and it largely rescues the page.

- It reports awkward facts unprompted. T-18.2 stays unticked because the ledger still says "crash injection remains" (`ROADMAP.md#944-950@ce998c8`).
- T-01.3 admits no executable law enforces it (`ROADMAP.md#229-232@ce998c8`, line range approximate).
- The salvage ledger says outright that it is "a decision record, not a completion claim" (`docs/audits/pr-107-salvage.md#8@ce998c8`).
- The PR body doesn't claim CI is green and says review is pending. When I checked, Rust quality gates and fuzz smoke were still pending (`gh pr checks 211`).

I'd trust these authors' candour. Their documents have simply fallen out of step with each other.

## Stumbles, ranked by harm

### 1. The roadmap says this PR's own work isn't on main (real harm)

- The ROADMAP says the mutation modules "exist only on the archived #107 branch" and that T-16.5 is "Planned on `main`" (`ROADMAP.md#695-701@ce998c8`).
- The F-16 status and checklist line repeat it, and quote the ledger row as "In progress in #19" (`ROADMAP.md#672-674@ce998c8`, `#108@ce998c8`).
- The same diff adds those exact three files and moves the ledger row to Implemented (`docs/formats/segment-store-v2/requirements.md#14@ce998c8`).
- The CHANGELOG contradicts itself across three lines. The roadmap has "statuses reconciled to main" (`CHANGELOG.md#19@ce998c8`), and two lines down KEEP-RETENTION-003 is implemented (`CHANGELOG.md#21@ce998c8`).

T-16.5 is complexity S with "Dependencies: none" (`ROADMAP.md#723-725@ce998c8`), so it looks like the ideal first task for a newcomer. It is the very task a contributor would most likely pick, and it's the wrong one. "The ledger wins" limits the damage, but only for someone who goes and checks.

The ROADMAP quotes a ledger status as current fact, and that quote is now false.

The PR body half-sees this. It names "documents describing a `main` that has moved" as a risk and says "See Review", but Review says only "pending" (PR #211 body, Failure modes and Review). So a known risk points to an empty section.

### 2. "Implemented for whole-record decoding" isn't explained (real harm for an auditor)

The requirement covers "every structural field, truncation boundary, ordering law…", and the status narrows that with this phrase (`docs/formats/segment-store-v2/requirements.md#14@ce998c8`). In the docs I searched, it appears only in that row and in the CHANGELOG. Nothing says what the phrase excludes.

T-16.5's acceptance criteria also require that "a deliberately weakened decoder fails at least one case per field", and its scope includes "a mutation table" (`ROADMAP.md#704-709@ce998c8`). The ledger row claims neither, and defers `mutations.tsv`. So once stumble 1 is fixed, I still can't tell whether T-16.5 is closed or only mostly closed.

*Conjecture:* it may mean streaming or partial decoding is out of scope. I didn't test that, and I shouldn't have to guess.

### 3. The roadmap and ledger can't tell you what's actually in flight (real harm)

- The legend defines "In review: an open pull request delivers it", but the word is used nowhere except the legend (`ROADMAP.md#34@ce998c8`).
- Live work appears instead as "salvage in progress" with no PR number (`ROADMAP.md#116@ce998c8`, `#1250@ce998c8`, `#1578@ce998c8`).
- PRs #190–#197 aren't mentioned in ROADMAP, CHANGELOG or the ledger. My grep for them found nothing.
- The ledger names only branches, and it has no status column (`docs/audits/pr-107-salvage.md#45@ce998c8`).
- By its own rule a row counts as delivered only "once that branch merges" (`docs/audits/pr-107-salvage.md#8@ce998c8`), but I had to run `gh pr view` on each number to map branches to PRs and states.
- After this PR, four of the eight salvage branches are merged and four are not (#191, #192, #193, #195 were open when I read). No document shows that split.

A contributor could easily duplicate #195 or #192 this way.

### 4. A ticked box doesn't reliably mean done (moderate harm)

- F-25 is ticked but reads "Partially implemented", which isn't a legend word, and T-25.2 is open (`ROADMAP.md#120@ce998c8`, `#1715@ce998c8`, `#1725@ce998c8`).
- F-05 is ticked "Done" with T-05.5 open, because two ledger rows name no test (`ROADMAP.md#91@ce998c8`, `#317@ce998c8`). That strains the legend's "executable evidence" definition (`ROADMAP.md#32@ce998c8`).
- F-03 and F-09 are also ticked with open tasks (`ROADMAP.md#89@ce998c8`, `#95@ce998c8`, `#270@ce998c8`, `#476@ce998c8`). Their notes do at least qualify the tick.

Someone skimming for unticked work will miss these tasks. Someone skimming for what's finished will overcount, and F-25 is the worst case.

### 5. The roadmap never links its companion ledger (moderate harm)

The ROADMAP says "the #107 salvage ledger" over and over (for example `ROADMAP.md#20@ce998c8`, `#207@ce998c8`, `#227@ce998c8`) but never gives the path. `grep pr-107-salvage ROADMAP.md README.md` returns nothing. The only pointer I found in the documents I read is in a table cell of the v2 requirements file. For a page that routes so much through this document, one link would do it.

### 6. Findings are buried in table cells (moderate harm)

Two things an auditor needs are hidden in the "Changes from the branch" column of the Ported table:

- `gc.md` omits that the candidate-set digest hashes a big-endian `u32` count first (`docs/audits/pr-107-salvage.md#53@ce998c8`). My search of `gc.md` found no such sentence, though `#13` does state the general big-endian rule.
- Known same-pattern typed-refusal sites remain, in five named files plus a lossy collapse (`docs/audits/pr-107-salvage.md#54@ce998c8`).

Neither is listed as an open item, and I didn't see either in the roadmap's task list. *Conjecture:* #195 and #191 may carry them, but I couldn't tell that from the ledger.

### 7. Time-bound claims will go stale silently (low to moderate harm)

"So the defect is live on main" is written in the present tense (`docs/audits/pr-107-salvage.md#52@ce998c8`), and this PR makes it false on merge. The roadmap's snapshot is `main at 3887f2a` (`ROADMAP.md#24@ce998c8`), 16 commits behind this PR's base. It discloses the anchor, which is good, but "every status claim has been reconciled" (`ROADMAP.md#17@ce998c8`) reads as more current than it is.

### 8. A dangling reference to a tag (low harm)

"Archived at the same tag" (`ROADMAP.md#22@ce998c8`) refers to no tag, because the sentence before it now cites a branch. The `archive/pr-107-*` tags exist only locally (`git tag -l` versus `git ls-remote`). The audit file really is on the branch, so a reader recovers quickly. Calling a live remote branch "archived" throughout is slightly misleading too.

### 9. PR body slips (low harm)

- "Retention root decoding" under the invariant section undersells #194, which also covers manifest and head (PR #211 body; `CHANGELOG.md#21@ce998c8`).
- Format/API compatibility says "See #190's description". That hides a source-breaking change to exhaustive patterns behind a click, when one sentence would do. The CHANGELOG and rationale do state it.
- "Independently ready" sits right beside "had one open review finding". That is disclosed, but a little loose.

## Taste, not harm

- The ROADMAP runs 2,951 lines, against the project's 500-line limit for files. I can't say whether that limit is meant for docs. Navigation holds up.
- The CHANGELOG entries lack the `(#NNN)` suffix their neighbours carry, and #197's ledger has no entry.
- "Does not itself prove absence of effects" and the rationale's double negative (`docs/formats/segment-store-v1/rationale.md#208@ce998c8`) are dense but accurate. I wouldn't soften them, because precision is the point.

## Where I was actually misled

Only once, by stumble 1. I read F-16 first and believed the mutation matrix wasn't on main, until the diff stat showed otherwise. Everything else made me work harder but didn't lead me astray.

```
VERDICT
witness: WATSON (blind comprehension)
target: e316d71..ce998c8, draft PR flyingrobots/keep#211
clarity_grade: B-
shipped_vs_planned: clear in framing and in most rows; false on T-16.5/F-16 after this merge
fact_vs_assumption: clear; ledger-wins rule stated; "whole-record" scope undefined
contributor_can_find_next: yes with effort; no "start here"; in-flight work not visible
tone_and_honesty: high; no overclaiming found; staleness disclosed but not resolved
stumbles_by_harm:
  1. ROADMAP T-16.5/F-16 say the mutation laws are not on main; this PR ships them; ledger and CHANGELOG disagree
  2. "Implemented for whole-record decoding" undefined; T-16.5 closure ambiguous
  3. "In review" unused; PRs #190-#197 uncited; ledger has no merge state
  4. Ticked features with open tasks (F-25 worst; also F-03, F-05, F-09)
  5. ROADMAP never links the salvage ledger by path
  6. gc.md gap and remaining typed-refusal sites buried in table cells
  7. Present-tense claims and snapshot will go stale ("live on main"; 3887f2a)
  8. Dangling "same tag" reference; "archived" used for a live branch
  9. PR body: root-only invariant, compatibility deferred to #190, Failure modes pointing to an empty Review
misled_once: yes, stumble 1
limits: read-only; did not read all of ROADMAP; did not verify law correctness or mutation strength; one citation range marked approximate
```

## Second Deposition — The Repair Menu

**Nothing here is fatal to the merge.** The merges are clean, the #190 outcomes are true, and neither Holmes nor Moriarty found bytes at risk. What's wrong is prose that outruns the proof. One conditional exception is S2 (M15).

**The honest fix is to narrow the claims in this PR and track the code and test work in issues.** No production code changes here. That's about 45 minutes of doc edits, then a rerun of the documentation gate.

`#RET`, `#REC` and `#DUR` are proposed new issues, and the placeholders need real numbers before merge (see M1). I found no open issue that fits them in a keyword search of the tracker. Line numbers are at `ce998c8`.

### Must-fix before merge, by yield per effort

**M1. File three issues [process, 10 min].** These give the narrowed claims somewhere true to point. Filing needs maintainer authority.
- `#RET` "Complete KEEP-RETENTION-003 law evidence": S4, S5, S6, S7, D4–D8.
- `#REC` "Report recovery effects truthfully in stage completion and refusal messages": S1, S3, D1–D3.
- `#DUR` "Decide whether a retry receipt after a failed directory sync proves durability": S2.

**M2. Reconcile F-16 and T-16.5 [prose, 5 min; my stumble 1, D1, S8].** Don't tick the box, even though #196's body says to. T-16.5's weakened-decoder criterion isn't met for every field (`ROADMAP.md#706`).
- `ROADMAP.md#108`, status text becomes: `Partial (T-16.5: selected-field mutation laws via #194; the complete matrix remains open)`
- `#672-674` becomes: `**Status:** Partial (issue #19, PR #78). Mutation laws for selected fields landed via #194; the `KEEP-RETENTION-003` ledger row owns what remains.`
- `#696-701` becomes: `Partial on main. The mutation modules … [same three paths] … cover selected fields, salvaged via #194; remaining work is tracked in #RET. Original task fields:`

**M3. Narrow the KEEP-RETENTION-003 ledger row [claim-narrow, 5 min; stumble 2, D6, S4, S5, S7].**

`docs/formats/segment-store-v2/requirements.md#14`, evidence wording becomes:
> field-mutation laws (`…three paths…`) over selected fields of the frozen corpus record, which reseal every digest and checksum the mutation did not target and assert the named refusal variant

Status becomes:
> In progress in #RET: selected-field evidence only; full reserved ranges, ordering beyond equal neighbours, truncation boundaries, and overflow are not yet proved at whole-record level; a language-neutral `mutations.tsv` conformance ledger is deferred (see `docs/audits/pr-107-salvage.md`)

*Dependency:* the listed gaps are source readings, and the surviving mutants are deduced, not run. I also haven't confirmed that no xtask check keys on the "Implemented" word, so rerun the gates.

**M4. Rewrite the CHANGELOG entries [claim-narrow, 3 min; D1, D4, S8].**
- `CHANGELOG.md#17`, replace the opening with: `Preserve removal and head-finalization outcomes in recovery stage-discard and next-head-finalization directory-sync errors,` (rest unchanged).
- `#19` ending becomes: `with statuses reconciled to main at the snapshot it names.`
- `#21` becomes:
> Add field-mutation laws for selected version-2 retention root, manifest, and head fields: rows mutate the frozen corpus record, reseal every digest and checksum they did not target, and assert the named refusal variant. KEEP-RETENTION-003 remains in progress (#RET). Salvaged from the closed #107 branch; tests and ledger only.

**M5. Scope the rationale [claim-narrow, 3 min; S1, D1].**

`docs/formats/segment-store-v1/rationale.md#206`: "Recovery therefore preserves" becomes "Stage discard and next-head finalization therefore preserve". Then add at the end of the section:
> Stage completion does not yet follow this rule: `SynchronizeStaging` and `SynchronizePool` omit the outcome, and the staging message says "after {stage} removal" even on a retry that found the name absent. No bytes are affected; #REC tracks the fix.

**M6. Rewrite the PR body [prose, 5 min; S13–S15, my stumble 9].**
- Problem: drop "independently ready". Use `These four PRs were held up by the backlog.`
- Invariant: `#190: stage-discard and next-head-finalization directory-sync errors retain whether the removal or finalization took effect. Stage completion does not yet (#REC). #194: selected fields of retention root, manifest, and head records refuse mutation with the named variant; KEEP-RETENTION-003 stays in progress (#RET). #196 and #197 are documentation.`
- Failure modes: `Open risks are a wrong conflict resolution, which only CHANGELOG.md needed, and documents describing a main that has moved; this PR reconciles those it knows of. See Review.`
- Compatibility: `Source-breaking for exhaustive patterns over RecoveryStageDiscardError::Synchronize and RecoveryNextHeadFinalizationError::SynchronizeRoot: bind outcome or use `..`. No on-disk format change.`
- Review: replace "pending" with the case file's result when it exists.

**M7. Link the ledger and fix the orphan [prose, 2 min; my stumbles 5 and 8, D8, D15, S10].**
- `ROADMAP.md#20`: `salvage is tracked in the [#107 salvage ledger](docs/audits/pr-107-salvage.md).`
- `#22`: `kept on the same branch and is not on `main`.`
- `#17`: `Every status claim was reconciled to `main` at the snapshot below; later merges are named where they change a status.`

**M8. Correct T-22.1a [prose, 1 min; D2].** This is a wrong digest in a format-adjacent claim.

`ROADMAP.md#1274-1276`: "four `definition.tsv` rows" becomes "seven", and `6cbc1c75…` becomes `a4a010ce…`. That matches `#686` and `docs/audits/pr-107-salvage.md#63`.

**M9. Add landing state and fix the ledger rows [prose, 8 min; my stumble 3, D11–D14, S9].**
- Add under the heading `## Landing state`:
> Recorded when #211 was opened; GitHub is authoritative afterwards.
>
> | Branch | PR | State |
> | --- | --- | --- |
> | `salvage/107-recovery-sync-outcomes` | #190 | Landing in #211 |
> | `salvage/107-retention-mutation-laws` | #194 | Landing in #211 |
> | `salvage/107-roadmap` | #196 | Landing in #211 |
> | `salvage/107-ledger` | #197 | Landing in #211 |
> | `salvage/107-typed-refusals` | #191 | Open |
> | `salvage/107-content-store-port` | #192 | Open |
> | `salvage/107-transfer-pipeline` | #193 | Open, stacked on #192 |
> | `salvage/107-gc-retirement-codecs` | #195 | Open |
- `#42`: `#130, #136` becomes `#163, #136`.
- `#52`: replace the cell's first two sentences with: `Shared v1 recovery: stage-discard and next-head-finalization sync errors keep the outcome; stage completion does not (#REC). Main's production files were byte-identical to the fix's parent, so the defect was live on main until this salvage merged.`
- `#61`: `Main lacks `CatalogSnapshot::record_segments`; whether main's live closure-member verification serves the planner is unassessed;` I chose this over Holmes's conjecture about the author's intent, so the author should confirm.
- `#82`: split `34db571` out as its own row: `| landing `34db571` | Source fix to `verification_observation_error.rs`, not a merge; its classification is addressed by `salvage/107-typed-refusals`. |` The #191 link is hearsay from its PR body, so check it. Don't write "superseded", because #191 hasn't merged.

**M10. Fix plain staleness and F-25 [prose, 3 min; D3, D4, D6, stumble 4].**
- `ROADMAP.md#2222`: `0.23.3` becomes `0.24.0`.
- `#471-472`: `Two baselines exist: `benchmark/baselines/c529c07-aarch64-apple-darwin.tsv` and `benchmark/baselines/30ffe90-aarch64-apple-darwin.tsv`.`
- `#120`: untick it, with status `Partial (T-25.2 is echo#722)`. Then `#1715`: `**Status:** Partial; the remainder is in the Echo repository (…`

**M11. Say T-22.1's fields are missing [claim-narrow, 2 min; D5].**
- `ROADMAP.md#41-42`: `Most unfinished tasks carry the full field set (T-01.2, T-03.3, T-05.5, T-22.1, T-25.2, T-41.4, and T-45.3 do not yet):`, keeping the list that follows.
- `#1272`: `Original task fields: see `ROADMAP.md` at `68b81ee` (there titled "GC record codecs and namespace admission").`

*Dependency:* the seven-task list is Holmes's, and I only spot-checked it. Restoring the fields in full goes with #195.

**M12. Resolve the #194 review thread [process, 1 min; S15].** It still reads `isResolved: false`. Resolve it once `ce998c8` lands, and don't describe it as settled before then.

### Carry forward

| # | Repair | Addresses | Effort | Issue |
| --- | --- | --- | --- | --- |
| C13 | Add `outcome` to `SynchronizeStaging` and `SynchronizePool`; correct the message; RED on an `AlreadyAbsent` retry | S1, D1 | S | `#REC` |
| C14 | Fix messages that imply no effect ("was refused", "failed to finalize"), plus `SynchronizeCandidate`'s "was not made durable" and the "durable" comments (`recovery_next_head_finalization_readiness.rs#8`, `recovery_next_head_finalization_storage.rs#18`) | D2, D3 | S | `#REC` |
| C15 | Add public integration laws that assert `outcome` | S3 | S | `#REC` |
| C16 | Full-range reserved-byte laws; a descending-anchor law; below-header truncation through `decode`; overflow | S5, D6 | M | `#RET` |
| C17 | Assert expected and observed values on the rows that elide them, and remaining inner causes | S4, D5 | S–M | `#RET` |
| C18 | Rename `*_refuse_after_complete_integrity`, or add laws that actually discriminate ordering | S6, D7 | S | `#RET` |
| C19 | Move `KEEP-RETENTION-006` off closed #19 | D8 | S | `#RET` |
| C20 | Carry the corrected row and the mutation evidence paths into `docs/formats/store/requirements.md` on rebase; re-reconcile the roadmap | S11, S12 | S | #198 |
| C21 | Restore the gc.md digest sentence (`docs/audits/pr-107-salvage.md#53`) | stumble 6 | S | #195 |
| C22 | Track the remaining typed-refusal sites (`docs/audits/pr-107-salvage.md#54`) | stumble 6 | S | #110 |
| C23 | Roadmap sweep: PR numbers at `#116`, `#1250`, `#1578`; `#179` F-24/#72 self-dependency; `#223` ADR-0010; `#331` and `#2245-2246` issues still open though described as closed or delivered | stumble 3, D7, D9, D10 | S | #132 (fit unverified from title) or new |

**C24 / M15. S2 is the only item that could become serious [experiment, L; `#DUR`].** It is conjecture. The retry protocol predates #190, though this PR pins it in a law. Don't change code here. Run Moriarty's dm-error experiment and record the decision. If it confirms, `recovery_next_head_finalization_receipt.rs#8` is making a false durability claim, which breaches the Core Law, and that blocks any durability promise, though not this merge. Filing `#DUR` before merge is the must-fix part.

### Corrections and caveats

- **My error:** I gave no line for one T-01.3 range and flagged it approximate. It doesn't affect any repair.
- **Moriarty's "four of seven":** three ported rows land here (recovery, mutation laws, roadmap), plus the ledger, which isn't a ported row. M9 records the states by branch.
- **Moriarty's "19 of 51 rows":** I didn't recount this, so no repair wording uses the number.
- **CI when I last looked:** Rust quality gates pending; documentation, dependency policy and fuzz smoke passing. That is hearsay and unpinned, and M3/M10 need a rerun of the documentation gate either way.

```text
VERDICT
witness: WATSON (repair menu)
target: e316d71..ce998c8, draft PR #211
fatal: none (S2 conditional on experiment; pre-existing protocol)
must_fix_before_merge: M1-M12 and filing #DUR
carry_forward: C13-C23, C24 (#REC, #RET, #DUR, #198, #195, #110, #132)
production_code_in_this_pr: none; narrow the claims and track the fixes
do_not: tick T-16.5; keep "Implemented" on KEEP-RETENTION-003
effort: about 45-50 min of docs and process, plus a gate rerun
unverified_dependencies: gate behaviour on the status word; T-22.1 field list; #191 link at row 82; #132 fit; S2; all mutants deduced, not run
```
