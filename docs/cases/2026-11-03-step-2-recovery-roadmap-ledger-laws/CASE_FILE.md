# Case file: integration step 2, recovery outcomes, roadmap, ledger, retention laws

- **Date:** 2026-11-03
- **Subject:** `git diff e316d71..ce998c8`, branch `integrate/open-prs`, PR #211 (PRs #190, #196, #197, #194)
- **Evidence:** base `e316d71`, target `ce998c8`

⚠️ ALL VERDICTS HEREIN ARE VALID ONLY FOR THE COMMITS ABOVE.

## Executive Deduction

The merged code is sound and no data is at risk, but its headline claims exceed its evidence: the outcome rule stops short of stage completion, and `KEEP-RETENTION-003` was marked Implemented on selected-field laws. The most consequential open question is whether a retry receipt after a real directory-sync failure proves durability, which remains a conjecture for #214.

## Report Card

| Measure | Grade | Basis |
| --- | --- | --- |
| Fidelity, code (HOLMES) | C+ | Core behaviour faithful; 8 drifts, in prose scope |
| Fidelity, documents (HOLMES) | B | No Done-claim falsified; 15 drifts, mostly staleness |
| Clarity (WATSON) | B- | Honest framing; misled once by the roadmap |
| Resilience (MORIARTY) | B- | 7 medium strikes, none corrupting data |
| **Overall at `ce998c8`** | **B-** | No fatal strike |

## Adjudication

- **Stage completion outcome:** Holmes (D1) and Moriarty (S1) converge independently. A real false diagnostic, not a data risk. Claim narrowed here, fix in #213.
- **Retry receipt:** Holmes marks it unverifiable and Moriarty marks it conjecture. Not tested by anyone. Filed as #214 and does not block this merge, because the protocol predates this PR. CONTESTED until the experiment runs.
- **`KEEP-RETENTION-003`:** Holmes (D6) and Moriarty (S4, S5, S7) converge. Status returned to in progress. Their surviving mutants are deduced from source and were not run.
- **T-16.5 checkbox:** #196's body said to tick it on merge; Watson and Moriarty show its acceptance criterion isn't met. Left open.
- **Row count disagreement:** Moriarty's "four of seven" is corrected by Watson. The ledger's landing table records state by branch.

## The Repair Menu

Applied in the commit after `ce998c8`: M1 to M11. M12 (resolve the #194 thread) follows the merge. Carried forward: C13 to C15 in #213; C16 to C19 in #212; C20 as a note on #204; C21 with #195; C22 with #191; C23 and C24 as filed or noted. C23's roadmap sweep has no issue yet.

## Ship Verdict

**REQUIRES INVESTIGATION**: sound after the named repairs, which are applied. Merge requires green CI on the repaired head.

## Chain-of-custody notes

- Graft MCP was unavailable. Holmes was split into code and documents because the diff exceeded 3,000 lines.
- Nothing was compiled by detectives. CI passed all four jobs at `ce998c8` (run 41670124897).
- Repairs were not re-testified, as the front office told the maintainer in advance.
- One repair wording departs from Watson's: the rationale also names #214. F-25's list and status words are as he proposed.
