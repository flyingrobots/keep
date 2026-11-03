# Testimony of S. Holmes: Keep `3887f2a..9878aab`

The pins are sound, Watson; the commentary around them less so. Every version, lockfile entry and digest in this diff holds. I find six drifts, all in the explanatory prose. One of them props up the acceptance of a high-severity advisory.

**Evidence basis:** the worktree was clean at `9878aab`, and nothing was edited there.
- `@9878aab` and `@3887f2a` cite the repository.
- `@crate-x.y.z` cites registry source whose checksum is recorded in `Cargo.lock`.
- **[HEARSAY]** marks unpinned evidence, gathered 2026-11-03: GitHub API, CI logs, the npm registry and the `npm audit` database.

## The material drifts

**D1. "Repository-authored" (worst by consequence)**
- **The page says:** "The tool only reads repository-authored Markdown" (`docs/dependencies/documentation-toolchain.md#23@9878aab`).
- **The evidence says:** CI runs on an unrestricted `pull_request` trigger (`.github/workflows/ci.yml#6@9878aab`), and the documentation job lints the checked-out tree (`ci.yml#90@9878aab`). Markdown proposed in a pull request is therefore read before any merge.
- **Why it matters:** this sentence is the stated reason for accepting GHSA-gg6h-894w-4m9c. The page never says that advisory is **high** severity, though it does call the others "low-severity" **[HEARSAY]**.
- **Mitigations the page omits:** a 10-minute job timeout (`ci.yml#93@9878aab`), `contents: read` (`ci.yml#7@9878aab`) and `persist-credentials: false` (`ci.yml#98@9878aab`).
- **Conjecture:** the realistic harm is a stalled CI job. I did not test whether markdownlint reaches re2js lookbehind, nor check the repository's fork-approval settings.

**D2. `statx` (the plainest falsehood)**
- **The page says:** Rustix reaches Keep's boundary "in one place", and the `statx` padding changes "are outside the operations Keep invokes" (`docs/dependencies/cap-std-4.0.3-and-cap-fs-ext-4.0.3.md#89@9878aab`).
- **The evidence says:** Keep invokes `statx` directly (`src/adapters/filesystem_platform_profile.rs#154@9878aab`, `#226@9878aab`).
- **The conclusion still stands:** `Padding` is `#[repr(transparent)]` (`src/types.rs#959@rustix-1.1.6`), and neither Keep nor the locked capability crates name those fields (repository and registry grep). The accurate wording is "an operation Keep invokes, with layout preserved and the fields unread".

**D3. `libc` minimum**
- **The page says:** Rustix's declared minimums for `bitflags` and `libc` rise (`cap-std-…-4.0.3.md#87@9878aab`).
- **The evidence says:** `bitflags` does rise, from 2.4.0 to 2.9.2 (`Cargo.toml#159@rustix-1.1.5`, `#158@rustix-1.1.6`). The runtime `libc` requirement is 0.2.182 in both releases (`Cargo.toml#207,#230@rustix-1.1.5`, `#206,#229@rustix-1.1.6`). Only the dev-dependency moves, from 0.2.171 to 0.2.182 (`Cargo.toml#175@rustix-1.1.5`, `#174@rustix-1.1.6`).

**D4. Rustix licence (pre-existing, on the renewed page)**
- **The page says:** "Rustix declares `Apache-2.0 OR MIT`" (`cap-std-…-4.0.3.md#55@9878aab`).
- **The evidence says:** `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT`, in both releases (`Cargo.toml#52@rustix-1.1.5`, `#52@rustix-1.1.6`). The renewal let this stand, though the same page names licence as a reopening concern (`#65@9878aab`).

**D5. "The CI job's" (cosmetic)**
- **The changelog says:** "the CI job's pinned" action (`CHANGELOG.md#15@9878aab`).
- **The evidence says:** two jobs use it, Rust quality gates (`ci.yml#35@9878aab`) and Dependency policy (`ci.yml#180@9878aab`).

**D6. "Move every pin" (self-corrected)**
- **The message says:** `430d12b` moved every pin.
- **The evidence says:** that commit left the old lock digest (`xtask/src/documentation_integrity/node_toolchain.rs#18@430d12b`). `9878aab` corrects it and says so.

## Reverse drift: what the prose omits

