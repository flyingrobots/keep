# Testimony of Professor Moriarty: PR #208 at `de515f3`

Head verified: `de515f351b3bd7b7c8d5ee909fa3ed4c520e9634`, base `3887f2a`.

How disappointing, my dear Holmes: the engineering holds. I could not reach Keep's behaviour through anything in this diff. The prose is another matter, and one security argument does not survive its own standard.

Citations to registry and npm packages use a version in place of the sha. My experiments ran on local Node 26.10.0, while CI pins 24.18.0. Timings are single samples. Scratch is under `/private/tmp/claude-501/-Users-j-git-keep/0c5d0f19-e5f7-4a34-aac7-bdc48cc6cf8a/scratchpad/moriarty/`, 21 MiB. No repository file changed.

## Strikes

### S1. MEDIUM: "it removes more than it adds" counts reports, not exposure

**Claims:**
- PR body, Alternatives rejected: "it removes more than it adds, and the new one looks unreachable."
- PR body, Tests: "6 reports before, 5 after."
- `docs/dependencies/documentation-toolchain.md#119@de515f3`

**Evidence:** The added advisory received a reachability review. The removed ones received none, and the doc admits it. So I did that review, and it took a load trace:

| Package | 0.23.3 with Keep's config | 0.24.0 with Keep's config |
|---|---|---|
| `markdown-it` (2 high, 1 low removed) | **not loaded** | not loaded |
| `re2js` (1 high added) | absent | **loaded** |
| `smol-toml` (1 medium removed) | loaded | loaded |

- `markdown-it` sits behind a lazy factory (`markdownlint-cli2/markdownlint-cli2.mjs#756-758@0.23.3`). The factory is needed only for a custom rule that requires that parser, and Keep sets no `customRules` (`.markdownlint-cli2.yaml#1-10@de515f3`).
- `re2js` runs on all input: the default front-matter pattern (`markdownlint/lib/markdownlint.mjs#812@0.42.0`) and MD011 on each line (`markdownlint/lib/md011.mjs#10@0.42.0`).
- `npm audit --package-lock-only`, run on both locks:
  - before: `{high 1, moderate 2, low 3}`
  - after: `{high 3, low 2}`
  - So "6 to 5" hides npm's high bucket tripling, while unique advisories fall from 5 to 2. The count flatters in one direction and undersells in the other, so it measures nothing.

**Consequence:** On Keep's path, the two high advisories this bump removes were not even loaded. It adds one high in a loaded package, unreachable by the flag argument. The bump is freshness, not a demonstrated security gain. The decision may still stand, but this reason for it does not.

**CONJECTURE:** `smol-toml` parses only TOML configuration, so it is also unreached in Keep (`markdownlint-cli2/parsers/toml-parse.mjs#3@0.23.3`). Experiment: trace `parse` calls on a run with Keep's config.

**Repair:** Say the removed `markdown-it` advisories were not loaded on Keep's path either, and that #188 is kept for currency rather than risk reduction. Drop the 6-to-5 count.

### S2. LOW: the word "directly" carries the katex sentence

**Claim:** "neither `markdownlint` nor `markdownlint-cli2` imports `katex` or the math HTML renderer directly" (`docs/dependencies/documentation-toolchain.md#114@de515f3`).

**Evidence:** The sentence is literally true and misleading in context. An ESM load trace of `markdownlint` shows these loading on every start:
- `micromark-extension-math/lib/html.js`
- `katex/dist/katex.mjs`

The chain is `markdownlint/lib/micromark-parse.mjs#7@0.42.0`, then `micromark-extension-math/index.js#3@3.1.0`, then `micromark-extension-math/lib/html.js#6@3.1.0`.

The honest sentence is stronger and shorter: katex is loaded at import, but `mathHtml` and `renderToString` are never called. My grep of `markdownlint/lib`, `markdownlint/helpers` and cli2's top-level modules found no reference to `mathHtml`.

### S3. LOW: MD011 narrows in a second way the record doesn't mention

**Claim:** "At least one rule narrows: MD011 no longer reports reversed links inside HTML blocks" (`docs/dependencies/documentation-toolchain.md#129@de515f3`).

**Evidence:** The HTML case is true, and I reproduced it. "At least one" also saves the sentence from being false. But the same upstream edit has a second effect the record misses:

| Input | 0.41.1 | 0.42.0 |
|---|---|---|
| `<div>` block containing `(text)[link]` | reports | silent (documented) |
| `(a)[b](c)[d]` | reports `(c)[d]` | **silent (undocumented)** |

