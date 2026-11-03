# Testimony of MORIARTY — PR #208

Pinned target: `integrate/open-prs` @ `00a36e0` (full SHA `00a36e018f277b3e905b00d6a92b6fbcbc8bbd88`, which matches the PR head), base `3887f2a`. No worktree was touched and the status is clean. Scratch work is in `/private/tmp/claude-501/-Users-j-git-keep/0c5d0f19-e5f7-4a34-aac7-bdc48cc6cf8a/scratchpad/moriarty-208/`.

I came for a corpse, my dear fellow, and found a patient with several small rashes. It is tiresome: the dangerous claims are true, and what I could wound is the paperwork around them. I shall enjoy that all the same.

**Status caveat:** CI run 41663162408 on `00a36e0` was still running when I stopped. Documentation, Dependency policy and Fuzz smoke were green, and **Rust quality gates was in progress**. The last fully green run is 41660329996 on `9878aab`, and `git diff 9878aab..HEAD` touches only `CHANGELOG.md` and two docs under `docs/dependencies/`.

## Strikes landed

### S1 (LOW-MEDIUM, vectors 2 and 7): a security fix filed under miscellany

- rustix 1.1.6 contains upstream commit `8e6aaa6`, a "Merge commit from fork" whose message reads "`getxattr`... special-cases a buffer length of 0... GHSA-jh2m-fhpj-3vr7". This is an upstream security-labelled fix.
- The admission reduces it to the words "extended attributes" in a list of "other areas": `docs/dependencies/cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#91@00a36e0`. The page has no GHSA reference for rustix, and the PR's "Security implications" discusses only the npm advisories.
- **Exposure: none found.** `git grep xattr` over `*.rs` and `*.toml` is empty, and a grep of cap-primitives, cap-std, fs-set-times and rustix-linux-procfs found no use.
- The defect is therefore in the record. A storage project's dependency admission fails to say that the upgrade is, among other things, a security fix.
- The global advisory endpoint returns 404 for this ID, so its severity is unknown to me. I do not inflate it.

### S2 (LOW-MEDIUM, vector 7): a review verdict nobody else can check

- The PR body says: "Baker Street: overall B, no behaviour defect, no weakened control... Full testimony is on local branch `cases/open-prs-integration`."
- That branch exists locally, but `git ls-remote origin 'refs/heads/cases/*'` returns nothing.
- The body names no reviewed SHA, and two documentation commits followed the review it describes: `57bb347` and `00a36e0` (the latter says it resolves "the review's deferred findings").
- So a public PR cites a grade for an unstated commit, with its evidence unreachable by any other reviewer. That breaks the global requirement for durable traceability.
- By charter I did not read that branch and make no claim about its contents.

### S3 (LOW-MEDIUM, vector 7): one bisection boundary over three unrelated dependency families

- The body says: "Only the merge of this PR is a bisection boundary."
- That one boundary contains the production filesystem dependency (rustix and cap-fs-ext), a documentation linter that adds a high-severity advisory, and a CI action. A later rustix regression bisects to the same integration commit as the linter, and a first-parent revert takes all of it.
- The pieces look separable. #206 had no failing check of its own, the fuzz-lock fix `99722f2` serves only the Rust bumps, and `430d12b`/`9878aab` serve only #188. I found no prerequisite edge from #188 or #206 to the Rust bumps.
- This cuts against "one executable issue → one coherent PR" and AGENTS.md's "prefer small, single-purpose PRs". The user may have ruled this deliberately, since the body calls it "Step 1". If so, it is a recorded choice rather than a defect, but the cost falls on exactly the storage-relevant dependency.

### S4 (LOW, vectors 3 and 1): the `RDONLY` law cannot see the constant that changed

