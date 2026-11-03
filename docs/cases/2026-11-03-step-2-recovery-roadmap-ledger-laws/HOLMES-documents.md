# Testimony of S. Holmes: documents section, PR #211

**Scope.** I examined `ROADMAP.md`, `docs/audits/pr-107-salvage.md` and the README line at `ce998c8`, against diff base `e316d71`. The worktree was left clean: `git status --short` returns 0 lines. I ran no build or test, so every CONFIRMED below is textual or structural, and none is an executed proof.

**Date.** Main sits at `e316d71` (`git ls-remote`, today). It is not stale against the PR base, but it is 16 commits past the roadmap's stated snapshot, `3887f2a`.

## The case in brief

The roadmap's claims of completion hold up well. I found no feature marked Done whose code is absent.

The fault lies elsewhere: these documents were reconciled to a main that this very merge commit changes. The roadmap declares of T-16.5 that three files "exist only on the archived #107 branch", yet those files are delivered at the commit under examination.

## What I sampled, and what I did not

**Sampled mechanically, over the whole file**
- Feature and checklist counts, and anchors.
- Task ID sets against the branch.
- 148 cited paths, 160 cited identifiers, and every cited `KEEP-*` ID.
- All 54 branch commits and 19 landing-only commits against the ledger.
- Every ledger hash.

**Read closely**
- `ROADMAP.md` lines 1–760, 880–1300, 1488–1590, 1713–1732 and 2215–2300.
- All 102 lines of the ledger.

**Not sampled**
- The field bodies of F-26 to F-36 and F-39 to F-51. For these I checked only that the planned artefacts are absent.
- T-17.2 and T-17.3 field bodies, and F-12's "only file replaced in place" claim.
- Whether the tests pass, and whether the lints pass.
- Ledger rows 70–75 beyond their commit subjects, and the "lossy `read_exact_optional`" characterisation.

Issue and PR states come from the GitHub API and are **hearsay**.

## Structural claims

| Claim | Evidence | Verdict |
| --- | --- | --- |
| 51 features, 51 checklist rows, all anchors resolve | Mechanical comparison gives an empty difference | CONFIRMED |
| Task IDs unchanged from the branch | The 121-task ID sets from `ROADMAP.md@57b6c37` and `ROADMAP.md@ce998c8` are identical | CONFIRMED |
| Field text carried over verbatim | No `**field**` line was removed relative to `57b6c37` | CONFIRMED |
| Issues' T-IDs resolve | Sampled #110, #115, #116, #118, #119, #120, #126, #132 (hearsay): none unresolved | CONFIRMED |
| "Each unfinished task carries the full field set" | `ROADMAP.md#42@ce998c8`; see D5 | **DRIFT** |
| Status vocabulary and checkboxes are coherent | `ROADMAP.md#120@ce998c8`; see D6 | **DRIFT** |
| F-21 is named by all ten listed features | Each of the ten sections mentions F-21, #20 or T-21 | CONFIRMED |
| Unmatched cited paths are branch-only, planned, or bare basenames | 13 of the branch-only paths sampled all exist at `57b6c37` | CONFIRMED |

## Status claims (weighted toward "done")

