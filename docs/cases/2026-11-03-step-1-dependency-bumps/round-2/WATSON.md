# Testimony of Dr. John H. Watson: blind comprehension deposition

**Target:** `integrate/open-prs` at `00a36e0`, against base `3887f2a`, excluding lockfile bytes.
**Read as:** a maintainer or auditor who was not present and is deciding later whether these versions are acceptable.

I read no prior testimony, nothing under `docs/cases`, and nothing on the case branch. I ran a few small checks only where a careful reader would stop and look. Anything I infer is marked *conjecture*.

## Overall impression

These pages are written by someone trying not to overclaim, and that deserves credit. Several sentences limit their own claims without being asked. However, the reader cannot find the one thing an admission needs most: whether the new versions were actually exercised on the platform they are admitted for.

## What serves the reader well

- **Dated advisory position.** The heading fixes the audit to a date, so nobody mistakes it for a current claim (`docs/dependencies/documentation-toolchain.md#25@00a36e0`).
- **Clear split of advisories.** They are sorted into removed, added, retained and no longer reported, and the top line admits this is not a clean audit (`documentation-toolchain.md#27-32@00a36e0`).
- **Limits stated outright:**
  - "a source review and one compile check, not a proof" (`documentation-toolchain.md#30@00a36e0`)
  - "a compile check only" (`docs/dependencies/cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#91@00a36e0`)
  - "does not assert universal equivalence" (`cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#93@00a36e0`)
