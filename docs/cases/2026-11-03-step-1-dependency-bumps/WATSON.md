# Watson's first deposition: comprehension pass, base 3887f2a, target 9878aab

I read this as a maintainer arriving months later and asking one question: may we keep these versions? The record has honest instincts, but in several places a plain reading misleads. The worst of these sit in the sentences that carry the acceptance argument.

**Basis of testimony.** I read the commit messages, the non-lockfile diff and the three focus documents. I also opened what that reader would open: the linked advisories and issue, the Keep call sites, the override check and the lockfile's version lines. The upstream Rustix and cap-fs-ext source differences I did not read, so I cannot say whether their descriptions are true. That is Holmes's beat. Advisory and issue states come from `gh` on 2026-11-03.

## What serves the reader

- **Scope limits are stated.** "This review is of the published source differences. It does not assert universal equivalence" says what the review is and isn't (`docs/dependencies/cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#91@9878aab`).
- **The audit caveat is frank.** "This development-tool update does not claim a clean npm audit" (`docs/dependencies/documentation-toolchain.md#23@9878aab`).
- **The accepted cost is admitted.** The paragraph says plainly that the re2js exposure remains until upstream moves (`docs/dependencies/documentation-toolchain.md#23@9878aab`).
- **The header and date-stamped section structure work.** I could find the new review at once (`docs/dependencies/cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#5@9878aab`, `#85@9878aab`).
- **One spot check held.** The sole `StatVfsMountFlags` constant Keep tests in production code is `RDONLY` (`src/adapters/filesystem_platform_profile.rs#291@9878aab`).

## Where I was misled

**1. The `statx` sentence.** The section says the reviewed changes to "`statx` padding fields" and others "are outside the operations Keep invokes" (`docs/dependencies/cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#89@9878aab`). Yet Keep calls `statx` directly and takes its device and mount identity from the result (`src/adapters/filesystem_platform_profile.rs#154@9878aab`, `#226@9878aab`).
- The operation is one Keep invokes, so the stated reason does not fit.
- *Conjecture:* the intended point is that Keep does not read the padding fields. That would be a different claim from the one written, and the page does not make it.
- The sentence that clears the change is therefore the one I trust least. It concerns code the identity checks rely on.

**2. The re2js severity is left out.** The paragraph calls `katex` and `micromark-extension-math` "low-severity" but gives re2js no rating, only "quadratic-time denial of service" (`docs/dependencies/documentation-toolchain.md#23@9878aab`). Clicking the link shows GitHub rates it **high** (GHSA-gg6h-894w-4m9c, via `gh`). Naming the low rating and omitting the high one makes the paragraph read calmer than its source.

**3. The braces advisory has been withdrawn, and the page doesn't say so.** It says the advisory "remains tracked… although `npm audit` did not report it" (`docs/dependencies/documentation-toolchain.md#23@9878aab`).
- The linked advisory is titled "Withdrawn Advisory", with `withdrawn_at` 2026-10-23 (`gh`). That date is before this paragraph's own 2026-11-03 audit date.
- So the "although" presents a mystery the first click explains.
- The earlier "no patched version" fact was dropped without comment (`docs/dependencies/documentation-toolchain.md#23@3887f2a`).
- Issue #176 is still open, titled "Resolve remaining braces advisory" (`gh`).

**4. The "repository-authored Markdown" mitigation.** The re2js exposure is excused because the tool "only reads repository-authored Markdown" (`docs/dependencies/documentation-toolchain.md#23@9878aab`). But the job runs on every `pull_request` (`.github/workflows/ci.yml#6@9878aab`, `#90@9878aab`), and the page itself calls this pull-request CI (`docs/dependencies/documentation-toolchain.md#112@9878aab`).
- A pull request's Markdown is not obviously "repository-authored". *Conjecture:* outside contributors could be supplying it.
- "This change does not add an input" is literally narrow, and it reads wider than it is.
- The page never says whether Markdown linting reaches lookbehind matching at all. It also doesn't connect the bounded runner to this risk, though it mentions that runner two sentences later.

## What the reader cannot tell

