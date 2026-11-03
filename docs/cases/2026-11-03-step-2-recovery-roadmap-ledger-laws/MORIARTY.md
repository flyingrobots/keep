# Testimony of Professor Moriarty: PR #211

How charming. Four merges, each a faithful little soldier, and a covering letter that promises rather more than the soldiers deliver. The merges and the new laws are sound. What collapses is every general claim written about them.

**Limits of this testimony.** Nothing was compiled or executed, as charged, so every mutant below is reasoned from source. I ran one trial `git merge-tree`, with objects kept in the scratchpad and deleted afterwards. Its output remains at `/private/tmp/claude-501/-Users-j-git-keep/0c5d0f19-e5f7-4a34-aac7-bdc48cc6cf8a/scratchpad/moriarty-211/merge.txt`. The worktree is still clean.

## Vector 1: #190, recovery outcomes

### S1, MEDIUM: the defect survives next door, and it misreports an effect
`execute_recovery_stage_completion` calls the same removal function as discard, then discards the outcome when the sync fails.
- **Lost outcome:** the error is built without `stage_outcome` at `src/adapters/recovery/recovery_stage_completion_executor.rs#49@ce998c8` through `#54@ce998c8`. The shared removal is at `src/adapters/filesystem_recovery_stage_completion_storage.rs#53@ce998c8`.
- **False message:** the Display text is unconditional: "failed to synchronize staging after {stage} removal" (`src/adapters/recovery/recovery_stage_completion_error.rs#84@ce998c8`).
- **Reachable path:** a retry does reach `AlreadyAbsent` (`tests/recovery_stage_completion/retry_laws.rs#100@ce998c8`). If that retry's staging sync fails, the error claims a removal this invocation never performed.
- **Self-condemnation:** #190's own rationale rejects this behaviour, since labelling every failure as a new mutation "falsely attributes an earlier invocation's work to a retry" (`docs/formats/segment-store-v1/rationale.md#206@ce998c8`). That paragraph sits under the unqualified heading "Recovery synchronization failures" (`#204@ce998c8`).
- **Same class:** `SynchronizePool` likewise drops `pool_outcome` (`recovery_stage_completion_executor.rs#34@ce998c8`, `#48@ce998c8`).

The real harm is limited. Retry still re-observes, so this is a false diagnostic and no bytes are at risk.

### S2, MEDIUM, CONJECTURE: a successful retry may be a false durability proof
- **The law:** after an injected EIO, it reopens and accepts a receipt (`src/adapters/filesystem_recovery_next_head_finalization_tests/sync_failure_laws.rs#66@ce998c8` through `#71@ce998c8`).
- **The receipt's claim:** "Proof that durable `HEAD` names one exact candidate" (`src/adapters/recovery/recovery_next_head_finalization_receipt.rs#8@ce998c8`).
- **The mechanism:** each sync opens a new descriptor (`src/adapters/filesystem_catalog_artifact.rs#27@ce998c8` through `#31@ce998c8`). On Linux, a descriptor opened after an error was reported may not see that error, so a clean retry need not prove the earlier directory update reached disk.
- **Scope:** the docs under `docs/formats/segment-store-v1` never discuss this. The retry protocol predates #190, but this PR now pins it in a law, and the Core Law asks that unprovable states be refused.
- **Experiment:** on Linux with dm-error or dm-flakey plus dm-log-writes, rename, fail the directory fsync, clear the fault, reopen, fsync successfully, cut power, and inspect `HEAD`. Run it on ext4 and XFS separately, because a journal abort may make the retry fail honestly.

### S3, LOW: no public law pins the new public field
Public integration tests elide `outcome` with `..` (`tests/recovery_stage_discard/execution_laws.rs#105@ce998c8`, `tests/recovery_next_head_finalization/execution_laws.rs#142@ce998c8`). Only in-crate tests assert it, although AGENTS.md lists public API integration tests as a required class.

