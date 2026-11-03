# Testimony of Professor Moriarty: `3887f2a..9878aab`, branch `integrate/open-prs`

My dear detective, I came for a corpse and found only a forged death certificate. The code stands. Its paperwork lies in small, well-mannered ways, and I shall enjoy naming each one.

**Evidence basis:** base `3887f2a`, target `9878aab`. Upstream citations use `<path>#<line>@<crate version>` or a diff hunk header. No file in any worktree was changed.

**Evidence limit:** when I last looked, CI run `41660329996` on `9878aab` was still `in_progress`.
- Finished with success: `Documentation and workflow integrity`, `Dependency policy`.
- Not finished: `Rust quality gates`, `Runtime fuzz smoke`. I assert nothing about those two, and I did not run cargo.

## Strikes landed

### Strike 1: LOW-MEDIUM. The admission says Keep does not invoke an operation it does invoke

- **The claim:** "The other reviewed changes to `statx` padding fields … are outside the operations Keep invokes on its supported targets." (`docs/dependencies/cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#89@9878aab`)
- **The evidence:** Keep calls `statx` on the production admission and identity paths. (`src/adapters/filesystem_platform_profile.rs#154@9878aab`, `#226@9878aab`)
- **The omission:** the same patch changes `StatFs.f_spare` to `Padding` (`src/backend/linux_raw/fs/types.rs` hunk `@@ -795,9 +802,9 @@`@rustix-1.1.6). Keep calls `fstatfs` at `#150@9878aab`, and `fstatvfs` at `#151@9878aab` is built from `StatFs` through `statfs_to_statvfs` (`src/backend/linux_raw/fs/syscalls.rs#961@rustix-1.1.6`). The admission never mentions this. `Stat` padding on the x86_64, aarch64 and riscv64 layouts also goes unmentioned.
- **Behaviour impact: none found.** `Padding<T>` is `#[repr(transparent)]` over `MaybeUninit<T>` (`src/types.rs#14-16@rustix-1.1.6`), and Keep reads no spare field.
- **Why it still counts:** the conclusion is right but the reason is false. The true sentence is "Keep invokes these, but reads no padding field and the layout is unchanged". In a storage admission, that difference matters.

### Strike 2: LOW. A false factual claim about `libc`

- **The claim:** "Rustix's declared minimums for its `bitflags` and `libc` dependencies rise." (`docs/dependencies/cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#87@9878aab`)
- **The evidence:** the normal `libc` requirement is `0.2.182` in both releases (`Cargo.toml#207,#230@rustix-1.1.5`, `#206,#229@rustix-1.1.6`). Only the dev-dependency moved, from `0.2.171` to `0.2.182` (`#175@1.1.5`, `#174@1.1.6`), and it does not reach Keep's resolution.
- The `bitflags` half is true (`2.4.0` to `2.9.2`). This is a dependency-review sentence that misread the manifest it reviewed.

### Strike 3: LOW-MEDIUM. The re2js argument is unsound, and the sound argument is missing

- **The claim:** "The tool only reads repository-authored Markdown… this exposure remains until `markdownlint` moves its pin." (`docs/dependencies/documentation-toolchain.md#23@9878aab`)

There are three defects:

1. **"Repository-authored" is not a trust boundary.** CI runs on an unconditional `pull_request` trigger (`.github/workflows/ci.yml#6@9878aab`), and the repository is public (`gh api repos/flyingrobots/keep`). A fork's Markdown can therefore reach the linter, subject to GitHub's first-time-contributor approval setting, which I did not read. The real limits are the ten-minute job timeout and the bounded runner, and the sentence relies on neither.
2. **The real defence is never stated.**
   - The advisory requires the opt-in `LOOKBEHINDS` flag.
   - markdownlint 0.42.0 never sets it. Its only re2js uses are `lib/md011`, `md013`, `md039` and `markdownlint.mjs#812`, and `re2js` is imported nowhere in markdownlint-cli2's package.
   - The only `(?<=` in its code is a native `RegExp` (`helpers/helpers.cjs#191`).
   - I ran it. Both `` re`(?<=a)b` `` and `RE2JS.compile(RE2JS.translateRegExp(/(?<=a)b/))` throw `RE2JSSyntaxException: invalid named capture`.
   - So the vulnerable path looks unreachable, and "this exposure remains" overstates the risk. Only a user-configured lookbehind front-matter pattern could differ, and that also throws. `.markdownlint-cli2.yaml` sets no `frontMatter`.
3. **Severity is selectively labelled.** The paragraph calls `katex` and `micromark-extension-math` "low-severity" but omits that GHSA-gg6h-894w-4m9c is **high**. My `npm audit --package-lock-only` on the target lock gives 3 high and 2 low, rolling up to `markdownlint-cli2` at high.