1. **The braces advisory is withdrawn.** GHSA-vfj7-8cjw-p6xm was withdrawn on 2026-10-23 **[HEARSAY]**. The page says it "remains tracked … although `npm audit` did not report it" (`documentation-toolchain.md#23@9878aab`), leaving the absence unexplained. Issue #176 is still open and was last updated 2026-10-03, so it may now be closable **[HEARSAY]**.
2. **The re2js risk is understated.**
   - The severity is never given.
   - Audit totals went from 3 low, 2 moderate, 1 high to 2 low, 0 moderate, 3 high **[HEARSAY]**.
   - The high count rises only because re2js propagates through `markdownlint` and `markdownlint-cli2`. Distinct advisories fell from five to two **[HEARSAY]**.
   - An issue and PR search for re2js returned nothing, whereas braces has #176 **[HEARSAY]**.
3. **The MSRV line was not renewed.** `cap-std-…-4.0.3.md#55@9878aab` still reasons from "Rustix 1.1.5 declares Rust 1.65". The floor is unchanged at 1.65 in 1.1.6 (`Cargo.toml#14@rustix-1.1.6`), so the fact survives but the sentence names the old version in a live section.
4. **The list of "other reviewed changes" is incomplete.** Rustix 1.1.6 also changes:
   - zero-length `getxattr` and `listxattr` semantics;
   - `recvmsg` control-message truncation;
   - 32-bit `setuid`, `setgid` and `setgroups` selection;
   - `inotify` constants;
   - `tcgetpgrp` handling.

   None of these reach Keep or the locked capability crates by grep, and the `net` feature is not resolved (`cargo metadata` at the target). The omission does no harm, but a reader may take the list as exhaustive.
5. **Statx is missing from the API list.** `cap-std-…-4.0.3.md#18@9878aab` lists the production initializer's Rustix APIs without it (pre-existing), despite `filesystem_platform_profile.rs#154@9878aab`.

**Stale pins and obligations:**
- No live stale pin remains. Old versions survive only in historical changelog entries, the dated update sections at `#67-#83` of the admission page, and `docs/testing-evidence`, which the page says keep their recorded coordinates.
- Nothing references the old admission filename.
- The toolchain review trigger for a version or lock graph change (`documentation-toolchain.md#136-#137@9878aab`) has been answered by this update.

**PR state, not prose drift:**
- PR #208's body says its required sections will be filled before it leaves draft **[HEARSAY]**.
- `430d12b` and the merges before `9878aab` are red states on the branch, so only the final integration commit should be treated as a bisection boundary.

## Verdict table