- This release replaces the derivation of `StatVfsMountFlags::RDONLY` with a hand-written literal (`rustix-1.1.6/src/backend/linux_raw/fs/types.rs`: `const RDONLY = 0x0001;`).
- Keep's admission tests that symbol at `src/adapters/filesystem_platform_profile.rs#291@00a36e0`. Its laws feed in the same symbol at `src/adapters/filesystem_platform_profile_tests.rs#22@00a36e0` and `#39@00a36e0`, so they would pass whatever value upstream chose.
- A search for `EROFS`, `Errno::ROFS`, `MS_RDONLY` and `mount -o` found no real read-only-mount law in `src`, `tests` or `xtask/src`.
- Note the contrast: Keep keeps the ext4 magic and casefold bit as local constants "visible at the exact decision boundary" (`filesystem_platform_profile.rs#284-288@00a36e0`), but borrows `RDONLY`.
- The claim "value unchanged" is **true**; I checked it. It rests only on someone having read it, and nothing pins it. The gap predates this PR; the upgrade merely exposes it.
- Proposed experiment: patch rustix locally with `RDONLY = 0x0002` and observe that the tests stay green. CONJECTURE until run; the charter forbids me running it. The fix is `assert_eq!(StatVfsMountFlags::RDONLY.bits(), 0x0001)`.

### S5 (LOW, vectors 5, 4 and 7): #210 is narrower than the PR says

- The body says "#210 tracks the CI tool installs the workflow contract doesn't cover". #210's scope is only `quality` and `dependency-policy`.
- Outside that scope and untracked: `cargo install cargo-fuzz` in fuzz-smoke (`.github/workflows/ci.yml#147@00a36e0`, version-pinned by policy and `--locked`) and `rustup toolchain install` (`ci.yml#143@00a36e0`).
- Also left outside #210's stated "action pins and tool inputs": unversioned `apt-get install strace` (`ci.yml#31@00a36e0`). It sits in `quality` but is not an action input.
- The inverse error is smaller. #210 says the install-action uses "are not checked". Its sentence refers to "this contract", which is accurate, but `every_fuzz_workflow_action_uses_an_immutable_commit` already enforces 40-hex pin shape across the workflow directory (`xtask/src/fuzz_campaign/workflow_tests.rs#19@00a36e0`, sources at `#179`). A floating tag is refused today, and only a different SHA would pass.

### S6 (LOW, vector 5): loose ends around closing #176 and the audit

- **Closed ahead of its record.** #176 closed at 15:21 with a comment saying the fact is "recorded... by #208", yet #208 is a draft. Main's page still says #176 "tracks its disposition" (`docs/dependencies/documentation-toolchain.md#23@3887f2a`). If #208 stalls, main points at a closed tracker.
- **Close reason.** `state_reason` is `completed`, although none of #176's acceptance work was done. The advisory evaporated, so `not_planned` would be the honest label. This is a nit.
- **No trigger behind "Reopen if it's reinstated".** I found no `npm audit` in `.github`, `scripts`, `xtask/src` or `CONTRIBUTING.md`. The advisory position is a dated snapshot that nobody re-reads automatically.
- **`katex` is orphaned.** The page reports it as retained (`documentation-toolchain.md#31@00a36e0`) with no reachability statement, and an issue search for `katex` returns nothing. re2js got a full argument and an issue; katex got a shrug.

### S7 (LOW, vectors 3 and 7): "no weakened control" did not cover the linter's rules

- markdownlint 0.42.0 changes 27 rule files against 0.41.1.
- At least one change narrows detection: MD011's ignore list gains `htmlFlow`, so reversed links inside HTML blocks are no longer reported (`lib/md011.mjs` diff). The repository's config disables only MD013, so MD011 is live.
- The PR tests "169 files pass", which shows only that nothing fails. I reproduced that number below.
- The narrowing is defensible upstream, and it is trivial for a documentation linter. But the toolchain admission records no rule-behaviour review.

### S8 (LOW, vector 6): stale present tense on the renewed page

- `docs/dependencies/cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#75@00a36e0` says "The direct cap-fs-ext pin remains 4.0.2... the Rustix 1.1.5 admission are unchanged".
- `#69@00a36e0` says the fuzz workspace "also selects 1.1.5".
- The page's own title says 4.0.3 and 1.1.6, and the body's stated invariant is that every "admission record matches what is locked". These sit under dated-by-version headings, so a careful reader will cope; a grep will not. Past tense or a "historical" label cures it.

### S9 (NIT, vectors 7 and 2): small body and prose gaps