**5. Whether validation passed.** "Current validation must exercise…" (`docs/dependencies/cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#91@9878aab`) states an obligation, not a result. Nothing in the range records a passing run. The only validation outcomes the commit messages report are refusals (`git log 430d12b`, `git log 99722f2`), and 9878aab repairs a digest the previous fix missed. A reader cannot separate "was run and passed" from "ought to be run".

**6. Who reviewed what, and how.** "This review" names no reviewer, source artifact, comparison method or link to the difference (`docs/dependencies/cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#89-91@9878aab`). Also, "one place that needs review" reads like an open to-do, when the next sentence treats it as done. Whether "the other reviewed changes" is a complete list or a selection is not stated.

**7. What "supported targets" means.** "Apple `fallocate`" and "32-bit" are excluded by reference to "supported targets", which the section does not define (`docs/dependencies/cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#89@9878aab`). The same page speaks of "every supported platform" and carries Windows resolution notes (`#63@9878aab`, `#51@9878aab`). The phrase "operations Keep invokes" also leaves open whether Rustix calls made by cap-primitives were considered. I raise that as a reader's question, not a finding.

**8. The re2js ownership gap.** Braces has follow-up #176, but re2js, the actual open exposure, has no issue, owner or stated acceptance (`docs/dependencies/documentation-toolchain.md#23@9878aab`). "Remains until `markdownlint` moves its pin" names no one who will notice.

**9. The MSRV gap and the stale floor.** The new section is silent on Rustix 1.1.6's Rust floor. The standing text still says "Rustix 1.1.5 declares Rust 1.65" and was left unedited (`docs/dependencies/cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#55@9878aab`), under a title now reading 1.1.6 (`#1@9878aab`). A sibling changelog entry does state MSRV as unchanged (`CHANGELOG.md#17@9878aab`), which makes this silence noticeable. The new `bitflags` and `libc` minimums are also not given, only that the locked versions "already satisfy them" (`#87@9878aab`).

**10. The unidentified advisories.**
- The removed `markdown-it` and `smol-toml` advisories carry no IDs (`docs/dependencies/documentation-toolchain.md#23@9878aab`).
- The older changelog entry says 0.23.3 already addressed advisories in those same packages (`CHANGELOG.md#31@9878aab`). I had to infer these are newer reports.
- "Their earlier low-severity reports" points to nothing. `katex` and `micromark` appear in no documentation or changelog text at base (`git grep` at 3887f2a), and they carry no IDs either.
- The audit result for the new graph is only implied. The one explicit date attaches to the previous graph.

**11. The unlinked override policy.** "Repository policy forbids npm overrides" gives no pointer (`docs/dependencies/documentation-toolchain.md#23@9878aab`).
- The rule is real and enforced in code (`xtask/src/documentation_integrity/node_toolchain.rs#48@9878aab`).
- The page's own review triggers list "npm dependency override" as if it could happen (`docs/dependencies/documentation-toolchain.md#138@9878aab`).
- The commit message softens it to "does not override" (`git log 430d12b`).

## Structure, tone and history

**12. The long paragraph.** One unwrapped line holds pins, removed advisories, a live exposure, the mitigation, two more dependency reports, an audit disclaimer and an unrelated closing sentence about `--no-globs` (`docs/dependencies/documentation-toolchain.md#23@9878aab`). The open risk sits inside a section called "Admission boundary". Its neighbours are hard-wrapped (`#19-22@9878aab`). I read it three times.

**13. Changelog wording.**
- "The new re2js advisory that its exact markdownlint pin retains" puts "new" against "retains", and "its" has an unclear referent (`CHANGELOG.md#11@9878aab`).
- "Removing… advisories" is stated flatly, without the single-audit-date limit the toolchain page carries.
- "The CI job's pinned" action (`CHANGELOG.md#15@9878aab`) is singular, but the diff moves it in two jobs, `quality` and `dependency-policy` (`.github/workflows/ci.yml#35@9878aab`, `#180@9878aab`). One of those installs the policy tools themselves.
- The #206 entry and range describe no review of that action. An auditor cannot tell whether one occurred.
- The #207/#189 entry says only that features are unchanged, and is silent on the raised minimums (`CHANGELOG.md#13@9878aab`).