### Attacks that failed
- **Executor outcomes:** `outcome` is exactly what `replace_head` or `remove_if_matching` returned, or the admitted readiness. Nothing reports success after a failed sync.
- **Fault location:** the injection replaces the real sync capability over real filesystem effects, and the file header discloses that limit.
- **Mutants:** a hard-coded outcome fails the typed and Display assertions. Syncing before the effect fails the namespace assertions, and swallowing the error fails `.ok_or`.
- **A small nuance:** the typed `matches!` assertion was added only at GREEN (`git diff 5108cdd d42fa77`). The Display assertion carried the RED, so nothing is left unproven.

## Vector 2: #194, mutation laws

### S4, MEDIUM: "exact named refusal" holds for 32 of 51 rows
By my count, 19 of 51 rows elide data-bearing fields: 8 of 24 for root, 6 of 16 for manifest and 5 of 11 for head.
- **Examples:** `tests/retention_root_decoding/mutation_laws.rs#293@ce998c8`, `#305@ce998c8` and `#311@ce998c8`; `tests/retention_head_codec/mutation_laws.rs#141@ce998c8`.
- **Surviving mutant:** a decoder that swaps `expected` and `observed` in any digest or checksum mismatch passes. The profile-digest row proves the author knows how to check this (`tests/retention_root_decoding/mutation_laws.rs#187@ce998c8`).
- **Contradiction:** the file header admits "Selected" fields and "variant-level oracles" (`tests/retention_root_decoding/mutation_laws.rs#3@ce998c8`). The ledger and changelog drop that qualification and say the laws assert "the exact named refusal" (`docs/formats/segment-store-v2/requirements.md#14@ce998c8`, `CHANGELOG.md#21@ce998c8`).
- **Partial fix:** ce998c8 fixed the two namespace cases only, though its own argument ("would have passed") applies to all 19.

### S5, MEDIUM: only one byte of each reserved field is ever tested
The decoders check 12, 48, 2, 2 and 8 reserved bytes (`src/adapters/retention/root_header_decoder.rs#86@ce998c8`, `#87@ce998c8`; `src/adapters/retention/manifest_header_decoder.rs#69@ce998c8`, `#70@ce998c8`; `src/adapters/retention/head_decoder.rs#58@ce998c8`). The laws flip one byte each: 98, 191, 42, 159 and 111.

A search of `tests`, `fuzz` and the retention test modules finds no other retention `NonZeroReserved` law. The fuzz oracle cannot help, because it asserts `root.encoded() == input` and `encoded()` simply returns the input (`fuzz/fuzz_targets/retention_format.rs#22@ce998c8`). Yet the requirement row promises every "reserved byte".

**Experiment:** change `require_zero(encoded, 180, 12, ...)` to `(191, 1)`. By inspection, no retention law should fail. #194's own RED weakened only the last head byte.

### S6, LOW: test names promise an ordering the tests cannot observe
`namespace_length_bounds_refuse_after_complete_integrity` and `anchor_order_and_count_ceilings_refuse_after_complete_integrity` feed only fully sealed records (`tests/retention_root_decoding/mutation_laws.rs#334@ce998c8`, `#359@ce998c8`, `#416@ce998c8`).

Moving namespace or count admission ahead of integrity leaves them green. The existing ordering laws cover generation and anchor identity, not these two (`tests/retention_root_decoding.rs`). AGENTS.md says test names describe laws, and these describe one they do not test.

**Experiment:** move `RetentionNamespace::try_from` above `root_integrity::verify` in `src/adapters/retention/root_decoder.rs#37@ce998c8`.

### S7, LOW: "Implemented" exceeds the roadmap's own acceptance criteria
- T-16.5 requires a case for every structural field, and that "a deliberately weakened decoder fails at least one case per field" (`ROADMAP.md#702@ce998c8` through `#707@ce998c8`).
- The namespace-length field at offset 40 has no solo row, and manifest entry digests at +0 and +40 have no row at all.
- #194's reported RED weakens two fields.