- **Trusted-input excuse declined.** "does not rely on trusted input" is the rarer and better sentence, and the workflow's `pull_request` trigger backs it (`documentation-toolchain.md#30@00a36e0`, `.github/workflows/ci.yml#6@00a36e0`).
- **Changelog hedges match the page.** The entry says "appears unreachable", which is the right strength (`CHANGELOG.md#11@00a36e0`).
- **The two Rustix checks are well argued.** Each gives the change, Keep's exposure and why it is safe, and the reader can follow it (`cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#89@00a36e0`).
- **Bisection limit disclosed.** "red by design" is plainly stated (PR #208 body, Tests, `@00a36e0`).

## Stumbles, ranked by harm

### 1. I cannot tell whether the new graph ran on Linux

This is the central question.

- The section closes with "Current validation must exercise the admitted filesystem and process boundaries with the new graph" (`cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#93@00a36e0`). That is an obligation, not a record. There is no run, commit or result.
- The same wording appears in the older Rustix section (`cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#71@00a36e0`), so it reads as boilerplate.
- The PR's Tests section lists only the "xtask toolchain laws", the mutations, `npm audit` and markdownlint. It says nothing about the filesystem, writer-lock or recovery laws (PR #208 body, Tests, `@00a36e0`).
- The header still says "Status: Accepted" (`cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#3@00a36e0`).
- When I asked, `gh pr checks 208` showed all four CI jobs pending at this head. So acceptance appears in the text before any Linux result does. CI may well go green, but nothing in the record will connect that result to this admission.

### 2. The compile check says little, and the reader is not told why

- The only execution evidence on the Rust side is `cargo check` on `aarch64-apple-darwin` (`cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#91@00a36e0`).
- The admission's filesystem profile is Linux ext4 (`cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#18@00a36e0`), and the notable Rustix changes are placed at the "Linux boundary", in the `linux_raw` backend (`cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#89@00a36e0`).
- *Conjecture:* a Darwin build does not compile `linux_raw` at all. If so, the check says nothing about the changes this section worries over.
- The caveat "`aarch64-unknown-linux-gnu` was not checked" is honest but puzzling. CI runs `ubuntu-latest` (`.github/workflows/ci.yml#18@00a36e0`), which I take to be x86-64 (*conjecture*). The page never mentions that target, so I cannot tell whether Linux as a whole was unchecked or only one architecture.

### 3. The boundary between reviewed and unreviewed in the Rustix diff is blurred

- "A search of Keep and the locked capability crates found no behavioural reach into them" (`cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#91@00a36e0`). The next sentence says the list is not complete.
- So does "them" mean the eight named areas or every changed area? I cannot tell whether the whole comparison was reviewed.
- "Capability crates" is never defined. The lockfile shows Rustix is also used by `fs-set-times` and `rustix-linux-procfs` (`Cargo.lock@00a36e0`, consumer rows, which I queried only for this name). The reader cannot tell whether the search covered them, even though they sit in the page's own list of locked packages (`cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#40@00a36e0`, `#49@00a36e0`).
- No search method or terms are recorded, so the finding cannot be reproduced later.

### 4. "Published" source, but the only link is a GitHub tag comparison

- The review is described as "of the published source differences" (`cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#93@00a36e0`).
- The one link goes to GitHub `compare/v1.1.5...v1.1.6` (`cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#91@00a36e0`).
- A supply-chain auditor needs to know whether the bytes reviewed were the registry packages Keep actually locks or the upstream git tags. The page does not say.
- The cap-fs-ext claim of "changes only" has no link at all (`cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#89@00a36e0`).

### 5. The PR is more confident than the admission page

- PR body: "Recovery implications: None." Also "No identity, format or durability code changes" and "No Keep source changes" (PR #208 body, `@00a36e0`).
- Admission page: these packages are "present in Keep's published library graph and production filesystem behavior" (`cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#18@00a36e0`), and it declines to claim equivalence (`#93@00a36e0`).
- "No code changes" is literally true of Keep's own recovery code, but a reader of "None" hears "assessed and clear". A dependency swap under the writer lock and ext4 admission has not been shown to have no recovery implications. It has only not been tested in anything the record shows.
- "No Keep source changes" also sits beside changed `xtask/src/documentation_integrity/*.rs` files (`xtask/src/documentation_integrity/node_toolchain.rs@00a36e0`). I took it to mean library source, but the PR does not say so.

### 6. The case for accepting a high advisory is only half made

- The rejected alternative is that leaving #188 out was refused because "it removes more than it adds" (PR #208 body, Alternatives rejected, `@00a36e0`).
- The page rates what was added (high) and what was retained (low), but gives no severity for the four removed advisories and no statement of whether they were reachable (`documentation-toolchain.md#29@00a36e0`).
- So the reader is offered a count, with four unrated advisories on one side and one high on the other, and cannot weigh it.
- The word "acceptance" appears in passing (`documentation-toolchain.md#30@00a36e0`), but no decision is stated and no reason is given for not staying on 0.23.3.
- The page's own "Alternatives rejected" section does not mention this choice (`documentation-toolchain.md#117-125@00a36e0`). Once the PR is closed, the rationale is effectively gone from the document a later reader will consult.
- `katex` is retained with neither a reachability note nor a tracking issue (`documentation-toolchain.md#31@00a36e0`).

### 7. The fallback argument covers only CI

- If the review is wrong, the limits "limit a CI run to a stalled job" (`documentation-toolchain.md#30@00a36e0`, confirmed at `.github/workflows/ci.yml#9,93,98@00a36e0`).
- The page says these tools also run "in contributor environments" (`documentation-toolchain.md#13@00a36e0`).
- A maintainer who lints a hostile PR locally is not covered by this argument, and that gap is not acknowledged. The practical harm is probably small, but the PR's unqualified "bound the effect to a stalled job" overstates it (PR #208 body, Failure modes, `@00a36e0`).

### 8. The review citation will not outlive the branch

- "Baker Street: overall B... Full testimony is on local branch `cases/open-prs-integration`" (PR #208 body, Review, `@00a36e0`).
- The branch exists locally, but `git ls-remote origin 'cases/*'` returns nothing. An auditor elsewhere cannot get it.
- "Baker Street" is never explained, and the reader cannot tell who performed the review or how independent it was.
- The grade is tied to `57bb347`, with `00a36e0` landing after it. That later commit made further prose changes (its commit message, `00a36e0`), so the review language suggests more coverage of this head than it has.
- "no weakened control" sits awkwardly beside a newly added high advisory and an open #210.

### 9. The b3sum install path is understated

- Changelog: "moves the cargo-binstall version... #210 tracks admitting that install path" (`CHANGELOG.md#15@00a36e0`).
- PR: "the CI tool installs the workflow contract doesn't cover" (PR #208 body, Security implications, `@00a36e0`).
- Only by opening issue #210 did I learn that the independent vector oracle binary comes through `cargo-binstall@latest` from the third-party source QuickInstall, with no admitted provenance. The issue fairly says this predates #206.
- No Markdown page other than the changelog mentions `install-action` (`git grep`, `@00a36e0`). So #206 has no admission record, and the PR's "renew both admission records" leaves that unnoticed. The fact is disclosed, but the alarming part sits one click away.

### 10. Corrections to earlier records are made silently

- The Rustix license changed from `Apache-2.0 OR MIT` to the three-way form (`cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#55@00a36e0`).
- Since "license" is a named reopen trigger (`cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#65@00a36e0`), a diff reader may think upstream relicensed. It did not: the local registry copies of 1.1.5 and 1.1.6 declare the same license, so the old text was simply wrong. The explanation exists only in the commit message (`57bb347`).
- `statx` was likewise added to the API list (`cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#18@00a36e0`) although Keep already called it at base (`src/adapters/filesystem_platform_profile.rs#154@3887f2a`). The page does not mark this as a correction either, and an audit record ought to.

### 11. Structure of the toolchain section

- The most consequential decision on the page sits in one bullet of about ten sentences (`documentation-toolchain.md#30@00a36e0`).
- The new dated H3 also captures the following, undated installer, digest and refusal material, which is not an advisory position at all (`documentation-toolchain.md#34-50@00a36e0`). A reader using the outline could treat those facts as dated to that audit or miss them.

### 12. Unexplained phrases

- **"one compile check"** (`documentation-toolchain.md#30@00a36e0`) appears to mean a regex-pattern compile trial, while the other page uses "compile check" for `cargo check` (`cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#91@00a36e0`). The first has no pattern, command or record.
- **"sets no front-matter pattern"** arrives with no explanation of why front matter matters to this advisory, so I cannot tell which of the three premises carry the argument (`documentation-toolchain.md#30@00a36e0`).
- **"Each mutation still hits its target"** means little to anyone who was not there (PR #208 body, Tests, `@00a36e0`).

### 13. Older sections read as current

- "The direct cap-fs-ext pin remains 4.0.2" and "the Rustix 1.1.5 admission are unchanged" are in the present tense, in a page titled for 4.0.3 and 1.1.6 (`cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#75@00a36e0`).
- The chronological order rescues a patient reader, and the header dates help (`cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#5-7@00a36e0`). Those sections carry no dates themselves, so a skimmer could still be misled.
- This one is minor.

## Where I was misled

There were no outright falsehoods that I could detect. I did not audit the upstream claims, which is another investigator's work. I was misled in three places:

- **"both pass"** (`cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#89@00a36e0`) made me picture a test run. Only four lines later did I learn these were source-reading checks.
- **"Recovery implications: None"** made me think the question had been examined.
- **"Baker Street... no weakened control"** made me think the current head had been reviewed.

## Grade

**C+.** The candour and much of the structure earn more. But an admission exists to let a later reader answer "was this exercised, and on what?", and that answer is absent from the page and from the PR. The advisory acceptance is honest about its limits but does not give the reader the means to weigh it. A later auditor could tell what was asserted but not always what was done.

```text
VERDICT
witness: WATSON
target: integrate/open-prs @ 00a36e0 (base 3887f2a, lockfile bytes excluded)
clarity_grade: C+
fact_vs_assumption: mostly distinguishable; hedges are honest, PR body less careful than the pages
reviewed_vs_unreviewed: NOT reliably distinguishable (scope of Rustix search; source of "published" bytes; Linux execution)
misleading_statements: 3 by impression ("both pass"; "Recovery implications: None"; review claim); no falsehood detected
stumbles_ranked:
  1. No record that the new graph ran on Linux; "must exercise" is an obligation; status already Accepted; CI pending at this head
  2. Only Rust execution evidence is a Darwin cargo check; relevance to linux_raw unstated; Linux target caveat puzzling
  3. Rustix search scope ambiguous; "capability crates" undefined; method unrecorded
  4. "Published" review links a git tag comparison; cap-fs-ext claim unlinked
  5. PR "Recovery: None" and "no code changes" overconfident versus the admission page
  6. High-advisory acceptance: removed advisories unrated, no stated decision, not in Alternatives rejected; katex without disposition
  7. Stalled-job fallback covers CI only, not contributor environments
  8. Review citation points to an unpushed branch; tool unexplained; grade predates head
  9. b3sum third-party provenance visible only by opening #210; #206 has no admission record
  10. License and statx corrections unmarked on the page
  11. Decision buried in a long bullet; dated H3 swallows undated installer material
  12. Unexplained: "one compile check", "front-matter pattern", "each mutation"
  13. Older sections' present tense clashes with the page title (minor)
evidence_limits: upstream claims and lockfile bytes not verified; CI status observed once, pending; x86-64 runner and linux_raw not compiled on Darwin are conjecture
repairs_written: none, per charter
```