**14. Header drift.** "Capability admission updated: 2026-10-03 (#93)" now sits below a later cap-fs-ext update (`docs/dependencies/cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#5-7@9878aab`). cap-fs-ext is itself one of the page's "capability packages" (`#55@9878aab`), so the line no longer names the latest capability change. It is minor, but I did pause on it.

**15. Commit traceability.** The Rustix and cap-fs-ext admission renewal lives in a commit whose subject concerns markdownlint and #188, mentioned only as "Also renew…" (`git log 430d12b`). Anyone running `git log` on the admission page lands on a misleading headline. Its body explains why the page must reopen, not what was reviewed.

The messages also say the bumps were refused by checks until later fixes (`git log 99722f2`, `git log 430d12b`), after their merge commits in this range. Whether those intermediate states reached `main`, I cannot tell from this record.

**16. Minor terms.** "Repository-task manifest" never names `xtask/Cargo.toml` (`docs/dependencies/cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#87@9878aab`). "Remaining resolved package set" made me stop to work out "remaining after what". "Both locked workspaces" goes undefined in the changelog (`CHANGELOG.md#13@9878aab`).

## Stumble list, by harm to the reader

| # | Location | Stumble | Harm |
|---|---|---|---|
| 1 | cap-std page #89 | `statx` called outside Keep's operations, though Keep calls it | Misled |
| 2 | toolchain #23 | High severity omitted, low severity named | Misled |
| 3 | toolchain #23 | Advisory withdrawal not disclosed | Misled |
| 4 | toolchain #23 | "Repository-authored" sits badly with `pull_request` CI; reachability unstated | Misled |
| 5 | cap-std page #91 | Validation is an obligation, with no recorded result | Cannot decide |
| 6 | cap-std page #89–91 | Review has no reviewer, method or scope list; "needs review" is ambiguous | Cannot decide |
| 7 | cap-std page #89 | "Supported targets" undefined | Cannot decide |
| 8 | toolchain #23 | re2js has no tracker or owner | Cannot decide |
| 9 | cap-std page #55, #87 | 1.1.6 Rust floor absent; stale 1.1.5 text; new minimums absent | Cannot decide |
| 10 | toolchain #23 | Advisories without IDs; dangling "earlier" | Re-read |
| 11 | toolchain #23, #138 | Override policy unlinked; forbidden yet listed as a trigger | Re-read |
| 12 | toolchain #23 | One overloaded paragraph, risk buried | Re-read |
| 13 | CHANGELOG #11, #13, #15 | Muddled re2js wording, flat "removing", singular "job" | Re-read |
| 14 | cap-std page #5–7 | "Capability" header line outdated | Paused |
| 15 | `git log 430d12b` | Admission renewal hidden under a markdownlint subject | Paused |
| 16 | cap-std page #87 | Unmapped terms | Paused |

## VERDICT

```
WITNESS: WATSON (blind comprehension pass)
TARGET: git diff 3887f2a..9878aab, excluding lockfile bytes, plus commit messages
AUDIENCE: later Keep maintainer or auditor deciding version acceptability
CLARITY GRADE: C+
  cap-std new section: B-  (well-scoped, but #1, #5, #6, #7, #9)
  toolchain paragraph: D+  (overloaded; #2, #3, #4, #8)
  CHANGELOG entries:   B-  (brief; #13)
  commit messages:     C   (explain the mechanics; #15; no validation result)
FACT vs ASSUMPTION: partly separable. Scope disclaimers are good; validation
  status is not determinable from this record.
REVIEWED vs NOT REVIEWED: partly separable. The diff is named as the basis, but
  there is no reviewer, method, completeness statement or difference link.
HONESTY OF FRAMING: good intent; three framing faults reachable by one click
  or one grep (#1, #2, #3), plus one narrow mitigation (#4).
SAFE FOR THE READER TO DECIDE FROM AS WRITTEN: NO
HIGHEST-PRIORITY STUMBLES: #1, #2, #3, #4, #5
NOT ASSESSED: truth of upstream source-difference claims (no upstream source
  read); Cargo.lock and package-lock.json bytes; whether CI ran or passed
CONJECTURE LABELED: intended meaning of the statx sentence (#1); outside
  contributors supplying Markdown (#4); transitive Rustix use is posed only
  as an open question (#7)
EDITS MADE: none. Repairs withheld per charter.
```

