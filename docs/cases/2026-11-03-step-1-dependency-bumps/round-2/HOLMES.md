# Testimony of S. Holmes: PR #208, verification pass

**Pinned head:** `00a36e0` (`00a36e018f277b3e905b00d6a92b6fbcbc8bbd88`), branch `integrate/open-prs`, against base `3887f2a`. The worktree stayed clean throughout.

**Upstream pins, from each crate's `.cargo_vcs_info.json`:**
- rustix 1.1.5: `287214b`
- rustix 1.1.6: `4a563ad`
- cap-fs-ext 4.0.2: `715e4ed`
- cap-fs-ext 4.0.3: `b7acf8e`

**Evidence not pinned to a commit, labelled hearsay:**
- GitHub API responses
- npm registry and audit results
- CI run logs

The npm tarballs were installed with `npm ci --ignore-scripts` from the lock at `00a36e0`. They are therefore integrity-bound, though they carry no git SHA.

I ran no cargo build or test. I read no prior testimony, and for branch `cases/open-prs-integration` I verified only that it exists.

## Findings

The technical substance holds. Every version, digest, advisory fact and Rustix reasoning step survived independent reproduction. The three drifts are all small: two lie in the PR body's process narrative, and one is an older understatement on a line this diff edited.

### The evidence that matters most

- **Lock digest.** I recomputed BLAKE3 over the committed lock and got `b330c608…14570e81`. This matches `xtask/src/documentation_integrity/node_toolchain.rs#18-19@00a36e0`. The same computation over the base lock gives `9e809c6a…845f5d37`, the digest the diff removed.
- **Audit.** Hearsay, reproduced under `/private/tmp/claude-501/-Users-j-git-keep/0c5d0f19-e5f7-4a34-aac7-bdc48cc6cf8a/scratchpad/holmes-audit/`.
  - Base has 6 reports: 3 low, 2 moderate, 1 high.
  - Head has 5 reports: 2 low, 3 high.
  - Head's entries are `katex`, `micromark-extension-math`, `markdownlint`, `markdownlint-cli2` and `re2js`.
  - `braces` appears on neither graph.
- **re2js refusal.** Reproduced against the locked 2.8.6.
  - `(?<=a)b` and `(?<!a)b` raise `RE2JSSyntaxException` under plain `compile`, under the `re` tag, and through `translateRegExp`.
  - The last is markdownlint's own front-matter path (`markdownlint@0.42.0 lib/markdownlint.mjs#812`).
  - With `RE2JS.LOOKBEHINDS` as a control, the pattern compiles.
  - `LOOKBEHINDS` appears nowhere in the `markdownlint` or `markdownlint-cli2` packages.
- **Markdownlint.** Run the way `xtask/src/documentation_integrity/tool.rs#56@00a36e0` runs it: `Linting: 169 files`, `Summary: 0 issues in 0 files`. Caveat: I ran it on Node 26.10, not the pinned 24.18.0.
- **Keep's Rustix surface, enumerated independently of the page.**
  - Non-test code uses `openat2`, `openat`, `flock`, `fstatfs`, `fstatvfs`, `statx`, `ioctl_getflags`, `makedev`, `fcntl_*`, `kill_process_group` and `fchdir`.
  - Only the padding and `StatVfsMountFlags` changes touch that set.
  - Resolved Rustix features are `alloc, default, fs, process, std, termios, time` (`cargo metadata --locked` on both workspaces). `net` and `thread` are absent, so the socket, `recvmsg` and `set*_thread` hunks are not compiled.
- **`RDONLY`.** `rustix-1.1.5/src/backend/linux_raw/fs/types.rs#559@287214b` derives it from `MS_RDONLY`, which is `1` (`linux-raw-sys-0.12.1/src/x86_64/general.rs#1366`, unpinned registry source). `rustix-1.1.6/src/backend/linux_raw/fs/types.rs#566@4a563ad` sets `0x0001`. Keep's only test of a mount flag is `src/adapters/filesystem_platform_profile.rs#291@00a36e0`.

### One deduction the page does not spell out

`docs/dependencies/cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#91@00a36e0` says the search found no behavioural reach into the listed areas, termios among them. A literal search does find a reach: `cap-primitives-4.0.3/src/rustix/fs/file_path.rs#3@b7acf8e` imports `rustix::termios::ttyname`.

The claim survives. No `ttyname` code changed: the termios hunks are one doc comment, BSD and `nto` type cfgs, and the libc `tcgetpgrp` translation. I record it as CONFIRMED with this footnote.

### Drift