| # | Claim (`ROADMAP.md`) | Evidence | Verdict |
| --- | --- | --- | --- |
| 1 | F-02: 59-byte form (#239) | `docs/adr/0001-exact-logical-byte-identity.md#107@ce998c8`. Document only; I did not locate a code constant | CONFIRMED |
| 2 | F-04: 16/64/256 KiB, 4 KiB state (#281–283) | `src/chunk/detector.rs#12@ce998c8`, `#14`, `#69` | CONFIRMED |
| 3 | F-05: 1,048,576 entries, 256 GiB (#306) | `docs/formats/flat-chunk-layout-v1/README.md#232@ce998c8`, `#242` | CONFIRMED |
| 4 | T-05.5: two rows only "Specified" (#318) | `docs/formats/flat-chunk-layout-v1/README.md#365@ce998c8`, `#367` | CONFIRMED |
| 5 | T-06.1: 216 sequences (#341) | `tests/streaming_cas/model_laws.rs#16@ce998c8`, `#30` (6³) | CONFIRMED |
| 6 | T-06.3: named laws (#347) | `src/reference/reconstruction_tests.rs#49@ce998c8`, `src/reference/range_read_tests.rs#163@ce998c8` | CONFIRMED |
| 7 | T-06.4: scratch limit (#382) | `src/reference/ingestion.rs#43@ce998c8`, `tests/streaming_cas_memory.rs#87@ce998c8` | CONFIRMED |
| 8 | "Defects #71 and #74 closed" (#331) | #71 is OPEN (hearsay). The code is fixed; the tracker disagrees | **DRIFT** (D9) |
| 9 | F-09: "One baseline exists" (#471) | Two files in `benchmark/baselines/` at `ce998c8` | **DRIFT** (D4) |
| 10 | F-09: 13 scenarios, 16 MiB (#468) | `docs/benchmarks/streaming-cas-baseline-v1/README.md#99@ce998c8`, `#80` | CONFIRMED |
| 11 | T-10.2: lint and no Python (#536) | `Cargo.toml#44@ce998c8`; zero tracked `.py` files | CONFIRMED |
| 12 | F-10: port "names seventeen" (#531) | `src/adapters/retention/publication_storage.rs#17@ce998c8`. 18 methods: `verify_current` plus 17 phase capabilities | CONFIRMED |
| 13 | F-08: checkers import no production code (#444) | `keep::` is imported only by `durability_crash_matrix` and `fuzz_seed_corpus`, neither of them a corpus checker | CONFIRMED |
| 14 | F-08: eight semantic laws (#442) | `docs/conformance/golden-file-worldline.md#12@ce998c8` and the eight items after it | CONFIRMED |
| 15 | F-11: 64/112/128 byte records (#545) | `docs/formats/segment-store-v1/segment.md#44@ce998c8`, `#78`, `#144` | CONFIRMED |
| 16 | F-13: 105 cases (#587) | `docs/formats/segment-store-v1/recovery.md#130@ce998c8` | CONFIRMED |
| 17 | T-01.3: row "Specified in #14" (#231) | `docs/formats/segment-store-v1/requirements.md#27@ce998c8` | CONFIRMED |
| 18 | F-14: absence law (#606) | `xtask/tests/segment_store_implementation_documentation.rs#43@ce998c8`, `#49` | CONFIRMED |
| 19 | F-16: record sizes (#676) | `docs/formats/segment-store-v2/retention.md#99@ce998c8`, `#101`, `#178`, `#179`, `#228`; `definition.tsv` rows | CONFIRMED |
| 20 | T-16.1: both digests (#686) | `conformance/segment-store/v2/README.md#42@ce998c8`, `conformance/segment-store/v2/README.md#49@57b6c37` | CONFIRMED |
| 21 | **F-16 / T-16.5 "not on main"** (#108, #672, #695–701) | See D1 | **DRIFT** (reverse) |
| 22 | F-17: 21 phases; ledger statuses (#734, #741) | `src/adapters/store_migration/migration_phase.rs#57@ce998c8`; `docs/formats/segment-store-v2/requirements.md#31@ce998c8` through `#38` | CONFIRMED |
| 23 | F-17: direct v2 initialization undefined (#745) | `docs/formats/segment-store-v2/recovery.md#205@ce998c8`. True now; see Horizon | CONFIRMED |
| 24 | T-18.0: 17 phases (#905) | `src/adapters/retention/publication_phase.rs#49@ce998c8` | CONFIRMED |
| 25 | F-18: ledger quotes for `-006`, `-007` (#885) | `docs/formats/segment-store-v2/requirements.md#17@ce998c8`, `#18` | CONFIRMED |
| 26 | T-18.2: `transitions.tsv` lists only 053–073 (#949) | First and last data rows are 053 and 073 at `ce998c8`; `README.md#55@ce998c8` claims 156 coordinates, so the open box is the honest reading | CONFIRMED |
| 27 | T-18.3: live re-verification and exact downcasts (#970) | `docs/formats/segment-store-v2/closure.md#3@ce998c8`; `src/adapters/retention/filesystem_retention_publication_closure_tests.rs#61@ce998c8`, `#66` | CONFIRMED |
| 28 | T-18.3: an equivalent corrupt-record law is "Unverified on main" (#979) | My search of the retention tests found none; a search is not proof of absence | UNVERIFIABLE |
| 29 | F-19: `KEEP-RETENTION-008` Implemented (#1004) | `docs/formats/segment-store-v2/requirements.md#19@ce998c8` | CONFIRMED |
| 30 | T-20.1: 343 histories (#1068) | `docs/formats/segment-store-v2/requirements.md#21@ce998c8` | CONFIRMED |
| 31 | F-21: `SnapshotBinding` unsupported; capability `declared-future` (#1099, #1163) | `docs/invariants/verification/README.md#32@ce998c8`; `conformance/golden-file-worldline/v1/capabilities.tsv#14@ce998c8` | CONFIRMED |
| 32 | T-21.1: `tests/verification_report.rs` branch-only (#1114) | Absent at `ce998c8`, present at `57b6c37` | CONFIRMED |
| 33 | T-21.2: 105 and 150 rows, capability `required` on branch (#1166, #1171) | Row counts at `57b6c37`; `conformance/golden-file-worldline/v1/capabilities.tsv#14@57b6c37` | CONFIRMED |
| 34 | F-22: nothing on main; GC artefacts refuse (#1242–1245) | `docs/formats/segment-store-v2/requirements.md#48@ce998c8`, `#49`; `docs/formats/segment-store-v2/recovery.md#219@ce998c8`; no `src/adapters/gc/` | CONFIRMED |
| 35 | **T-22.1a: "four rows", digest `6cbc1c75…`** (#1274–1276) | See D2 | **DRIFT** |
| 36 | T-23.1: durable surface, fixture through v1, migrate, retain (#1513) | `tests/golden_file_worldline/durable_fixture.rs#73@ce998c8`; `README.md#200@ce998c8`; authenticated-reconstruction `requirements.md#17@ce998c8`, `#18` | CONFIRMED |
| 37 | F-23: persisted receipts separate (#1493) | `docs/invariants/authenticated-reconstruction/requirements.md#14@ce998c8` | CONFIRMED |
| 38 | F-24: crate doc says unimplemented (#1564) | `src/lib.rs#58@ce998c8` | CONFIRMED |
| 39 | F-25: #22 and #23 closed 2026-08-15; PR #77 merged (#1715, #1725) | Hearsay | CONFIRMED |
| 40 | **F-37: markdownlint-cli2 0.23.3** (#2222) | See D3 | **DRIFT** |
| 41 | F-37: lychee 0.21.0, actionlint 1.7.12 | `CONTRIBUTING.md#34@ce998c8`, `#35` | CONFIRMED |
| 42 | F-37: actions pinned; no persisted credentials | Zero unpinned `uses:` lines; 5 of 5 checkouts set `persist-credentials: false` | CONFIRMED |
| 43 | F-37: command surface | `xtask/src/main.rs#113@ce998c8`; `xtask/src/fuzz_campaign.rs#134@ce998c8`, `#135` | CONFIRMED |
| 44 | F-37: Miri, mutation and API-diff gates not automated (#2238) | None found in `.github` or `xtask/src`. A reverse-drift search that came back empty | CONFIRMED |
| 45 | F-38: `migration-inventory.md` still lags (#2248) | `docs/formats/segment-store-v2/migration-inventory.md#66@ce998c8`; `docs/formats/segment-store-v2/requirements.md#33@ce998c8` | CONFIRMED |
| 46 | T-38.1: gap rows name live issues | `README.md#104@ce998c8`, `#105`, `#106` (#125, #21, #82 open, hearsay); `#86` also open | CONFIRMED |
| 47 | F-43: `0.0.0`, `publish = false`; F-47: `pure`, `std` | `Cargo.toml#3@ce998c8`, `#13`, `#20` | CONFIRMED |
| 48 | F-42, F-45, F-46, F-48: `keep-cli/`, `formal/`, `mutants-exclusions.toml`, `diff_layouts` | All absent, as a plan requires | PLANNED |
| 49 | Snapshot "main at 3887f2a, 2026-10-26" (#24) | Main sat at `3887f2a` from 2026-10-20 until today's #208 (hearsay). True as dated; 16 commits stale | CONFIRMED |

## Ledger verdicts

| Row | Claim (`docs/audits/pr-107-salvage.md`) | Evidence | Verdict |
| --- | --- | --- | --- |
| #3–5 | Closed 2026-10-20 unmerged; 93 conflicting files | `closedAt` 2026-10-20 (hearsay); a trial merge of `3887f2a` and `57b6c37`, in a scratch object directory, lists exactly 93 | CONFIRMED |
| #17–21 | Heads `57b6c37`, `f21589f`; base `c9277ea` (2026-09-09); 54 and 360 commits | `git ls-remote` and `git rev-list` | CONFIRMED |
| Whole page | Every commit has a verdict | 54 of 54 branch and 19 of 19 landing-only commits are named; all 74 hashes resolve. `5a23238`, `9a10b22` and `ee4ef54` carry split verdicts, unlike #197's "one verdict" | CONFIRMED |
| #35–43 | Superseded rows | Commit subjects match the outcomes; the named PRs merged (hearsay), except row 42 | CONFIRMED |
| #42 | "Delivered on main by #130, #136" | #130 is the issue; the PR is #163 (hearsay; `ROADMAP.md#2245@ce998c8` has it right) | **DRIFT** (D13) |
| #49 | `contains_blob` fallible; `DurableSnapshot` read half | `src/store/reads.rs#38@88db212`, `src/adapters/durable/port.rs#14@88db212` | CONFIRMED |
| #50 | `src/pipeline`; shared `ChunkReader`; stacked on the port | `src/authenticated_read/chunk_reader.rs#16@ccc618f`; #193's base is the port branch (hearsay) | CONFIRMED |
| #51 | Mutation matrices; `byte_patches` helpers | `tests/support/byte_patches.rs` in the `e316d71..ce998c8` diff | CONFIRMED |
| #52 | Production files byte-identical to the fix's parent | 6 of 6 production paths identical between `66384de^` and `3887f2a`; the 3 that differ are test files | CONFIRMED |
| #52 | "so the defect is live on main" | False once #190 lands in this same PR | **DRIFT** (D11) |
| #53 | Strips `decode_unbound`, `from_fields`, `candidate_set_digest`; no digest change; `gc.md` lacks the count sentence | First two absent at `08547f2`; the encoder function added by `9a10b22` is absent, though the identifier survives through `e4fb098`'s accessors and `intent_format`. `definition.tsv` unchanged | CONFIRMED |
| #54 | Five sites remain; source-text test gone | All five files exist at `2132cc6`; no `tests/typed_refusal_source_contract.rs` | CONFIRMED |
| #61 | Main lacks `record_segments` | No match at `ce998c8` | CONFIRMED |
| #61 | Main lacks "retained-closure member verification" | See D12 | **DRIFT** |
| #62 | No exclusive fence API; `empty gc/`; `.next` names against the rule | Exclusive reader-lock `flock` appears only in tests (`src/adapters/retention/filesystem_retention_snapshot_tests.rs#121@ce998c8`); `src/adapters/filesystem_initialization_namespace.rs#123@ce998c8`; `docs/formats/segment-store-v2/gc.md#19@ce998c8`, `#20`; `docs/formats/segment-store-v2/gc-execution.md#44@57b6c37`. The cell "typed `RetentionStorageError`" is elliptical: the type exists on main (`src/adapters/retention.rs#285@ce998c8`) and not on the branch | CONFIRMED |
| #63 | Seven rows; `32381f1a…` to `a4a010ce…` | `git diff 3887f2a 57b6c37 -- conformance/segment-store/v2/definition.tsv` shows +7 | CONFIRMED |
| #65 | Compaction needs GC planning | `src/adapters/compaction/filesystem.rs#292@57b6c37` | CONFIRMED |
| #66 | `open_version_two`, `close_sealed`, `select_closed` absent from main | `src/adapters/filesystem_catalog_publisher.rs#85@57b6c37`, `#186`, `#207`; zero matches in the publisher at `ce998c8` | CONFIRMED |
| #66 | "#129 is moot" | #129 is still open (hearsay); not traced | UNVERIFIABLE |
| #67 | 384-byte record | Located nowhere in my search | UNVERIFIABLE |
| #67 | #165 excludes serialization | `docs/audits/114-durable-verification-scope.md#79@ce998c8` | CONFIRMED |
| #69 | 99 + 51 rows | Records at `57b6c37`: gc-intent 19, gc-receipt 17, disposition 15 = 51; the rest = 99 | CONFIRMED |
| #82 | `34db571` among "Re-integration merges" | See D14 | **DRIFT** |
| #83, #86 | Newer dependency admissions; `KEEP-VERIFY-006` regressed on branch | `docs/dependencies/` has `blake3-1.8.7`, `cap-std-4.0.3…`, `yaml-rust2-0.13.0`; the branch's row reads Planned in #20 | CONFIRMED |
| #84 | Durable-store docs dropped | Named README and rationale exist at `57b6c37`, but so does `docs/architecture/durable-store/ingestion.md`, which the row does not cover | CONFIRMED (omission noted) |
| #85 | Landing logs on the audit branch | 85 files at `001ae2a` | CONFIRMED |
| #101 | Ported branches independent except pipeline | #190–#192, #194, #195 base on main (hearsay) | CONFIRMED |

## The drifts

**D1 (worst): T-16.5 and F-16.**
- `ROADMAP.md#696@ce998c8` says "Planned on `main`".
- `ROADMAP.md#699@ce998c8` says the modules "exist only on the archived #107 branch".
- `ROADMAP.md#108@ce998c8` and `#672@ce998c8` repeat this, and `#673` quotes the ledger as "In progress in #19".
- All three `mutation_laws.rs` modules exist at `ce998c8`.
- `docs/formats/segment-store-v2/requirements.md#14@ce998c8` reads "Implemented for whole-record decoding".
- #196's own body says T-16.5 "should be checked" when this PR merges (hearsay).
- This is reverse drift. The acceptance clause "a deliberately weakened decoder fails" (`ROADMAP.md#706@ce998c8`) I did not test, so the correct wording may be "Implemented for whole-record decoding", as the ledger has it.

**D2: T-22.1a's row count and digest.**
- `ROADMAP.md#1274@ce998c8` says "four `definition.tsv` rows"; `#1276` says digest `6cbc1c75…`.
- The branch adds seven rows, and its digest is `a4a010ce…` (`conformance/segment-store/v2/README.md#49@57b6c37`).
- The roadmap contradicts itself at `ROADMAP.md#686@ce998c8` and contradicts the ledger at `docs/audits/pr-107-salvage.md#63@ce998c8`. The ledger is correct.
- Carried over from `ROADMAP.md#1203@57b6c37`.

**D3: F-37's tool version.** `ROADMAP.md#2222@ce998c8` says 0.23.3. `CONTRIBUTING.md#34@ce998c8` and `scripts/documentation-tools/package.json#5@ce998c8` say 0.24.0. This is plain staleness, caused by #188 in step 1.

**D4: F-09.** "One baseline exists" (`ROADMAP.md#471@ce998c8`). `benchmark/baselines/30ffe90-aarch64-apple-darwin.tsv` has existed since `6a95b5f` (2026-10-01), so this was false even at the snapshot.

**D5: Field set.** `ROADMAP.md#42@ce998c8` claims every unfinished task has one.
- T-22.1 ends in "Original task fields:" with nothing after it (`ROADMAP.md#1272@ce998c8`), and T-22.1a then says "as T-22.1" (`#1294`).
- The fields existed earlier (24 lines at `68b81ee`) and were lost by `e4fb098`; #195 implements this very task.
- Six others have no labelled fields: T-01.2, T-03.3, T-05.5 (inline, unlabelled), T-25.2, T-41.4, T-45.3.

**D6: F-25.** It is checked (`ROADMAP.md#120@ce998c8`) with status "Partially implemented", a word outside the vocabulary table (`#28`–`38`), and T-25.2 is open (`#1726`). F-03, F-05 and F-09 are also checked with open tasks, but those at least disclose them.

**D7: Dependency map.** `ROADMAP.md#179@ce998c8` says "the F-06 defect #72". #72 is a P3 `[COOL IDEA™]` (hearsay) owned by T-24.3 inside F-24 (`#1669`). So F-24 "needs" itself, and it is not an F-06 defect.

**D8: "The same tag".** `ROADMAP.md#22@ce998c8` refers to a tag the page never names. `archive/pr-107-*` exists locally, but `git ls-remote origin 'refs/tags/*'` returns zero. The sentence is a leftover from an edit.

**D9: #71.** See row 8 of the status table.

**D10: "ADR-0010".** `ROADMAP.md#223@ce998c8` means the typed-refusals ADR, while `#1794` proposes `0010-git-cas-import-posture.md`. #199 takes `0010-one-storage-format.md` (hearsay). The #191 branch has already re-slugged the first as `docs/adr/preserve-typed-storage-refusals.md`. `AGENTS.md#165@ce998c8` forbids the bare number.

**D11: Ledger row 52.** See above.

**D12: Ledger row 61.** Main already has live member verification: `docs/formats/segment-store-v2/closure.md#3@ce998c8`, `src/adapters/retention/filesystem_retention_closure_admission.rs@ce998c8`, `tests/golden_file_worldline/durable_closure_refusal.rs@ce998c8`. `ROADMAP.md#970@ce998c8` says so too. *Conjecture:* the author meant the fenced-view shape in `src/adapters/gc/liveness_observation.rs#74@57b6c37`.

**D13: Ledger row 42.** See above.

**D14: Ledger row 82.** `34db571` has one parent and is a source fix touching `src/adapters/retention/verification_observation_error.rs`, not a merge. The verdict is plausibly sound: #191 addresses the same classification (its body, lines 31–32, hearsay). The stated reason is wrong.

**D15: Discoverability.** The charter says the README line "links them". `README.md#288@ce998c8` links only the roadmap. `ROADMAP.md` says "#107 salvage ledger" repeatedly (`#20` and others), with zero occurrences of the path. The sole citation on the target is `docs/formats/segment-store-v2/requirements.md#14@ce998c8`.

## Reverse drift

- D1 is the only true case: the roadmap understates what exists at the target.
- T-18.2's lagging ledger row is the ledger's lag, faithfully reported by the roadmap.
- My searches for unclaimed Miri, mutation, CLI, MCP, `formal/` and `diff_layouts` found nothing.

## Horizon (open, therefore not drift today)

All hearsay.

- #198, #205, #209 and #210 are uncited by the roadmap, as are #115–#124 and others, though the ledger names #120–#124 and #115.
- #203, "Direct initialization replaces migration", and #204, "Delete version 1", would falsify `ROADMAP.md#745@ce998c8`, F-17's framing and the v1 milestones. They would also falsify the reasoning of `docs/audits/pr-107-salvage.md#63@ce998c8` ("Stores migrated by either side").
- #200 and #201 would discharge the blockers at `docs/audits/pr-107-salvage.md#66@ce998c8`.
- #191, #192, #193 and #195 are open PRs described as "salvage in progress". The "In review" status (`ROADMAP.md#35@ce998c8`) is used by no feature.
- The ledger names no PR numbers and records no merged state.

## VERDICT

| Metric | Count |
| --- | --- |
| Status and ledger claims tested | 8 structural, 49 roadmap status, 26 ledger rows |
| CONFIRMED | 67 |
| DRIFT rows | 13 (2 structural, 5 roadmap status, 4 ledger, plus F-37 and T-22.1a counted above) |
| Distinct drifts | **15** (D1–D15) |
| PLANNED | 1 row, spanning F-42, F-45, F-46 and F-48 |
| UNVERIFIABLE | 3 |

**Claims of completion falsified by absent code:** 0. **Row-count caveat:** the tally is by table row; several rows bundle more than one check, and the 15 distinct drifts include D15 and D10, which are not table rows.

**Worst drift:** D1. At `ce998c8`, `ROADMAP.md#696`–`#700` say the T-16.5 modules exist only on the archived branch, while those modules and an "Implemented for whole-record decoding" ledger row land in the same commit (`docs/formats/segment-store-v2/requirements.md#14@ce998c8`). The roadmap's own rule, "the ledger wins" (`#12`), overrules it, and #196's body said it should flip on this merge.

**Fidelity grade: B.**
- Done-claims are sound, the ledger's accounting is complete to the commit, and every digest and row count I could recompute in the ledger holds.
- The grade is lost to staleness at the target itself (D1, D3, D11), one identity-adjacent wrong digest (D2), a missing field set on an in-flight task (D5), and an unlinked ledger (D15).

**Before merge, in order:**
1. Reconcile T-16.5 and F-16.
2. Correct T-22.1a.
3. Restore T-22.1's fields from `68b81ee`.
4. Update F-37 and F-09.
5. Rewrite ledger rows 42, 52, 61 and 82.
6. Link the ledger from the roadmap.

*Elementary, though not, I grant, brief.*