## Second Deposition — The Repair Menu

**Plain finding:** nothing here is fatal to the versions. Holmes and Moriarty both confirm the pins, and neither finds a behaviour break or a control downgrade (`HOLMES.md#127-129`, `MORIARTY.md#137-138`).

Two written arguments, however, are dead, and I would not tidy their wording:
- **"Repository-authored Markdown"** cannot justify accepting the re2js advisory, because CI lints pull-request Markdown (`.github/workflows/ci.yml#6@9878aab`). It needs replacing, not softening.
- **"Outside the operations Keep invokes"** is false for `statx` (`src/adapters/filesystem_platform_profile.rs#154@9878aab`). The conclusion survives only on a different reason.

**Fresh evidence, gathered 2026-11-03 [HEARSAY: unpinned]:**
- **CI run `41660329996` on 9878aab:** `Runtime fuzz smoke` has now passed, which updates Moriarty's evidence limit. `Rust quality gates` is still in progress, and the run has no conclusion.
- **re2js advisory:** its text confirms the flaw requires "the opt-in `LOOKBEHINDS` flag".
- **Scratch install (markdownlint 0.42.0, re2js 2.8.6):** the string `LOOKBEHINDS` appears nowhere in the `markdownlint` or `markdownlint-cli2` packages, and my search of cli2's top-level `.mjs` files found no re2js import.
- **`.markdownlint-cli2.yaml` in the worktree at 9878aab:** it sets no `frontMatter`.
- **Not reproduced by me:** Moriarty's runtime test showing lookbehind patterns throw (`MORIARTY.md#38`).
- **Saved audits** (`/private/tmp/claude-501/-Users-j-git-keep/0c5d0f19-e5f7-4a34-aac7-bdc48cc6cf8a/scratchpad/old.audit.json`, `/private/tmp/claude-501/-Users-j-git-keep/0c5d0f19-e5f7-4a34-aac7-bdc48cc6cf8a/scratchpad/new.audit.json`): these supply the advisory IDs below.

### Must-fix before merge

**R1. GATE: do not call this validated yet.** Effort: waiting time plus 2 min.
- **Covers:** Watson #5, Holmes claim 13, `MORIARTY.md#143`.
- **Edit:** none to the page. A commit cannot carry its own CI result, so "must exercise" (`docs/dependencies/cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#91@9878aab`) may stand.
- **Do:** put the green run URL for the final head in PR #208's body. The prose repairs below create a new head, so the result on 9878aab will not carry over.
- **Depends on:** a passing `Rust quality gates` run, which is not yet observed.

**R2. FALSE CLAIM: the `libc` minimum.** Effort: 1 min.
- **Covers:** Holmes D3, Moriarty S2, Watson #9.
- **Evidence:** registry `Cargo.toml` for rustix 1.1.5 and 1.1.6 (`HOLMES.md#26`). I re-read the 1.1.6 values.
- **Location:** `docs/dependencies/cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#87@9878aab`.

Replace "Rustix's declared minimums for its `bitflags` and `libc` dependencies rise, and the locked `bitflags` 2.13.1 and `libc` 0.2.186 already satisfy them." with:

```markdown
Rustix raises its minimum `bitflags` requirement from 2.4.0 to 2.9.2, which the locked `bitflags` 2.13.1 satisfies. Its runtime `libc` requirement stays at 0.2.182, which the locked `libc` 0.2.186 satisfies; only its development dependency on `libc` moves.
```

**R3. FALSE CLAIM and STALE: licence and Rust floor.** Effort: 2 min.
- **Covers:** Holmes D4 and reverse drift 3, Moriarty S5, Watson #9.
- **Evidence:** both releases declare the licence below and Rust 1.65 (`Cargo.toml#14,#52@rustix-1.1.6`, `Cargo.toml#14,#52@rustix-1.1.5`), which I confirmed directly. The licence error predates this diff, but the page has just been renewed and names licence as a reopening concern (`#65@9878aab`).
- **Location:** `docs/dependencies/cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#55@9878aab`.