The likely mechanism: the old `(?!\()` lookahead became a post-match filter (`markdownlint/lib/md011.mjs#10@0.42.0`, `#33@0.42.0`). A rejected match still consumes the `]` that the next match needs as its leading character. This silent miss sits outside HTML, in ordinary prose.

### S4. LOW: the RDONLY law's name claims more than it tests

**Claim:** `the_read_only_mount_flag_has_the_kernel_st_rdonly_value`, with the comment "only this law notices a dependency changing the value admission depends on" (`src/adapters/filesystem_platform_profile_tests.rs#16@de515f3`).

**Evidence:**
- **It can fail, but only one way.** It fails if upstream edits the constant. It compares a rustix literal to a Keep literal and no kernel is consulted, so "kernel" in the name is an assumption.
- **It would have passed before the change it guards.** At 1.1.5, `RDONLY` is `MS_RDONLY` (`rustix/src/backend/linux_raw/fs/types.rs#559@1.1.5`), which is `1` (`linux-raw-sys/src/x86_64/general.rs#1366@0.12.1`). So the law cannot distinguish the versions, and no RED was possible from this bump.
- **Admission depends on two things, and the law pins only one.** The other is the pass-through at `rustix/src/backend/linux_raw/fs/syscalls.rs#981@1.1.6`, reached from `src/adapters/filesystem_platform_profile.rs#151@de515f3`. A rustix change there leaves this law green. My grep found no test with a real read-only mount.
- **It has not yet run in the runs I examined.** Run 41660329996 predates the law, and the Rust gates at `de515f3` were still in progress at my last check.

The value itself is correct, and the law is cheap and harmless. Its name and comment oversell it. "Only" should read "only for the constant."

### S5. LOW: "the final head's run is the merge gate" describes a habit, not a gate

**Claim:** PR body, Tests.

**Evidence:**
- `main` has no classic branch protection (API 404).
- Its only ruleset, `proteckt`, has `non_fast_forward`, `deletion` and `required_signatures`, and no required status checks. It also has an always-bypass actor.
- With Rust gates still running, `gh pr view` reported `MERGEABLE` and `UNSTABLE`, which suggests GitHub would not hold the merge for that job.

Nothing in GitHub enforces this gate. It holds only if the maintainer waits.

### S6. NIT: "#176 closes after this merge"

`closingIssuesReferences` on #208 is empty, and #176 is OPEN with its own acceptance list. Nothing closes it automatically, so this is a manual step the PR states as fact. Name who does it, or add the closing keyword.

### S7. NIT: "removes the affected version from the graph"

This is true of Keep's two lock graphs. But CI's `cargo install cargo-fuzz --locked` still compiles rustix 1.1.4, which is inside the advisory's range (`>= 1.0.0, <= 1.1.5`):
- Run 41660329996: `Downloaded rustix v1.1.4`, and also `Compiling rustix v1.1.4`, in "Install pinned fuzz runner".
- Base run 39396158932 shows the same, so this predates the PR.

The record doesn't mention it. I did not check that tool's xattr use, so this is only a scope note.

## Attacks that failed