1. **PR body, "The intermediate commits on this branch are red by design."** This is overbroad.
   - `9878aab` had a successful CI run, 41660329996 (hearsay).
   - #206, merged by `429d651`, had no failed check (hearsay).
   - `git diff 9878aab..00a36e0` touches only three documentation files.
   - The PR's real point stands: only the merge of #208 is a bisection boundary.
2. **PR body, "The prose errors it found are fixed in `57bb347`."** This is incomplete. `00a36e0`'s own message says it resolves "the review's deferred findings", including the `statx` addition at `docs/dependencies/cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#18@00a36e0`. The body does not mention that commit.
3. **Reverse drift, `docs/dependencies/cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#14@00a36e0`.** It says the library admits Rustix "for safe Linux filesystem-profile inspection and no-symlink root opening". The library also takes `flock` and `openat` for the writer lock (`src/adapters/filesystem_writer_lock.rs#10@00a36e0`) and `flock` for the reader fence (`src/adapters/retention/reader_fence.rs#7@00a36e0`). The omission predates this PR, but the diff edited the line.

### Reverse-drift notes, not counted

- The page omits three lesser Rustix changes. None alters a claim, since Keep names none of these fields and selects none of these options.
  - `Statx.__spare2` and `__spare3` narrow from `pub` to `pub(crate)` (`rustix-1.1.6/src/fs/statx.rs#50-51@4a563ad`).
  - `StatxTimestamp.__reserved` also becomes `Padding`.
  - The optional `rustc-std-workspace-*` requirements move to 1.0.1.
- `docs/dependencies/cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#32@00a36e0` describes direct declarations only. The resolved union also carries `default`, `alloc`, `termios` and `time`. The Rustix `process, std` selection of the process-spawn crate (`repository-process-spawn/Cargo.toml#10@00a36e0`) is not named in that section.
- `docs/dependencies/cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#75@00a36e0` still says the cap-fs-ext pin "remains 4.0.2". It was true when written and sits under a dated heading, but nothing marks it superseded.
- `docs/dependencies/documentation-toolchain.md#30@00a36e0` bounds the failure case only for CI, though line 13 of the same page says these tools also run on contributor machines. Conjecture: the consequence there is a stalled local command.
- Page line 91 reports only the macOS compile check and gives no Linux evidence. Yet CI on `9878aab` passed on Linux (hearsay; the log shows `x86_64-unknown-linux-gnu` packages). The page understates the evidence.

### Current state (hearsay, time-sensitive)

- #208 is a draft, `UNSTABLE`.
- At `00a36e0`, Documentation, Dependency policy and Runtime fuzz smoke pass. Rust quality gates are still pending.
- #206, #207, #189 and #188 remain OPEN. Their merge commits' second parents do match their PR heads (`04fc1c0`, `ee985f3`, `27372f3`, `30c9f0e`).
- Conjecture: GitHub marks them merged only if #208 lands by a true merge, not a squash.

## Verdict table