| # | Claim | Verdict | Citation |
|---|---|---|---|
| 1 | Root, task and spawn manifests pin Rustix 1.1.6 | CONFIRMED | `Cargo.toml#23`, `xtask/Cargo.toml#40`, `repository-process-spawn/Cargo.toml#10` `@9878aab` |
| 2 | Root and task manifests pin cap-fs-ext 4.0.3 | CONFIRMED | `Cargo.toml#21`, `xtask/Cargo.toml#32` `@9878aab` |
| 3 | Fuzz lock resolves both versions | CONFIRMED | `fuzz/Cargo.lock#44,#284@9878aab` |
| 4 | Features and remaining package set unchanged | CONFIRMED | `git diff 3887f2a..9878aab -- Cargo.lock fuzz/Cargo.lock '*Cargo.toml'` shows only version and checksum lines in the locks and version-only pin changes in the manifests |
| 5 | `bitflags` minimum rises; locked 2.13.1 and libc 0.2.186 satisfy | CONFIRMED | `Cargo.toml#158@rustix-1.1.6`; `Cargo.lock@9878aab` |
| 6 | `libc` minimum rises | **DRIFT (D3)** | above |
| 7 | cap-fs-ext changes only requirements and one import | CONFIRMED, apart from version, VCS and bundled-lock metadata | `src/dir_entry_ext.rs#1-3@cap-fs-ext-4.0.3` |
| 8 | `linux_raw` moves `StatVfsMountFlags` from `MS_*` to `ST_*` | CONFIRMED | `src/backend/linux_raw/fs/types.rs@rustix-1.1.6`, `RDONLY = 0x0001` |
| 9 | Keep reads only `RDONLY`, value unchanged | CONFIRMED | `filesystem_platform_profile.rs#291@9878aab`; `src/x86_64/general.rs#1366@linux-raw-sys-0.12.1` |
| 10 | `statx` padding is outside Keep's operations; "one place" | **DRIFT (D2)** | above |
| 11 | 32-bit `chown` and `getgroups`, Apple `fallocate`, socket options unused | CONFIRMED | repository and registry grep |
| 12 | Page's own review was of the published source differences | UNVERIFIABLE (act); my own diff supports it | not applicable |
| 13 | Current validation must exercise the new graph | PLANNED; Rust gates and fuzz smoke still running when read **[HEARSAY]** | `cap-std-…-4.0.3.md#91@9878aab` |
| 14 | Historical receipts retain recorded builds | CONFIRMED | `docs/testing-evidence/…/intent-red.txt#21@9878aab` |
| 15 | Locked graph list and admitted-use pins correct | CONFIRMED | `Cargo.lock#65,#409@9878aab`; `cap-std-…-4.0.3.md#34-#49@9878aab` |
| 16 | Rustix licence | **DRIFT (D4, pre-existing)** | above |
| 17 | js-yaml 5.4.3, smol-toml 1.9.1, markdown-it 15.0.3 | CONFIRMED | `package-lock.json#396,#468,#1253@9878aab` |
| 18 | markdown-it and smol-toml reports removed | CONFIRMED **[HEARSAY]** | audit of both locks in scratchpad |
| 19 | markdownlint 0.42.0 requires exactly re2js 2.8.6 | CONFIRMED | `package-lock.json#511,#525@9878aab` |
| 20 | GHSA-gg6h description; patched from 2.8.7 | CONFIRMED **[HEARSAY]** | GitHub advisory API |
| 21 | Overrides forbidden | CONFIRMED | `node_toolchain.rs#47@9878aab`; `node_toolchain/tests.rs#45@9878aab` |
| 22 | Exposure remains until markdownlint moves its pin | CONFIRMED; latest 0.42.0 still pins 2.8.6 **[HEARSAY]** | npm registry |
| 23 | Tool reads only repository-authored Markdown | **DRIFT (D1)** | above |
| 24 | This change adds no outside input | CONFIRMED | diff touches no input-selection code |
| 25 | katex and micromark-extension-math stay low | CONFIRMED **[HEARSAY]** | audit |
| 26 | braces 3.0.3 retained; #176 tracks it; audit silent | CONFIRMED, literally; see reverse drift 1 | `package-lock.json#104@9878aab`; **[HEARSAY]** |
| 27 | No clean audit claimed | CONFIRMED; 5 reports **[HEARSAY]** | audit |
| 28 | Changelog: markdownlint and Rustix entries | CONFIRMED | `CHANGELOG.md#11,#13@9878aab` |
| 29 | Changelog: "the CI job's" | **DRIFT (D5)** | above |
| 30 | Action SHA is v2.88.0 | CONFIRMED **[HEARSAY]** | `ci.yml#35@9878aab`; tag via GitHub API |
| 31 | `99722f2`: fuzz lock missed; `--locked` refused both PRs | CONFIRMED | stats of `27372f3`, `ee985f3`; CI runs 40161693380, 41273080378 **[HEARSAY]** |
| 32 | `430d12b`: contract pins tool, reported version, js-yaml, markdown-it | CONFIRMED | `tool.rs#25,#33@9878aab`; `node_toolchain.rs@9878aab` |
| 33 | `430d12b`: bump refused by the documentation job and xtask laws | CONFIRMED **[HEARSAY]** | CI run 40161675945 (version mismatch; 3 failed tests) |
| 34 | `430d12b`: "Move every pin" | **DRIFT (D6, self-corrected)** | above |
| 35 | `430d12b`: admission reopens on direct version change | CONFIRMED | `cap-std-…-4.0.3.md#65@9878aab` |
| 36 | `9878aab`: BLAKE3 digest pinned and moved | CONFIRMED by independent computation of `b330c608…0e81`, matching the constant | `node_toolchain.rs#17-20,#120@9878aab` |

## VERDICT

```
TARGET:          git diff 3887f2a..9878aab (flyingrobots/keep#208, draft)
CLAIMS CHECKED:  36
CONFIRMED:       28
DRIFT:           6 (D1-D6; 1 pre-existing, 1 self-corrected, 1 cosmetic)
PLANNED:         1
UNVERIFIABLE:    1
REVERSE DRIFT:   5 significant omissions
STALE LIVE PINS: 0
WORST DRIFT:     D1. "repository-authored Markdown" justifies accepting a
                 high-severity advisory whose severity is unstated, while CI
                 lints pull-request content (ci.yml#6@9878aab).
PLAINEST ERROR:  D2. statx "outside the operations Keep invokes", against
                 filesystem_platform_profile.rs#154,#226@9878aab.
MOST USEFUL OMISSION: GHSA-vfj7-8cjw-p6xm was withdrawn 2026-10-23 [HEARSAY].
FIDELITY GRADE:  B
GRADE BASIS:     Every pin, lock entry and digest is exact, and no drift
                 overturns a conclusion. Risk-acceptance wording and the
                 upstream-difference summary need correction before merge.
```
