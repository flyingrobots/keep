# Case file: integration step 1, dependency bumps

- **Date:** 2026-11-03
- **Subject:** `git diff 3887f2a..9878aab`, branch `integrate/open-prs`, PR #208 (PRs #206, #207, #189, #188 and their fixes)
- **Evidence:** base `3887f2a`, target `9878aab`; registry sources for rustix 1.1.5 and 1.1.6, cap-fs-ext 4.0.2 and 4.0.3

⚠️ ALL VERDICTS HEREIN ARE VALID ONLY FOR THE COMMITS ABOVE.

## Executive Deduction

The re2js acceptance rested on "repository-authored Markdown", which CI's `pull_request` trigger contradicts, while the real defence (the opt-in `LOOKBEHINDS` flag markdownlint never sets) was never stated. No detective found a behaviour break, a weakened control or a stale live pin. The defects are all in written reasoning and are repaired in `57bb347`.

## Report Card

| Measure | Grade | Basis |
| --- | --- | --- |
| Fidelity (HOLMES) | B | 36 claims, 6 drifts, every pin exact |
| Clarity (WATSON) | C+ | Toolchain paragraph overloaded; three framing faults |
| Resilience (MORIARTY) | B+ | 6 strikes landed, none behavioural |
| **Overall** | **B** | No fatal strike; false prose in an admission record |

## Adjudication

- **statx:** all three detectives converge. The sentence was false and the conclusion true, because the padding is `repr(transparent)` and unread. Repaired.
- **re2js:** Holmes called the risk understated and Moriarty called it overstated. Both hold: the severity was omitted and the reachability overstated. The front office independently reproduced that `re2js` 2.8.6 refuses `(?<=a)b` without the flag and compiles it with the flag, and found no `LOOKBEHINDS` reference in either package. Acceptance stands on the corrected reasoning.
- **libc minimum, licence, Rust floor:** Holmes and Moriarty converge. Repaired.
- **Braces advisory:** withdrawn 2026-10-23, confirmed through the GitHub API. Issue #176 stays open for the maintainer to decide.
- **cargo-binstall route for `b3sum`:** Moriarty alone, resting on a README. Not established, so it is not recorded in the repository.
- **CONTESTED:** none.

## The Repair Menu

Applied in `57bb347`: R2, R3, R4, R5, R6, R8, R9, R10, R12 (partly). R1 and R7 are handled on PR #208.

Deferred, for the maintainer: R11 (the Rustix API list, which needs its own reading of line 18), R13 (close or re-scope #176; optionally open a re2js issue), R14 (confirm the binstall route from an install log), R15 (the workflow contract covers only the documentation job; pre-existing), R16 (compile checks on `aarch64-apple-darwin` and `aarch64-unknown-linux-gnu`).

## Ship Verdict

**REQUIRES INVESTIGATION**: sound after the named repairs, which are applied. Merge requires green CI on the repaired head.

## Chain-of-custody notes

- Graft MCP was unavailable, so detectives used plain reads.
- Testimony concerns `9878aab`. The repair commit `57bb347` touches only `CHANGELOG.md` and two dependency pages and was not re-testified.
- The front office added "and one compile check" to Watson's R5 wording, after reproducing Moriarty's experiment, and "production code" to R4.
- Moriarty and Watson cited CI as still running; all four gates later passed on `9878aab` (run 41660329996).
- The case is filed on local branch `cases/open-prs-integration`, not on the integration branch, so that testimony files do not enter `main` or the documentation lint corpus.
- The front office's own earlier error: the first push missed the lock digest because the pin search looked only for version strings. CI caught it.