The decision to accept survives. The written reasoning does not.

### Strike 4: LOW. "Reviewed" is a claim with no receipt and a partial list

`docs/dependencies/cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#89@9878aab` names `StatVfsMountFlags`, `statx`, `chown`, `getgroups`, Apple `fallocate` and socket options. It stands in for a diff of 28 source files.

- **Not named, by size:** the largest hunks, `src/fs/xattr.rs` (232 lines) and `src/termios/types.rs` (145 lines).
- **Not named, in features Keep's graph enables:** `src/process/ioctl.rs`, and termios through cap-primitives' `termios` feature (`Cargo.toml#73-80@cap-primitives-4.0.3`). Also unnamed are `fs/dir.rs`, `inotify`, `msg.rs`, `futex`, `clock` and `sendfile` gating, about nineteen files in all.
- **The `chown` entry is also incomplete:** the 32-bit selection change covers `fchown` too.
- **The phrase "on its supported targets":** I found no document defining Keep's supported target set. The qualifier does no work anyway, because Keep calls neither `chown` nor `getgroups` at all.

I found no behavioural reach in any omitted file, so "one place that needs review" survives on the facts. A reader still cannot tell which omissions were reviewed and dismissed, and which were never read.

### Strike 5: LOW. Stale version in an evergreen section (vector 6)

- `docs/dependencies/cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#55@9878aab`, in "Safety, licensing, and compatibility", still says "Rustix 1.1.5 declares Rust 1.65". Every other current-state section was updated (`#14`, `#48`).
- The value is unchanged in 1.1.6 (`Cargo.toml#14@rustix-1.1.6`), so nothing false is implied. But the admitted version's floor is not asserted anywhere.
- `#75@9878aab` still says "The direct cap-fs-ext pin remains 4.0.2" in the present tense. It sits in a section dated by its heading, and `#87` supersedes it. I consider that tolerable.

### Strike 6: LOW. The install-action bump has an undocumented reach and no contract

- **Pinning is correct.** See the failed attacks below.
- **No contract covers it.** The workflow contract inspects only `jobs.documentation` (`xtask/src/documentation_integrity/workflow_contract.rs#93@9878aab`). Both `install-action` uses sit in `quality` and `dependency-policy` (`.github/workflows/ci.yml#35,#180@9878aab`). This gap predates the merge, so it is an absent control and not a downgrade.
- **The oracle's installer changed.** `b3sum` has no manifest at v2.88.0 (`gh api .../contents/manifests?ref=v2.88.0`). The action's README says such tools fall back to cargo-binstall, and `manifests/cargo-binstall.json` moved from `1.25.3` to `1.26.1` in this range. So this bump changes how the independent vector oracle `b3sum@1.8.5` gets installed, while `CHANGELOG.md#15@9878aab` presents it as a simple pin update.
- **A small wording slip:** "the CI job's" is singular, and two jobs use the action.

### Conjecture, not counted as a strike

**Claim:** no CI lane compiles rustix 1.1.6 for Apple, aarch64 Linux or any `libc`-backend Linux target. All jobs are `ubuntu-latest` (`.github/workflows/ci.yml#18,#92,#127,#169@9878aab`). Yet the repository keeps `aarch64-apple-darwin` benchmark baselines, and 1.1.6 widens the termios constants compiled on Apple.

**What I checked:** `OFILL`, `OFDEL`, `NLDLY`, `NL0` and `CRDLY` all exist in Apple's module in libc 0.2.186, so I predict no failure.

**Experiment:** run `cargo check --locked --all-features` on `aarch64-apple-darwin` and on `aarch64-unknown-linux-gnu`.

## Attacks attempted and failed

Failed attacks are certification, and it pains me to sign them.

**Vector 1: rustix and cap-fs-ext reach**
- `StatVfsMountFlags::RDONLY` is `0x0001` in 1.1.6 and `MS_RDONLY = 1` in `linux-raw-sys-0.12.1/src/x86_64/general.rs#1366`. `f_flag` still uses `from_bits_retain`, and the `libc` backend's flag constants did not change.
- Keep touches `RDONLY` only, plus `empty()` in tests. The read-only refusal stands. No Keep test pins the numeric value, though, so its tests would not notice if that constant changed.
- No diff touches the implementations of `flock`, `openat2`, `fchdir` or `kill_process_group`. Nothing changed under `backend/*/process` except `getgroups`. `build.rs` is untouched.
- `fchdir` in the spawn hook keeps its async-signal-safe posture, because no code on that path changed.
- cap-fs-ext 4.0.3 is exactly as claimed: version requirements plus one import reorder (full diff read).
- Apple `fallocate` has no caller in Keep, cap-primitives or fs-set-times. No crate I checked enables `net`.
- Removing `pub` from `__spare*` breaks no Keep code (grep).

