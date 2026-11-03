# Testimony of Dr. John H. Watson: Blind Comprehension Deposition, PR #208 Dependency Record

I read this as the person it was written for: an auditor arriving months later, with a suspicion and no one to ask.

**Scope**
- Target confirmed: `git rev-parse HEAD` in `/Users/j/git/keep-integrate` gives `de515f351b3bd7b7c8d5ee909fa3ed4c520e9634`. PR #208 reports the same head, and base `3887f2a…`.
- I did not read `docs/cases`, the case branch or any scratchpad.
- PR body citations are by section, because the body has no line numbers.
- I made small checks where a sentence tempted me to believe something. This is not a full verification.
- All short SHAs below are `de515f3` unless stated.

## General impression

This is an unusually honest record, and I'd trust its author more for how it hedges.

It repeatedly separates what was read from what was run:
- "both hold on source reading" (`docs/dependencies/cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#93@de515f3`)
- "That is a compile check only… so it says nothing about the two checks above" (`…cap-fs-ext-4.0.3.md#97@de515f3`)
- "That is a source review and one trial, not a proof" (`docs/dependencies/documentation-toolchain.md#121@de515f3`)
- "no CI job re-runs `npm audit`" (`documentation-toolchain.md#110@de515f3`)
- "they were not reviewed one by one" (`documentation-toolchain.md#129@de515f3`)
- "No issue tracks it" (`documentation-toolchain.md#114@de515f3`)

The Corrections paragraph owns two earlier errors without fuss (`…cap-fs-ext-4.0.3.md#99@de515f3`). The re2js risk is told plainly, including who is left unprotected: "A contributor who runs the linter directly on hostile Markdown has neither limit" (`documentation-toolchain.md#123@de515f3`).

The CHANGELOG keeps the same register ("appears unreachable", "no Keep exposure found", `CHANGELOG.md#11@de515f3` and `#13@de515f3`). It keeps pace with the docs, and nothing is oversold from one document to the next.

The final section's bolded labels (Security, What was reviewed, What was run, Corrections) answer the auditor's questions in the order they arise. The sentence "with this graph" also holds up. Neither lockfile nor any manifest changed between `9878aab` and `de515f3` (`git diff --stat 9878aab de515f3` touches only CHANGELOG, two docs and one test file). The lock diffs against base change only the versions 4.0.2→4.0.3 and 1.1.5→1.1.6, which supports "the other resolved packages are unchanged" (`…cap-fs-ext-4.0.3.md#87@de515f3`).

A few things did still trip me, and only one actually misled me.

## Where I was misled

**The new law reads as if it had run, and the record shows no run of it.**

Line 93 says a law "now pins" the `RDONLY` value. Line 97, four lines on, says run 41660329996 at `9878aab` passed "including the filesystem and process laws" (`…cap-fs-ext-4.0.3.md#93@de515f3`, `#97@de515f3`). I took the pinning law to be among them.

It is not. The law arrived in `de515f3` itself, after that run (`git diff --stat 9878aab de515f3` lists `src/adapters/filesystem_platform_profile_tests.rs`). The macOS check can't cover it either. The test module is gated on Linux (`src/adapters/filesystem_platform_profile.rs#322@de515f3`), and `cargo check` executes nothing anyway.