Replace "Rustix declares `Apache-2.0 OR MIT`." with:

```markdown
Rustix declares `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT`.
```

Replace "Rustix 1.1.5 declares Rust 1.65, below Keep's pinned Rust 1.96.0; its previous 1.1.4 release declared 1.63." with:

```markdown
Rustix 1.1.6 declares Rust 1.65, the same floor as 1.1.5 and below Keep's pinned Rust 1.96.0.
```

**R4. FALSE CLAIM and UNDEFINED SCOPE: the Rustix review paragraph.** Effort: 15 min.
- **Covers:** Holmes D2 and reverse drift 4, Moriarty S1 and S4, Watson #1, #6 and #7.
- **Location:** `docs/dependencies/cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#89@9878aab`. Replace the whole paragraph with:

```markdown
Published cap-fs-ext 4.0.3 changes only its cap-primitives and cap-std requirements and one import order, apart from version and packaging metadata. Published Rustix 1.1.6 needs two checks at Keep's Linux boundary, and both pass. First, its `linux_raw` backend replaces `StatVfsMountFlags` constants derived from `MS_*` values with the kernel's `ST_*` values. Keep tests only `RDONLY`, whose value remains `0x0001`. Second, it changes padding fields in its `Statx`, `StatFs` and `Stat` layouts to `Padding`, a `#[repr(transparent)]` wrapper. Keep does call `statx`, `fstatfs` and `fstatvfs`, but it reads no padding field, and neither Keep nor the locked capability crates name those fields.