| Area | Attack | Result |
|---|---|---|
| Rustix security paragraph | Fake advisory, wrong severity, wrong range, no real fix, or xattr reach | **Surrender.** The repository advisory is published and rated high, and affects `>= 1.0.0, <= 1.1.5` on zero-size queries. The global advisory API returns 404, but the doc links the repository advisory. `patched_versions` is blank, but `rustix/src/fs/xattr.rs@1.1.6` does contain the fix. No xattr use in Keep, cap-std, cap-fs-ext, cap-primitives, fs-set-times or rustix-linux-procfs. |
| "What was reviewed" | Registry differs from Git; unchecked claims | **Surrender.** Crate SHA-256s equal both lock checksums. Tag v1.1.6 dereferences to the crate's `sha1`, `4a563ad`. The upstream comparison's source files match the registry diff by filename. The cap-fs-ext diff is exactly as claimed. Also confirmed: the `MS_*` to `ST_*` change, `Padding`, `bitflags` 2.9.2, `libc` 0.2.182 runtime with only the dev dependency moving, MSRV 1.65, the three-way license in both versions, the resolved features via `cargo tree`, the `ttyname` call in cap-primitives with no change in the libc backend diff, and `statx` already used on base. |
| "What was run" and run 41660329996 | Wrong head, skipped jobs, ignored laws | **Surrender.** Head `9878aab`, success, on ubuntu-24.04. Debug and release each have 128 test-result lines, with none reporting ignored tests. Filesystem laws appear by name. `9878aab..de515f3` leaves the lockfiles and `scripts/` unchanged. |
| re2js "one place configuration reaches" | Rule options, inline config, hostile config, inline flags | **Surrender.** Only `md011`, `md013`, `md039` and `markdownlint.mjs` import it, and the rule patterns are fixed. MD013's `line_length` is a number compared after matching, never in a pattern. A hostile `frontMatter: "(?<=a)b"` exits with rc 2, `invalid named capture`. `(?L)` is also refused. A custom rule could set the flag, but that is already arbitrary code execution, not this path. |
| LOOKBEHINDS | Flag set somewhere, or the `re` tag enables it | **Surrender.** Absent from `markdownlint/lib` and `helpers` and from cli2's top-level modules. The `re` tag compiles with no flags. |
| Two-minute and ten-minute limits | Wrong constant, tool outside the runner | **Surrender.** `xtask/src/documentation_integrity/execution.rs#20@de515f3`, `#204@de515f3`, `.github/workflows/ci.yml#93@de515f3`. |
| Hostile Markdown stalls | Superlinear path newly opened by re2js | **Surrender.** `(a)[b] ` repeated takes 3.2s at 256 KB and 13.6s at 512 KB on 0.42.0, against 3.4s and 15.4s on 0.41.1. That superlinearity exists on both versions. At 1 MB both die with `RangeError` in the parser, so the failure is a tool failure on both. |
| Behaviour reach into Keep | `fstatvfs`, `statx`, `openat2`, `flock`, padding, identity | **Surrender.** In the linux_raw fs syscalls, only 32-bit chown selection changed. `f_flag` still uses `from_bits_retain`. Only `RDONLY` is tested (`src/adapters/filesystem_platform_profile.rs#291@de515f3`), and flags don't enter identity. Keep reads no padding field and constructs no `Statx` or `StatFs` literal. |
| xtask laws | Seeded bad input now equals the admitted value | **Surrender.** `999.0.0` and `5.2.1` still differ from the admitted values, and `lock_graph.rs` has no stale version strings. |
| Pins and #206 | Wrong SHA, false binstall claim | **Surrender.** Both SHAs match their tags. binstall moves from 1.25.3 to 1.26.1, and the QuickInstall warning appears in both runs' logs. |
| Remaining PR claims | Red status, missing merge paths, counts | **Surrender.** #207, #189 and #188 are red and #206 is green. The file lists match. All four heads are ancestors of `de515f3`. The repository allows only merge commits. All 14 commits are verified as signed. I linted 169 files with 0 issues. The audit count and versions match, the `braces` advisory shows `withdrawn_at` 2026-10-23, and markdownlint 0.42.0 is latest. |
| Not attempted | | The Apple Silicon `cargo check` (cargo builds forbidden; no receipt or commit named) and the Baker Street grades and fix claims (case material forbidden). These are untested, not confirmed. |

## Verdict

The technical record survives hostile reading, and I found no defect in Keep's behaviour. The one real wound is S1, a security justification that applies its reachability standard to one side of the trade. The others are overstatements in a record whose whole job is exact statements: a "directly" doing too much work, an incomplete MD011 sentence, an oversold test name, and an unenforced "gate."

```text
VERDICT
target:          flyingrobots/keep#208 @ de515f3 (base 3887f2a)
strikes:
  S1  MEDIUM  "removes more than it adds": removed markdown-it highs not loaded on Keep's path; re2js loaded; high bucket 1->3
  S2  LOW     katex "directly": html.js and katex.mjs load on every start; never called
  S3  LOW     MD011 second narrowing undocumented: (a)[b](c)[d] reported by 0.41.1, silent on 0.42.0
  S4  LOW     RDONLY law: can fail only on a constant edit; passes on 1.1.5; f_flag pass-through unpinned; name overclaims
  S5  LOW     "merge gate" not enforced: no required status checks on main
  S6  NIT     "#176 closes" has no closing reference
  S7  NIT     rustix 1.1.4 still compiled in CI through cargo-fuzz; predates this PR
behaviour defects in Keep: 0
attacks failed:  11 areas, itemized above
untested:        Apple Silicon cargo check; Baker Street history claims
pending:         Rust quality gates at de515f3 in progress at last check (run 41665161676)
                 documentation and dependency-policy jobs and Runtime fuzz smoke passed
blocking:        none for behaviour; S1 should be reworded before this record is relied on
resilience grade: B+
```