### Attack that failed: passing for the wrong reason
I walked all three decoders against the tables. Each row reaches the check it names. Fixed-field rows refuse before integrity, so their resealing does nothing, but it does no harm. Digest rows reseal the correct layers. Expected and observed lengths match the arithmetic in `src/adapters/retention/root_header_decoder.rs#113@ce998c8`. The two namespace patterns are type-correct against `src/retention/namespace_error.rs#12@ce998c8`. This attack is surrendered.

## Vector 3: the merges (itemized surrender, and it galls me)
1. **Parents:** the second parents d42fa77, f9553a5, f371c44 and 7880d8a equal the live PR heads, so "unchanged" is true.
2. **Conflicts:** `git show --remerge-diff` shows only `CHANGELOG.md` conflicted, and #197 had none.
3. **Changelog:** there are exactly three new entries and no duplicates, and all prior entries are retained. ce998c8's one extra line restores a dropped blank separator.
4. **Module declarations:** all five are present (two `sync_failure_laws`, three `mutation_laws`).
5. **Support exports:** the re-export covers every helper the laws import.

The one real merge flaw produces no textual conflict, so it appears as S8.

## Vector 4: ROADMAP.md and the ledger

### S8, MEDIUM: the PR lands a roadmap made false by its own fourth merge
- **Line 108:** the mutation matrix "is not on `main`" (`ROADMAP.md#108@ce998c8`).
- **Lines 673 and 674:** the ledger row "reads 'In progress in #19'" and the matrix "exists only on the archived #107 branch" (`ROADMAP.md#673@ce998c8`).
- **Lines 695 to 700:** the same three files this PR adds "exist only on the archived" branch (`ROADMAP.md#695@ce998c8`).
- **The claim above them:** "Every status claim has been reconciled to `main`" (`ROADMAP.md#17@ce998c8`), repeated at `CHANGELOG.md#19@ce998c8`. The roadmap's snapshot is 3887f2a (`ROADMAP.md#24@ce998c8`), but that is little comfort when the contradiction arrives in one PR.

The "ledger wins" clause (`ROADMAP.md#12@ce998c8`) limits the damage to confusion, which is why this stops at MEDIUM.

### S9, LOW: the ledger is stale on arrival and cannot say what landed
"The defect is live on main" (`docs/audits/pr-107-salvage.md#52@ce998c8`) is false once this merges. The ledger says delivery depends on the branch merging (`docs/audits/pr-107-salvage.md#8@ce998c8`), yet it has no per-row status. Four of the seven ported branches merge here (`git branch -r --merged ce998c8`), and nothing in the ledger says which.

### S10, LOW: an orphaned reference
"Archived at the same tag" (`ROADMAP.md#22@ce998c8`) points at nothing. Commit 6041923 replaced the tag citation with a branch and left this sentence behind.

### Attacks that failed
- Every commit hash in the ledger exists and is reachable from one of the two source branches.
- The heads 57b6c37 and f21589f match the live remote refs, and the merge base c9277ea is correct.
- The gc.md finding still stands.
- All Markdown links in `ROADMAP.md` resolve. Its references to absent paths are presented as branch-only or planned, and the audit file's absence is stated outright.

## Vector 5: the feature/198-delete-version-one stack

### S11, MEDIUM: an easy resolution silently reverts KEEP-RETENTION-003
A trial merge of ce998c8 with ee5d60a conflicts in three files: `CHANGELOG.md`, `tests/support/mod.rs`, and `docs/formats/segment-store-v2/requirements.md` as a modify/delete.

The stack's relocated row still reads "mutation coverage remains | In progress in #19" (`docs/formats/store/requirements.md#14@ee5d60a`), and that file does not conflict. Accepting the deletion, the natural choice in a delete-the-old-format stack, quietly downgrades the requirement while the laws remain. The stack's narrower re-export (`tests/support/mod.rs#19@ee5d60a`) at least fails closed at compile time if mishandled.

### S12, LOW: the roadmap's foundations move again
- Lines 7 and 8 of `ROADMAP.md` link pages the stack deletes. The documentation link check should catch that.
- F-17 "Done" describes a migration path that commit 25c98f8 replaces.
- The stack has no copy of `ROADMAP.md` or the ledger, so a rebase owes a second reconciliation.