**Vector 2: prose claims that held**
- I reproduced the audit on today's date. The old lock reports `markdown-it` and `smol-toml`. The new lock reports neither, and `braces` is absent. `katex` and `micromark-extension-math` are retained at low.
- The bare claim that `markdown-it` and `smol-toml` were removed is therefore true.
- "Current validation must…" is a stated requirement, not a boast, and I found no tense inflation there.

**Vector 3: control downgrades**
- `deny.toml` is unchanged. `INSTALLER_DIGEST` is unchanged and the installer was not modified.
- The new `LOCK_DIGEST` is enforced by `lock bytes match the reviewed digest`, and the documentation job passed on `9878aab`. I could not recompute BLAKE3 locally because `b3sum` is absent, so I rely on that CI result.
- Every replaced test string was checked, and each mutation still hits its target: the first `"5.4.3"` is the cli2 dependency edge, and `"version": "15.0.3"` still exists with its comma. Expected and observed values move together, and every law still refuses.
- `lock_graph.rs` contains no version strings. The override refusal is intact.

**Vector 4: the action pin**
- The tag `v2.88.0` resolves directly to commit `86f7a0ffb25f40114606501a311ebcfaf882ea32` ("Release 2.88.0"), which is a full SHA matching its comment in both places.
- The range is 99 commits ahead and 0 behind, touching only manifests, tool lists and the changelog.
- No `cargo-deny` or `cargo-audit` manifest changed.
- Commits are unsigned, the same as the previous pin, so this is no regression.

**Vector 5: re2js exposure**
- The vulnerable path looks unreachable, as Strike 3 shows. There is no practical attack here. Only the written argument is broken.
- npm `overrides` really are refused.

**Vector 6: remaining old pins**
- Remaining `4.0.2`, `1.1.5` and `0.23.3` references are historical changelog entries, dated sections or testing evidence, except the two in Strike 5.
- Nothing links to the old admission filename.
- There are only two Cargo lockfiles, both updated, each locking `bitflags` 2.13.1 and `libc` 0.2.186. The lock diffs touch exactly two packages.
- The checksums for `rustix-1.1.6.crate` and `cap-fs-ext-4.0.3.crate` match my local registry cache.
- The banner string matches the published source (`constants.mjs#4`, `markdownlint-cli2.mjs#28`).

## Repairs, all in prose

1. **Line 89 of the admission:** say Keep invokes `statx` and `fstatfs`, reads no padding, and the layout is unchanged. List the remaining changed areas, or link the reviewed diff.
2. **Line 87 of the admission:** limit "rise" to `bitflags`.
3. **`documentation-toolchain.md`, line 23:** rest the re2js acceptance on the absent `LOOKBEHINDS` flag and the job timeout. Drop "repository-authored" and state the high severity.
4. **Line 55 of the admission:** update the version to 1.1.6.
5. **Changelog or admission:** optionally note that the bump changes the cargo-binstall route used to install `b3sum`.

## VERDICT

```
TARGET:            git diff 3887f2a..9878aab (integrate/open-prs)
STRIKES LANDED:    6 (all documentary or control-coverage; none is a behaviour defect)
  S1 LOW-MEDIUM    admission says statx is not invoked; fstatfs padding unmentioned
  S2 LOW           false claim that the rustix libc minimum rises (dev-dep only)
  S3 LOW-MEDIUM    re2js reasoning unsound; real defence and "high" severity omitted
  S4 LOW           "reviewed" has no receipt; about 19 changed files unnamed
  S5 LOW           stale Rustix 1.1.5 in an evergreen section
  S6 LOW           no contract on install-action; cargo-binstall bump changes oracle install
CONJECTURE:        1 (no Apple or aarch64 compile lane; defect not predicted; experiment given)
BEHAVIOUR BREAKS:  0 found by source review
CONTROL DOWNGRADES:0 found
VECTORS 1-7:       behaviour attacks on 1, 3, 4, 5 failed; prose strikes on 2, 5, 6; 7 conjecture only
OPEN EVIDENCE:     Rust quality gates and Runtime fuzz smoke still in progress on 9878aab
RESILIENCE GRADE:  B+ (A- for code and controls; marked down for false admission prose)
RECOMMENDATION:    Repair prose (S1-S3 at minimum) before merge; no code change indicated;
                   do not call it validated until the two pending CI jobs pass
```

Scratch files are under `/private/tmp/claude-501/-Users-j-git-keep/0c5d0f19-e5f7-4a34-aac7-bdc48cc6cf8a/scratchpad/`: `audit-old`, `audit-new`, `pk`, and `lb.mjs`.
