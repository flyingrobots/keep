# Open pull request landing pass

This ledger records the maintainer-authorized landing pass following the completed-roadmap audit (#131/#132). Each candidate receives a fresh Code Lawyer review, complete feedback reconciliation, current-head validation and an effective independent approval before a normal merge. The maintainer explicitly authorized independent Codex review using the agy-review protocol and appropriately sized models. No force push, history rewrite or repository-rule bypass is authorized.

The starting mainline is `6051abb25a9fd33ae7ee0de5614514b709a4d82a`. Open correction PRs at the start are #133, #156, #157, #158, #159, #160, #161, #162, #163, #164, #165, #167, #170 and #172. Current state and final evidence are recorded below; green earlier heads are not transferable to changed candidates.

## Candidates

| Candidate | Current disposition | Evidence and next action |
| --- | --- | --- |
| #172 / #171 | Merged as `d0cff10d7c911d33d615c3aa2246ae2b4497432a` | [Fresh independent GPT-6.1 high-reasoning APPROVE](https://github.com/flyingrobots/keep/pull/172#issuecomment-5972433501) and [Code Lawyer closure](https://github.com/flyingrobots/keep/pull/172#issuecomment-5972433684) cover candidate `07e6cc4875c05592b71bb1f8b9c80631a31bcda5`. Required candidate checks passed. Signed normal merge has exactly the reviewed tree. [Mainline run 37146130023](https://github.com/flyingrobots/keep/actions/runs/37146130023) completed with all four jobs successful. |
| #158 / #146 | Integrated current main in candidate `e781c0b276ec4d1f66a76668bd31893261a2e6dd`; fresh independent review pending; hosted validation and CodeRabbit review passed | Full copied-Docker validation passed, including both crash campaigns, both feature checks/Clippy profiles, debug/release workspace tests, all-feature doctests/docs and fuzz-target compilation/linting. [Hosted run 37146207010](https://github.com/flyingrobots/keep/actions/runs/37146207010) passes all four jobs on the updated head; CodeRabbit approves with no actionable feedback. The only text conflict was the changelog; both entries were retained. The sealed-stage and crash-observer implementation is unchanged by the merge. |
| #157 / #150, #133 / #110 | Queued correctness candidates | Reconcile current full feedback and integration against the then-current mainline before landing. |
| #156 / #147, #159 / #128, #160 / #113, #161 / #111, #162 / #112, #167 / #166, #170 / #169 | Queued evidence candidates | Fresh review and current integration evidence required; existing approvals are inputs, not a new landing verdict. |
| #164 / #109, #165 / #114 | Queued API candidates | Preserve independent scope and source-specific validation. #164 retains an earlier GitHub changes-requested state requiring explicit reconciliation. |
| #163 / #130 | Queued documentation candidate | Reconcile claims with the actual integrated mainline so landing does not restore stale status. |
| #107 and #141 | Unfinished branch/audit work; preserve | Do not treat an open PR as acceptance. Evaluate remaining obligations and superseded slices after focused corrections land. |
| #93, #94, #102, #103, #104, #105, #106 | Dependency candidates not yet reviewed in this pass | Determine compatibility, overlap and current validation before deciding whether to land or explicitly defer. |

The separate duplicate-record/incomplete-tail defect #173 remains unresolved. Landing #172 corrects observed partial-seal fixed framing; it does not certify all recovery invariants or reopen #99's approved incomplete-retention-stage disposition deferral (#155). The broader completed-task audit remains unfinished.

## Validation boundaries

The fresh #172 focused run used the existing clean copied Docker tree whose tracked tree hash `af00023bb50da6b5f390fc8f2e4b070bf15ceb52` equals the candidate's tree. Its synthetic copy commit differs from the source commit; tree identity was checked before execution. An initial command named a nonexistent test target and ran no tests; that setup failure is retained separately and is not runtime RED. The corrected command executed the intended named laws in both profiles and returned success.

The complete prior local validation and exact-head hosted run `37097418652` remain attributable to the same unchanged source. The fresh focused run supplements those results; it does not claim another full crash/fuzz campaign, physical power-loss evidence, or universally enforced test resource isolation. The landing review also distinguishes CodeRabbit's optional docstring percentage warning from repository-required public documentation and successful documentation CI.