- **"No Keep source changes."** `src/` and `tests/` are indeed untouched, but `xtask/src/documentation_integrity/` (`node_toolchain.rs`, `tool.rs` and their tests) is changed. The body's own Tests section says so.
- **Evidence the admission demands is not cited.** The page says "Current validation must exercise the admitted filesystem and process boundaries" (`#93`). The Tests section lists only xtask toolchain laws and never cites a Rust run on the new graph, though run 41660329996 did exercise it. This is a citation omission, not a lack of evidence.
- **macOS compile check.** It has no receipt (no commit, toolchain or log), and there is no `target/` directory in this worktree. The claim may be perfectly true, but it is uncheckable. It also cannot touch either named Linux check, because Keep's admission code is `#[cfg(target_os = "linux")]` (`filesystem_platform_profile.rs#10,41,279@00a36e0`) and macOS uses rustix's libc backend. The prose honestly says "compile check only", so I do not call it an overclaim. But it names `aarch64-unknown-linux-gnu` as unchecked and never mentions that x86_64 Linux *was* exercised in CI, which leaves the reader with a falsely gloomy impression.

## Attacks attempted and failed (itemized surrender)

### Vector 1: rustix and cap-fs-ext behaviour reaching Keep. Surrendered

Read from the actual published source diffs.

| Attack | Result |
|---|---|
| `StatVfsMountFlags` | Only `RDONLY` is tested, and the old and new values are both `0x0001` (the linux-raw-sys value I took as known). `fstatvfs` still uses `from_bits_retain(statfs.f_flags)` (`linux_raw/fs/syscalls.rs:981`). |
| Other mount flags stored or compared | `LinuxDirectoryProperties` only copies them. Comparison uses device and mount id. No Debug output. |
| `Padding` fields | Zero uses in Keep, cap-primitives, cap-std, cap-fs-ext or rustix-linux-procfs. Keep reads only `stx_mask`, `stx_dev_*`, `stx_mnt_id`, `stx_ino` and `f_type`. |
| `openat2`, `flock`, `statx`, `fstatfs`, `ioctl_getflags`, `fchdir`, `kill_process_group`, `fcntl_setfl` | The linux_raw fs and process syscall diffs show only 32-bit `chown`, `fchown` and `getgroups` changes. x86_64 and aarch64 are untouched. |
| Apple `fallocate` | No `fallocate` in Keep or the checked capability crates. |
| TIOCSCTTY, termios, inotify, cmsg | No use in Keep's rustix surface; the capability-crate grep is empty for inotify and recvmsg. |
| cap-fs-ext 4.0.3 | Exactly as claimed: import order plus requirement bumps. cap-primitives and cap-std were already 4.0.3 in the lock. |
| Claims about `bitflags`, `libc`, MSRV, licence | All true. |
| Lockfiles | Only cap-fs-ext and rustix move, in both. |

### Vector 2: re2js reachability. Surrendered, reluctantly and after trying properly

- markdownlint's only `RE2JS.compile` call passes no flags (`lib/markdownlint.mjs:812`). The `re` tag uses in md011, md013 and md039 contain no lookbehind.
- `LOOKBEHINDS` does not appear in either package.
- I ran re2js 2.8.6 from a verified tarball. `RE2JS.compile('(?<=a)b')`, `(?<!a)b`, `` re`(?<=a)b` ``, `(?i)(?<=a)b`, `(?L)a` and `translateRegExp(new RegExp('(?<=a)b','u'))` were all **refused**. Only an explicit flag value of 512 compiled.
- Project config: a PR-supplied per-directory `frontMatter` goes through the same flagless compile, so it fails closed. Custom rules need code execution, which is not a re2js route and is not added by this PR.
- The advisory text supports the prose's premise.
- The blast-radius statement is conservative. Job limits are confirmed at `ci.yml#93`, and `TOOL_DEADLINE` is a tighter two minutes (`xtask/src/documentation_integrity/execution.rs#20@00a36e0`).
- The prose calls itself "not a proof", which is correct and annoying of it.

### Vector 3: weakened controls. Surrendered, except S4 and S7

- Test edits are mechanical. Each mutation's replacement string still lands on its target, by inspection.
- No `0.23.3` or `0.41.1` remains outside the CHANGELOG and historical evidence.
- The installer digest is unchanged, and the installer does not appear to name a version.
- `deny.toml` is unchanged. The lock digest is implicitly proven by the green Documentation job on `00a36e0`.
- SHA-512 of the fetched `markdownlint`, `markdownlint-cli2` and `re2js` tarballs matches the lock.
- A malformed test file still produces exact diagnostics under 0.24.0 (5 issues in 1 file).