| # | Source | Claim | Verdict | Evidence |
|---|---|---|---|---|
| 1 | Admission #1, #14 | Pins cap-std 4.0.3, cap-fs-ext 4.0.3, rustix 1.1.6 | CONFIRMED | `Cargo.toml#21-23@00a36e0`, `xtask/Cargo.toml#32-33,40@00a36e0` |
| 2 | Admission #5-7 | Update dates and issue references | CONFIRMED | Commits dated 2026-11-03; earlier dates from `44c736c`. PR merges were 10-20 (hearsay), so 10-03 is the commit date |
| 3 | Admission #14 | Library Rustix use limited to profile and root opening | DRIFT (reverse) | `src/adapters/filesystem_writer_lock.rs#10@00a36e0` |
| 4 | Admission #18 | `openat2`, `fstatfs`, `fstatvfs`, `statx`, inode flags | CONFIRMED | `src/adapters/filesystem_platform_profile.rs#45,148-154@00a36e0` |
| 5 | Admission #32 | Direct feature selections | CONFIRMED | `Cargo.toml#21-23@00a36e0`, `xtask/Cargo.toml#14-25,40@00a36e0` |
| 6 | Admission #34-49 | Locked graph list | CONFIRMED | Both locks at `00a36e0` |
| 7 | Admission #55 | Licences, floors 1.65 and 1.96.0, `winx` exception | CONFIRMED | `rustix-1.1.6/Cargo.toml#14,52@4a563ad`, `rust-toolchain.toml@00a36e0`, `deny.toml#20@00a36e0` |
| 8 | Admission #65 | Reopens on direct version change | CONFIRMED | The line itself; the PR did so |
| 9 | Admission #87 | Three manifests select 1.1.6; two pin cap-fs-ext | CONFIRMED | Row 1, `repository-process-spawn/Cargo.toml#10@00a36e0` |
| 10 | Admission #87 | Fuzz resolves both; other packages unchanged | CONFIRMED | Lock diffs change only these two packages; `cargo metadata --locked` passes |
| 11 | Admission #87 | `bitflags` 2.4.0 to 2.9.2; locked 2.13.1 | CONFIRMED | `rustix-1.1.5/Cargo.toml#159@287214b`, `rustix-1.1.6/Cargo.toml#158@4a563ad` |
| 12 | Admission #87 | Runtime `libc` stays 0.2.182; only dev moves | CONFIRMED | `rustix-1.1.5/Cargo.toml#175,207,230@287214b`, `rustix-1.1.6/Cargo.toml#174,206,229@4a563ad` |
| 13 | Admission #89 | cap-fs-ext: requirements, one import order, metadata only | CONFIRMED | `diff -r`; `cap-fs-ext-4.0.3/src/dir_entry_ext.rs#1-3@b7acf8e` |
| 14 | Admission #89 | `MS_*` constants replaced by `ST_*` | CONFIRMED | `rustix-1.1.6/src/backend/linux_raw/fs/types.rs#546-566@4a563ad` |
| 15 | Admission #89 | Only `RDONLY` tested; value unchanged | CONFIRMED | See evidence above |
| 16 | Admission #89 | `Padding` in `Statx`, `StatFs`, `Stat`; `repr(transparent)` | CONFIRMED | `rustix-1.1.6/src/types.rs#14-16@4a563ad`, `rustix-1.1.6/src/fs/statx.rs#28,50-51@4a563ad` |
| 17 | Admission #89 | Keep and capability crates name no padding field | CONFIRMED | Search of Keep and seven locked crates returned nothing |
| 18 | Admission #91 | Listed other touched areas | CONFIRMED | `rustix-1.1.6/src/net/send_recv/msg.rs#546@4a563ad`, `rustix-1.1.6/src/backend/libc/fs/syscalls.rs#1817@4a563ad`, and the sockopt, chown32, getgroups32, `tcgetpgrp`, inotify and xattr hunks |
| 19 | Admission #91 | No behavioural reach | CONFIRMED | With the `ttyname` footnote above |
| 20 | Admission #91 | `cargo check` passes on `aarch64-apple-darwin` | UNVERIFIABLE | Not run under the charter |
| 21 | Admission #91 | `aarch64-unknown-linux-gnu` not checked | UNVERIFIABLE | A statement of absence |
| 22 | Admission #93 | Current validation must exercise the boundaries | PLANNED | Rust quality gates pending at head (hearsay) |
| 23 | Toolchain #8, #23 | 0.24.0 with `js-yaml` 5.4.3, `smol-toml` 1.9.1, `markdown-it` 15.0.3 | CONFIRMED | `scripts/documentation-tools/package-lock.json#396,468,1253@00a36e0` |
| 24 | Toolchain #27 | Five reports from two advisories | CONFIRMED | Audit reproduction (hearsay) |
| 25 | Toolchain #29 | Three `markdown-it` and one `smol-toml` advisory removed | CONFIRMED | Base versus head audit (hearsay) |
| 26 | Toolchain #30 | Exact pin, high, quadratic, patched at 2.8.7 | CONFIRMED | `scripts/documentation-tools/package-lock.json#525@00a36e0`; GitHub advisory API (hearsay) |
| 27 | Toolchain #30 | Advisory requires `LOOKBEHINDS` | CONFIRMED | Advisory text and control table (hearsay) |
| 28 | Toolchain #30 | Published packages do not set the flag | CONFIRMED | Search of the installed locked packages |
| 29 | Toolchain #30 | Refused at compile time without the flag | CONFIRMED | Reproduced |
| 30 | Toolchain #30 | Config sets no front-matter pattern | CONFIRMED | `.markdownlint-cli2.yaml@00a36e0`; the only such config tracked |
| 31 | Toolchain #30 | Job lints PR Markdown | CONFIRMED | `.github/workflows/ci.yml#6@00a36e0` |
| 32 | Toolchain #30 | Ten-minute limit, read-only, no persisted credentials | CONFIRMED | `.github/workflows/ci.yml#8-9,93,98@00a36e0` |
| 33 | Toolchain #30 | Contract refuses `overrides` | CONFIRMED | `xtask/src/documentation_integrity/node_toolchain.rs#47-48@00a36e0` |
| 34 | Toolchain #30 | #209 tracks the update and repeats the check | CONFIRMED | Issue #209 (hearsay) |
| 35 | Toolchain #31 | `katex` retained, low, via `micromark-extension-math` | CONFIRMED | Audit and API (hearsay) |
| 36 | Toolchain #32 | `braces` 3.0.3 retained; withdrawn 2026-10-23; #176 closed | CONFIRMED | `scripts/documentation-tools/package-lock.json#104@00a36e0`; `withdrawn_at` 2026-10-23T12:53:02Z (hearsay) |
| 37 | Changelog #188 | Removals and high re2js addition | CONFIRMED | Rows 25-26 |
| 38 | Changelog #207, #189 | Both workspaces; features and floor unchanged | CONFIRMED | Rows 7, 10 |
| 39 | Changelog #206 | Two jobs; cargo-binstall to 1.26.1; #210 tracks | CONFIRMED | `.github/workflows/ci.yml#17,35,168,180@00a36e0`; manifests 1.25.3 to 1.26.1 (hearsay) |
| 40 | PR, diff | Every pin and the BLAKE3 digest moved | CONFIRMED | Digest recomputed; no `0.23.3` or `0.41.1` outside `CHANGELOG.md` at head |
| 41 | PR Problem | Three red; fuzz lock and contract pins missing | CONFIRMED | Dependabot commit stats; failed checks (hearsay) |
| 42 | PR | Four real merge commits on the branch | CONFIRMED | Parents of `429d651`, `be0faa7`, `761df52`, `5ea366c` |
| 43 | PR | "So GitHub marks it merged" | PLANNED | All four PRs OPEN (hearsay) |
| 44 | PR | No identity, format or durability code change | CONFIRMED | No diff under `src`, `tests` or fuzz sources |
| 45 | PR | `npm audit`: 6 before, 5 after | CONFIRMED | Reproduced (hearsay) |
| 46 | PR | All 169 Markdown files pass | CONFIRMED | Reproduced; Documentation job also passes at head (hearsay) |
| 47 | PR | Each mutation still hits its target | CONFIRMED | Read from `xtask/src/documentation_integrity/node_toolchain/tests.rs@00a36e0`, not executed |
| 48 | PR | Intermediate commits are red by design | DRIFT | Drift 1 above |
| 49 | PR | No Keep source changes | CONFIRMED | True of `src/`; `xtask/src` pins did move |
| 50 | PR | #176, #209, #210 dispositions | CONFIRMED | Issue states (hearsay) |
| 51 | PR | Baker Street overall B, no defect | UNVERIFIABLE | Prior testimony barred by charter |
| 52 | PR | Prose errors fixed in `57bb347` | DRIFT | Drift 2 above |
| 53 | PR | Testimony on local branch | CONFIRMED | Branch exists at `ea253ea`, absent from `origin`, so outside readers cannot check it |
| 54 | PR | Later steps land as separate PRs | PLANNED | Future act |
| 55 | #176 comment | Withdrawn; `braces` unreported on both graphs | CONFIRMED | Rows 24, 36 |
| 56 | #209 | Evidence items; 0.42.0 is latest | CONFIRMED | npm registry (hearsay); rows 28-29, 33 |
| 57 | #210 | Contract inspects only `jobs.documentation` | CONFIRMED | `xtask/src/documentation_integrity/workflow_contract.rs#93@00a36e0` |
| 58 | #210 | Run 41660329996: fallback, 1.26.1, QuickInstall | CONFIRMED | Run log (hearsay). The issue omits that the log also records a verified signature |
| 59 | #210 | Predates #206; #206 changes only the binstall version in use | CONFIRMED | Compare touches manifests, `TOOLS.md`, `CHANGELOG.md` and `tools/`; no `cargo-deny` or `cargo-audit` manifest (hearsay) |
| 60 | #209, #210 | Acceptance outcomes | PLANNED | Both issues open |

```
VERDICT
target:        flyingrobots/keep#208 @ 00a36e0 vs 3887f2a
claims:        60
CONFIRMED:     50
DRIFT:         3 (1 reverse, 2 PR-body narrative)
PLANNED:       4
UNVERIFIABLE:  3
worst_drift:   PR body "The intermediate commits on this branch are red by design":
               9878aab ran green and #206 had no failed check (both hearsay).
               Process claim only; no pin, digest, advisory or safety reasoning is affected.
fidelity:      A
scratch:       19M retained at /private/tmp/claude-501/-Users-j-git-keep/0c5d0f19-e5f7-4a34-aac7-bdc48cc6cf8a/scratchpad/holmes-audit
```