The [Rustix comparison](https://github.com/bytecodealliance/rustix/compare/v1.1.5...v1.1.6) touches other areas too, including extended attributes, `recvmsg` truncation, termios, `inotify`, `tcgetpgrp`, 32-bit `chown` and `getgroups` selection, Apple `fallocate` and socket options. A search of Keep and the locked capability crates found no behavioural reach into them. This paragraph summarises the comparison and does not list every changed file.
```

Facts this wording depends on:
- **Padding and field-use claims:** these rest on Holmes and Moriarty's source reading (`HOLMES.md#22`, `MORIARTY.md#17-18`). I saw the `Padding` hunks but did not repeat the searches.
- **The compare link:** this is a git tag range, while the review was of published source. I did not check that tag and published crate match. If that matters, cite the `Cargo.lock` checksums as well.
- **"Keep tests only `RDONLY`":** this holds for production code. Tests also use `empty()` (`src/adapters/filesystem_platform_profile_tests.rs#14@9878aab`).
- **Who reviewed:** I will not invent a name. If the maintainer wants one, they must supply it (Holmes claim 12 is unverifiable).

The phrase "on its supported targets" is removed on purpose. Moriarty found no definition of the set (`MORIARTY.md#51`), and the new wording doesn't need one.

**R5. UNSOUND ARGUMENT, OMISSION and STRUCTURE: the toolchain paragraph.** Effort: 20 min.
- **Covers:** Holmes D1 and reverse drift 1–2, Moriarty S3, Watson #2, #3, #4, #8, #10, #11 and #12.
- **Location:** replace `docs/dependencies/documentation-toolchain.md#23@9878aab` with:

```markdown
`markdownlint-cli2` 0.24.0 admits `js-yaml` 5.4.3, `smol-toml` 1.9.1 and `markdown-it` 15.0.3. The production documentation command supplies explicit Git-selected paths with `--no-globs`; its bounded runner reports tool failure rather than admitting failed validation.

### Advisory position on 2026-11-03

This development-tool update does not claim a clean npm audit. The lock graph has five `npm audit` reports from two advisories, which are `re2js` and `katex` with their dependents.

- **Removed by this update:** `markdown-it` ([GHSA-r7fv-28h4-cvq7](https://github.com/advisories/GHSA-r7fv-28h4-cvq7), [GHSA-vfgh-jcmw-g37q](https://github.com/advisories/GHSA-vfgh-jcmw-g37q), [GHSA-gfr2-rv44-r23c](https://github.com/advisories/GHSA-gfr2-rv44-r23c)) and `smol-toml` ([GHSA-r4xh-jqrq-34v2](https://github.com/advisories/GHSA-r4xh-jqrq-34v2)).
- **Added by this update, rated high:** `markdownlint` 0.42.0 requires exactly `re2js` 2.8.6, which [GHSA-gg6h-894w-4m9c](https://github.com/advisories/GHSA-gg6h-894w-4m9c) reports as quadratic-time denial of service; the patched range starts at 2.8.7. The advisory requires the opt-in `LOOKBEHINDS` flag. The published `markdownlint` 0.42.0 and `markdownlint-cli2` 0.24.0 packages do not set it, and `.markdownlint-cli2.yaml` sets no front-matter pattern, so we found no path to the affected matching. That is a source review, not a proof. This acceptance does not rely on trusted input, because the `documentation` job lints Markdown proposed in pull requests. If the review is wrong, the job's ten-minute timeout, read-only permissions and non-persisted checkout credentials limit a CI run to a stalled job. The [Node toolchain contract](../../xtask/src/documentation_integrity/node_toolchain.rs) refuses npm `overrides`, so `re2js` stays at 2.8.6 until `markdownlint` moves its pin. Any such update reopens this review under the triggers below, including this reachability check.
- **Retained, rated low:** `katex` ([GHSA-238p-pmpm-9mq7](https://github.com/advisories/GHSA-238p-pmpm-9mq7)), which `npm audit` also reports through `micromark-extension-math`.
- **No longer reported:** the graph retains `braces` 3.0.3, but GitHub withdrew [GHSA-vfj7-8cjw-p6xm](https://github.com/advisories/GHSA-vfj7-8cjw-p6xm) on 2026-10-23. Follow-up [#176](https://github.com/flyingrobots/keep/issues/176) remains open.
```

Facts this wording depends on:
- **Audit data:** IDs, ratings and the withdrawal date are registry snapshots from 2026-11-03, not pinned evidence. The `micromark-extension-math` rollup is my inference from the audit's structure, and Holmes's count of two distinct advisories supports it (`HOLMES.md#46`).
- **"Stalled job":** the limit claim is grounded in `HOLMES.md#16`, but I keep it hedged because Holmes marked the realistic harm as conjecture. It covers CI only. I did not establish a deadline for local contributor runs, so I claim none.
- **Link:** please confirm lychee accepts the relative path to a source file. I did not run it.
- **Re-check obligation:** the promise to redo the reachability check is a new commitment. It rests on the existing triggers (`documentation-toolchain.md#136-137@9878aab`). The maintainer should accept it knowingly.

**R6. MISLEADING SUMMARY: changelog entry for #188.** Effort: 3 min.
- **Covers:** Watson #13.
- **Location:** replace `CHANGELOG.md#11@9878aab` with:

```markdown
- Update the development-only Markdown validation graph and exact admission policy to markdownlint-cli2 0.24.0. On the 2026-11-03 audit this removes the markdown-it and smol-toml advisories and adds the high-severity re2js advisory GHSA-gg6h-894w-4m9c through markdownlint's exact pin; the toolchain admission records why the affected path appears unreachable (#188).
```

**R7. PROCESS: complete PR #208's body.** Effort: 20 min.
- **Covers:** `HOLMES.md#65`, Watson #6, the AGENTS.md Pull Requests section.
- **Do:** fill in the required sections before it leaves draft. Record the CI URL from R1, the audit totals, and which review claims rest on registry source. PR state is `isDraft: true` at head 9878aab (`gh`).
- **Also state:** only the final integration commit is a bisection boundary, because `430d12b` and the earlier merges are red (`HOLMES.md#66`). Do not rewrite history to hide that.
- **Repair commit subject:** name #207 and #189 as well as #188 so `git log` on the admission page leads somewhere true. This covers Watson #15 without touching 430d12b.

### May defer

| ID | Type | Repair | Effort | Covers |
|---|---|---|---|---|
| R8 | WORDING | Replace `CHANGELOG.md#15@9878aab` with: "Update the pinned taiki-e/install-action action to v2.88.0 in the Rust quality gates and Dependency policy jobs (#206)." | 1 min | Holmes D5, Watson #13 |
| R9 | OMISSION | Change the end of `CHANGELOG.md#13@9878aab` to "selected features and the Rust floor are unchanged (#207, #189)." | 1 min | Watson #13 |
| R10 | STALE LABEL | Replace `cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#7@9878aab` with "- Earlier cap-std admission update: 2026-10-03 (#93)". | 1 min | Watson #14 |
| R11 | OMISSION | At `cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#18@9878aab`, change the list to "`openat2`, `fstatfs`, `fstatvfs`, `statx`, and ext4 inode flag APIs". | 1 min | Holmes reverse drift 5 |
| R12 | TERMS | At `cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#87@9878aab`, change "repository-task" to "`xtask` repository-task" once, and "remaining resolved package set" to "the other resolved packages". | 2 min | Watson #16 |
| R13 | TRACKER, maintainer's decision | Close or re-scope #176, since its advisory is withdrawn. Optionally open a re2js issue, as none exists (`HOLMES.md#47`, and my own search). R5 stands without it. | 5 min | Holmes reverse drift 1–2, Watson #8 |
| R14 | OMISSION, unverified | Note that the #206 bump moves the cargo-binstall route used for `b3sum`. Moriarty alone reports this (`MORIARTY.md#65`) and it rests on a README's stated fallback. Confirm it from an install log before writing anything. | 15 min | Moriarty S6 |
| R15 | CONTROL GAP, separate PR | The workflow contract checks only `jobs.documentation` (`workflow_contract.rs#93@9878aab`). The gap predates this range, so it should not grow this PR. Worth its own issue, not this repair. | Issue 10 min; fix unestimated | Moriarty S6 |
| R16 | EXPERIMENT | Moriarty's `cargo check --locked --all-features` on `aarch64-apple-darwin` and `aarch64-unknown-linux-gnu` (`MORIARTY.md#74`). He predicts no failure, so this is low value unless those targets are claimed as supported. Follow the workstation's heavy-work lock rules. | 20–30 min | Moriarty conjecture |

### Deliberately no repair

- **Holmes D6, "Move every pin":** the message is immutable and 9878aab already corrects it.
- **`cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#75@9878aab`, "remains 4.0.2":** this is dated history under its own heading, and Moriarty also tolerates it (`MORIARTY.md#59`).
- **A Keep test pinning `RDONLY`'s numeric value** (`MORIARTY.md#82`): I would not do this. It would freeze an upstream constant without testing the behaviour that matters. If the gap troubles anyone, a test that a read-only mount is refused is the better idea, but that is a new crash-and-platform test and outside this PR.

### Effort and landing

- **Must-fix prose (R2–R6):** about 40 min, all in one commit.
- **PR work (R1, R7):** about 20 min, plus the wait for CI.
- **No code change is indicated.** Neither lock digest should move, as the repairs touch only docs and the changelog. Moriarty confirms the digest covers the lock bytes (`MORIARTY.md#96`) and the documentation says it also covers the installer (`documentation-toolchain.md#33-36@9878aab`).

```
VERDICT
WITNESS: WATSON (second deposition, repair menu)
TARGET: 3887f2a..9878aab
FATAL TO VERSIONS: none found
FATAL TO ARGUMENT: "repository-authored" (R5); "outside the operations Keep invokes" (R4)
MUST-FIX: R1 R2 R3 R4 R5 R6 R7
MAY-DEFER: R8 R9 R10 R11 R12 R13 R14 R15 R16
OPEN EVIDENCE: Rust quality gates still in progress at last look; a new head will need a fresh run
UNVERIFIED INPUTS: reviewer identity; tag-versus-published source match; lychee on the
  new relative link; Moriarty's runtime lookbehind test; fork-approval setting;
  local-run deadline; cargo-binstall route for b3sum
READY TO MERGE AS IS: NO
READY AFTER MUST-FIX: yes, if R1's CI is green on the new head
EDITS MADE: none
```