### Attacks that failed
- #190's doc text auto-merges into `docs/formats/store/stage-recovery.md`, and the rationale into `catalog-protocol-rationale.md`.
- The fixture functions the sync laws call still exist. On the stack `discarder()` returns a boxed error, which `?` accepts, and the laws' `drop(storage)` before reopening suits writer-lock admission.
- All public names the laws import remain exported.
- Not compiled, as charged.

## Vector 6: the PR body

### S13, MEDIUM: both invariant claims overreach
- "Recovery directory-sync errors retain whether the removal... had already taken effect" is falsified by S1.
- "Retention root decoding refuses mutated records with the exact named cause" is overstated by S4, and it omits manifest and head altogether.

### S14, LOW: tense inflation
"The reviewed risks are..." and "See Review" point to a section that says only "review pending". The named risk, documents describing a moved `main`, is real, and S8 shows it still unresolved.

### S15, LOW: two softened statements
- "Had one open review finding": the thread on #194 is still `isResolved: false`.
- The body defers to #190's "Additive field" wording, which then admits exhaustive patterns must change. That is a source break, and `CHANGELOG.md#17@ce998c8` states it more honestly.

### Attacks that failed
- `main` has no required checks (`enabled: false`).
- #208 merged as e316d71.
- The repository allows merge commits only, so the promised merge-commit route is available.
- "No on-disk format change" holds on this diff.
- CI at last sight: documentation, fuzz smoke and dependency jobs passed, Rust quality gates were still running, and the body claims no green.

## Summary of strikes

| # | Severity | Strike |
|---|---|---|
| S1 | Medium | Stage completion loses outcomes and can claim a removal that did not happen |
| S2 | Medium (conjecture) | A retry after failed sync yields a "proof" receipt |
| S4 | Medium | 19 of 51 rows are not exact, against the ledger and changelog wording |
| S5 | Medium | One byte tested per reserved field; fuzz oracle is tautological |
| S8 | Medium | ROADMAP is false on arrival about #194 |
| S11 | Medium | Stack merge can silently revert KEEP-RETENTION-003 |
| S13 | Medium | PR invariant claims overreach |
| S3, S6, S7, S9, S10, S12, S14, S15 | Low | Public-law gap, name-only ordering, acceptance gap, stale ledger, orphan tag, roadmap churn, tense, softened statements |

Nothing here corrupts data or weakens production verification. The code changed by #190 is correct and its laws have teeth. The flaw throughout is prose that outruns the proof, which in foundational storage is merely the politest way to begin lying.

```text
VERDICT
target: flyingrobots/keep#211, e316d71..ce998c8
strikes:
  high: 0
  medium: 7 (S1, S2-conjecture, S4, S5, S8, S11, S13)
  low: 8 (S3, S6, S7, S9, S10, S12, S14, S15)
surrenders:
  vector_3_merges: complete (5 items)
  vector_2_wrong_reason: complete
  vector_1_fault_injection_and_masking: complete
attacks_failed:
  - executor outcome fidelity on discard and finalization
  - sync laws passing with the bug present
  - mutation rows refused by an earlier check than claimed
  - lost or duplicated CHANGELOG entries and module declarations
  - ledger hash reachability
  - textual and API breakage of sync laws on the stack (not compiled)
  - PR claims: unchanged heads, no required checks, merge-commit route
fix_before_merge:
  - S1: carry the outcomes in stage completion errors, or narrow the rationale and PR claims
  - S8 and S9: reconcile ROADMAP F-16/T-16.5 and ledger row 52 in this PR
  - S4, S5 and S13: tighten the laws or qualify requirements.md, CHANGELOG and the PR body
carry_forward:
  - S2: run the fault-injection experiment and record a decision
  - S11: carry the KEEP-RETENTION-003 row to docs/formats/store/requirements.md on rebase
not_executed: no cargo, no Docker; mutants reasoned from source
resilience_grade: B-
```