The PR body partly rescues this with "The final head's run is the merge gate" (PR #208 body, Tests), but it never says the new law is outside the cited run. While I was reading, run 41665161676 at the head was still `in_progress` (`gh run view`). The durable page carries no such caveat.

The substance is probably harmless, since the merge gate should execute the law. But this page's whole selling point is telling you what was run on what, and here the proximity of two sentences gives the wrong answer.

## Where I was left without an answer

**#206 has no reviewed-or-unreviewed statement.** PR #208 merges four dependency PRs, but the Approach section describes review and "both admission records" only for the other three (PR #208 body, Approach). The Problem section says none of the four carried admission work.

For `taiki-e/install-action` v2.87.28→v2.88.0 (`.github/workflows/ci.yml#35@de515f3`, `#180@de515f3`) I can find:
- the CHANGELOG line (`CHANGELOG.md#15@de515f3`);
- no mention in `docs/` (`git grep install-action -- docs` is empty);
- nothing saying whether anyone read the upstream change.

This action installs the independent vector oracle and the dependency-policy tools. The third-party b3sum source is disclosed and tracked, which is to the record's credit. But the CHANGELOG line alone could suggest #206 introduced that source. Only issue #210 says "Both predate #206". An auditor asking "was this version accepted, and on what basis?" gets no answer.

**Who reviewed and who accepted is not stated.** The page says "Status: Accepted" (`…cap-fs-ext-4.0.3.md#3@de515f3`), "The review compared…" (`#91@de515f3`) and "A search found…" (`#89@de515f3`, `#95@de515f3`). The toolchain page says "Keep accepts" and "we found" (`documentation-toolchain.md#119@de515f3`, `#121@de515f3`).

No person, agent or decider is named. The searches record no terms, so they can't be replayed. The trial `RE2JS.compile` records no command or location, though its version can be inferred from context. The labels tell me what kind of evidence I have, but not whose it is or how to reproduce it.

**The cited run proved more than the page says.** The page asks for dependency-policy and advisory lanes (`…cap-fs-ext-4.0.3.md#55@de515f3`, `#57@de515f3`) but reports only "Rust quality gates" (`#97@de515f3`). The run had four jobs, all successful, including Dependency policy and Runtime fuzz smoke (`gh run view 41660329996`).

So the page is honest but undersells, and a later reader can't tell from the text whether `cargo deny` and RustSec passed on this graph. The only route is the link.

*Conjecture:* if Actions logs age out, the link may answer less later on. That would leave the page's main execution evidence thin.

## Lesser stumbles

- **The reason for rejecting 0.23.3 is odd.** "Retains… reports without a reachability review" (`documentation-toolchain.md#138-139@de515f3`; also `#119@de515f3`) treats a review nobody did as a property of the rejected option. It is disclosed, but the trade is really one reviewed high against two unreviewed highs and a medium. That is a sound instinct resting on an asymmetry, not on a finding. The PR's version ("removes more than it adds", PR #208 body, Alternatives) is a count argument. The two rationales differ slightly.
- **Features look contradictory within the page.** Line 32 says default features are disabled and Keep enables `fs` and `std` (plus `process` in xtask). Line 95 lists resolved features including `default`, `alloc`, `termios` and `time`. The page never explains the gap. *Conjecture:* feature unification through the dependencies. I stopped to work this out. The sentence is load-bearing, since "not compiled" rests on it. It also says "thread changes" though no thread change appears in the preceding list (`#95@de515f3`).
- **The Benchmark section doesn't follow.** "None measured. No production code under `src/` changes" (PR #208 body, Benchmark). Rustix and cap-fs-ext *are* production dependencies under the writer lock (`…cap-fs-ext-4.0.3.md#14@de515f3`), so an unchanged `src/` is no reason to expect unchanged performance. "None measured" is true, but the second sentence reads as reassurance. The Recovery section frames the same fact correctly.
- **The review summary is opaque to an outsider.** "Baker Street ran twice… (Holmes A, Watson C+, Moriarty B+)" (PR #208 body, Review) is undefined jargon with unexplained grades. Neither named round was at the final head, which you can only work out from the SHAs. *Conjecture:* if the `de515f3` law came from a review finding, then "findings were about the written record" slightly understates it. That commit adds a test gap closure (`git log -1 de515f3`). I am barred from the testimony, so I can't settle this.
- **The aarch64 gap has no stated weight.** "`aarch64-unknown-linux-gnu` has no CI lane and was not checked" is admirably frank (`#97@de515f3`). But the page never says which platforms are supported, despite saying "every supported platform" (`#63@de515f3`). Other evidence pages use Linux aarch64 in Docker (`docs/testing-evidence/catalog-platform-admission.md#29@de515f3`), so I can't size this gap. The macOS check also has no commit named, unlike the CI run.
- **The trail from the page to this PR is thin.** The header and CHANGELOG cite #207, #189, #188 and #206 (`#5@de515f3`, `CHANGELOG.md#11-15@de515f3`), which the PR body says carried no admission work. The PR holding the review narrative, #208, appears in neither (`git grep '#208' -- docs CHANGELOG.md` has only an unrelated fuzz-log hit). Git ancestry will find it, but the citations point the wrong way first.
- **The Admission boundary section lists packages without saying why.** It names `js-yaml`, `smol-toml` and `markdown-it` (`documentation-toolchain.md#23@de515f3`). The old "addressing advisories" clause is gone, so the three look arbitrary, and the one package with a live high advisory isn't among them. The Advisory section covers it two sections later, so little harm is done.
- **Small hedges.** "imports… directly" (`#114@de515f3`) leaves transitive loading of `katex` open. "Six reports" (`#110@de515f3`) can't be reconciled from the page, though advisory-by-advisory bullets make that harmless. "Latest as of 2026-11-03" for markdownlint appears only in issue #209, so the page doesn't say that no fixed release existed.

## Matters of taste, not harm

- The filename omits rustix although the page admits it.
- No document links to this page by filename, which was already the case at base (`git grep` empty at both).
- "Rust floor" and "MSRV" are both used in the changelog.
- Present tense survives under the historical heading at line 69.

## Grade: B+

The prose earns the plus. Fact, assumption and limit are labelled better than in most audit records I've read, and the tone never sells. It misses an A on the auditor's own criteria, for three reasons:
- the law appears covered by a run that predates it;
- one of the four dependency moves has no review status;
- the judgement has no named author and its searches can't be replayed.

The grade concerns clarity. Whether the claims are true is another witness's business.

```
VERDICT
witness: WATSON (blind)
target: /Users/j/git/keep-integrate @ de515f351b3bd7b7c8d5ee909fa3ed4c520e9634 vs 3887f2a (lockfile bytes excluded)
clarity_grade: B+
fact_vs_assumption: mostly clear; explicit source-reading vs run labelling
reviewed_vs_unreviewed: clear for #207, #189, #188; absent for #206
what_ran_on_what: clear except new law and macOS check commit
head_run_at_reading: 41665161676 in_progress; not observed green
stumbles_ranked_by_harm:
  1. Pinning law reads as covered by CI at 9878aab; it was added at de515f3, and no recorded run covers it (…cap-fs-ext-4.0.3.md#93,#97@de515f3) [misleads]
  2. #206 install-action bump has no review status or admission record, despite oracle/policy tool role (CHANGELOG.md#15@de515f3) [gap]
  3. Reviewer/decider unnamed; searches and trial not replayable (…cap-fs-ext-4.0.3.md#3,#89,#95@de515f3; documentation-toolchain.md#121@de515f3) [gap]
  4. Dependency-policy and fuzz-smoke results of cited run not stated in text (…cap-fs-ext-4.0.3.md#55,#97@de515f3) [gap, undersells]
  5. 0.23.3 rejection rests on absence of a review; PR and page rationales differ (documentation-toolchain.md#119,#138@de515f3) [weak reasoning, disclosed]
  6. Declared vs resolved Rustix features unexplained (…cap-fs-ext-4.0.3.md#32,#95@de515f3) [confusion]
  7. Benchmark section implies no impact from unchanged src/ (PR #208 body, Benchmark) [mild misdirection]
  8. Baker Street jargon, undefined grades, rounds predate head (PR #208 body, Review) [opacity]
  9. aarch64 gap unweighted, supported platforms unstated; macOS check has no SHA (…cap-fs-ext-4.0.3.md#63,#97@de515f3) [minor]
  10. #208 not cited from page or CHANGELOG (…cap-fs-ext-4.0.3.md#5@de515f3) [minor traceability]
  11. Admission-boundary package list unexplained, `directly` hedge, six-report count (documentation-toolchain.md#23,#110,#114@de515f3) [trivial]
taste_only: filename omits rustix; no inbound links (pre-existing); floor/MSRV wording; tense at line 69
conjecture_labelled: log retention; feature unification; origin of the de515f3 law
repairs_offered: none (per charter)
```