### Vector 4: install-action. Surrendered, except S5

- `86f7a0f` is the v2.88.0 commit and `1168f68` is v2.87.28. The compare is 99 ahead, 0 behind, and touches only manifests, `TOOLS.md`, the changelog, `tools/ci` and codegen. `main.sh` is untouched.
- #210's "only changed the cargo-binstall version" holds for Keep's tools: no `cargo-deny` or `cargo-audit` manifest change.
- The log confirms the b3sum fallback, `cargo-binstall@latest (1.26.1)` and QuickInstall.
- `cargo-deny` and `cargo-audit` come from upstream releases, not the fallback, so #210 omitted no third fallback.
- No other workflow uses the action.

### Vector 5: #176, #209, #210. Partly surrendered

- The withdrawal is real (`2026-10-23T12:53:02Z`).
- `npm audit` on copies of both locks reports no `braces`. `braces` 3.0.3 remains in the graph.
- #209's facts hold: 0.42.0 is latest and pins re2js 2.8.6 exactly.
- Nothing of substance from #176 was lost. See S6 for the loose ends.

### Vector 6: stale text. Only S8

No links to the old filename remain. The CONTRIBUTING and Standards pages are updated.

### Vector 7: PR body. The rest stands

- "Three red": #207, #189 and #188 show failing checks; #206 none. Confirmed.
- `npm audit` 6→5 and the removals and additions: confirmed on copies of both locks. Severity counts: highs 1→3 packages, but advisory-level highs go 2→1, so "removes more than it adds" stands.
- "All 169 Markdown files pass": **I reproduced this** on a copy: `Linting: 169 files`, `Summary: 0 issues`.
- "Real merge commit": the repository allows only merge commits, so the claimed mechanism is available and cannot be squashed away.
- All AGENTS.md PR sections are present.

## Verdict

```
VERDICT — MORIARTY
Target: flyingrobots/keep#208 @ 00a36e0 vs 3887f2a

STRIKES LANDED: 9
  S1 LOW-MEDIUM  rustix 1.1.6 upstream security fix (GHSA-jh2m-fhpj-3vr7) undisclosed; no Keep exposure found
  S2 LOW-MEDIUM  Review verdict cites a local-only branch and no reviewed SHA
  S3 LOW-MEDIUM  One bisection boundary over three unrelated dependency families (may be a deliberate user choice)
  S4 LOW         RDONLY laws use the same symbol as the predicate; no pinned value or real read-only mount
  S5 LOW         #210 narrower than the PR body says; fuzz-smoke and apt installs untracked
  S6 LOW         #176 closed before its record lands; no audit trigger; katex untracked
  S7 LOW         Upstream rule narrowing (MD011) unreviewed against "no weakened control"
  S8 LOW         Stale present-tense 4.0.2 / 1.1.5 on the renewed admission page
  S9 NIT         "No Keep source changes"; Rust validation uncited; macOS claim has no receipt

CRITICAL / HIGH: 0
BEHAVIOUR DEFECT REACHING KEEP: none found
WEAKENED CONTROL INTRODUCED BY THIS DIFF: none, beyond S7's upstream rule narrowing
LABELLED CONJECTURE: S4's mutation experiment only

ATTACKS FAILED: Vector 1 in full; re2js reachability (with live compile probes);
digest, pins and test mutations; install-action provenance and effects; #176 withdrawal;
#209 facts; audit counts; 169-file lint; merge-commit claim.

OPEN AT TIME OF TESTIMONY: CI run 41663162408 on 00a36e0, Rust quality gates in progress.
Docs-only delta since fully green run 41660329996 on 9878aab.

RESILIENCE GRADE: B+
Engineering claims held under direct attack. Losses are in record-keeping and traceability.
```

Nothing I found should stop this merging on correctness once the Rust gates finish. S1, S2 and S8 are one-paragraph fixes, and S4 is a one-line law. S3 is the only strike that argues for a different shape, and that decision belongs to the user.
