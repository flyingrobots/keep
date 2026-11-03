# Testimony of S. Holmes: code verification pass, flyingrobots/keep#211

**Scope.** `git diff e316d71..ce998c8` in `/Users/j/git/keep-integrate`; the worktree HEAD is `ce998c8` and clean. Unless a citation says otherwise, line numbers are at `@ce998c8`.

**Limits of the evidence.**
- Nothing was compiled or run, as instructed. Every "mutant survives" statement is deduction from source, not an observed run.
- PR bodies, issue state and CI state come from `gh` and are **hearsay** (unpinned).
- At my reading, CI on `ce998c8` had Rust quality gates and fuzz smoke pending (hearsay). No pass is claimed here.

**The short of it.** The recovery code does what the new paragraphs say, and I found no outcome in the two amended errors that reports the opposite of what occurred. The drift lies in the prose around them:
- The "recovery directory-sync errors" claim is wider than the code that was changed.
- Docs promising "no claim" sit beside wording that makes one.
- A ledger row says "Every … Implemented" over laws whose own headers say "Selected".

---

## I. Recovery outcomes after a directory-sync failure

**What holds.**

- **The outcome is captured before the sync and carried through the error.**
  - `src/adapters/recovery/recovery_stage_discard_executor.rs#25-34@ce998c8`
  - `src/adapters/recovery/recovery_next_head_finalization_executor.rs#37-63@ce998c8`
- **The labels are true.**
  - `Removed` is returned only after `remove_file` succeeds: `src/adapters/filesystem_recovery_stage_discard_storage.rs#124-127@ce998c8`.
  - `AlreadyAbsent` is returned only on an observed `NotFound` with namespaces reverified: same file `#104-108`, `#130-138`.
  - `Finalized` is assigned only after `replace_head` returns `Ok`: `recovery_next_head_finalization_executor.rs#45-51@ce998c8`.
  - `AlreadyFinalized` requires a current match and `head.next` absent: `src/adapters/filesystem_recovery_next_head_finalization_storage.rs#31-37,#85-86,#124-136@ce998c8`.
  - Ready takes priority, and the planner requires an exact successor, so the two cannot be confused in lawful use: `src/adapters/recovery/recovery_next_head_finalization_planner.rs#71-94@ce998c8`.
- **A failed sync yields no receipt.** The `?` precedes the receipt constructor: `recovery_next_head_finalization_executor.rs#57-64@ce998c8`, `recovery_stage_discard_executor.rs#28-35@ce998c8`.
- **The source chain survives.** `src/adapters/recovery/recovery_stage_discard_error.rs#47-54@ce998c8`, `src/adapters/recovery/recovery_next_head_finalization_error.rs#79-88@ce998c8`.
- **The laws exercise real filesystem effects with injected EIO, for both outcomes and both operations.**
  - `src/adapters/filesystem_recovery_stage_discard_tests/sync_failure_laws.rs#15-23,#40-60,#83-85@ce998c8`
  - `src/adapters/filesystem_recovery_next_head_finalization_tests/sync_failure_laws.rs#16-24,#38-64,#102-104@ce998c8`
- **Retry re-observes.** `src/adapters/filesystem_recovery_stage_discard_storage.rs#100-123@ce998c8`, `src/adapters/filesystem_recovery_next_head_finalization_storage.rs#17-40@ce998c8`.

**Limits, honestly stated by the code.** The law headers disclaim kernel and power-loss faults: `.../sync_failure_laws.rs#3@ce998c8` in both files. "Removal is not rolled back" (`docs/formats/segment-store-v1/recovery.md#299@ce998c8`) holds for the failed call only, and the same sentence goes on to say durability is unconfirmed. Discard injection covers only `RecoveryStage::NextHead`, the mock ignores the parent argument, and the real-filesystem law never exercises `staging`: `src/adapters/filesystem_recovery_stage_discard_tests/sync_failure_laws.rs#27,#83@ce998c8`.

**Drift.**

**D1 (worst): the claim is wider than the fix, and the defect survives one module away.**
- CHANGELOG says "recovery directory-sync errors" (`CHANGELOG.md#17@ce998c8`). #190's invariant says "Recovery errors preserve … the known effects" (hearsay, PR body). #211's says "recovery directory-sync errors" (hearsay).
- `RecoveryStageCompletionError::SynchronizeStaging` has no outcome field: `src/adapters/recovery/recovery_stage_completion_error.rs#49-54@ce998c8`.
- The executor binds `stage_outcome` and then omits it from the error: `src/adapters/recovery/recovery_stage_completion_executor.rs#49-54@ce998c8`.
- The removal runs through the same function that can return `AlreadyAbsent`: `src/adapters/filesystem_recovery_stage_completion_storage.rs#53@ce998c8`.
- Its message is unconditional: "failed to synchronize staging after {stage} removal" (`recovery_stage_completion_error.rs#82-85@ce998c8`).
- On a retry that found the name absent, that message attributes an earlier invocation's work to this one. The new rationale condemns exactly this: "falsely attributes an earlier invocation's work to a retry" (`docs/formats/segment-store-v1/rationale.md#206@ce998c8`).
- `SynchronizePool` likewise drops `pool_outcome`: `recovery_stage_completion_executor.rs#34,#46-48@ce998c8`.
- This code predates the range. The drift is in the scope of the new claim, not in new code. Either narrow the prose to "discard and next-head finalization", or fix the sibling.
- I did not survey every other recovery path outside these modules.

**D2: "makes no claim" sits beside text that makes one.** The executors promise "A removal error makes no claim that the capability had no effects" and the replacement equivalent: `recovery_stage_discard_executor.rs#19@ce998c8`, `recovery_next_head_finalization_executor.rs#25@ce998c8`. `recovery.md#299,#325@ce998c8` says the same. Against that:
- A `remove_file` I/O failure maps into `Remove` (`filesystem_recovery_stage_discard_storage.rs#124-126@ce998c8`) and prints "stage discard **was refused**" (`recovery_stage_discard_error.rs#32@ce998c8`).
- `Replace` is documented as "could not atomically replace" and "could not be finalized", and prints "failed to finalize": `recovery_next_head_finalization_error.rs#29-32,#61-65@ce998c8`.
- `SynchronizeCandidate` says "was not made durable": same file `#24@ce998c8`.

This is the one place an error can assert no effect when an effect occurred. A rename or unlink that returns EIO after taking effect is **conjecture** as to real-world occurrence, but it is the very case the docs reserve.

**D3: "Durable" survives where #190 removed it.** The outcome enum was rewritten to "Namespace outcome" (`src/adapters/recovery/recovery_next_head_finalization_outcome.rs#3-6@ce998c8`). The readiness enum still says "Durable `HEAD` already names the exact candidate" (`src/adapters/recovery/recovery_next_head_finalization_readiness.rs#6-8@ce998c8`). The port still says "Revalidates durable current state" (`src/adapters/recovery/recovery_next_head_finalization_storage.rs#18@ce998c8`). After a failed root sync, `verify_current` observes the name only and cannot know its durability.

**Unverifiable: a retry's receipt after a real failed sync.**
- The law asserts that a retry after failure returns a receipt: `.../filesystem_recovery_next_head_finalization_tests/sync_failure_laws.rs#65-71@ce998c8`.
- The receipt is documented as "Proof that durable `HEAD` names one exact candidate": `src/adapters/recovery/recovery_next_head_finalization_receipt.rs#8@ce998c8`.
- The retry syncs through a freshly opened descriptor: `src/adapters/filesystem_catalog_artifact.rs#27-31@ce998c8`.
- *Conjecture:* on some kernels and filesystems, a later fsync on a new descriptor may succeed without proving the earlier dirty state reached disk. The injected fault cannot test this, and neither can I.
- This is the highest-risk open item against "represent it as uncertainty and refuse".

---

## II. Do the mutation laws test what KEEP-RETENTION-003 claims?

**Rows confirmed.**
- I walked every table row against the decoders. Offsets and expected values match:
  - `src/adapters/retention/head_decoder.rs#25-30,#36-60@ce998c8`
  - `src/adapters/retention/manifest_header_decoder.rs#31-45,#52-70@ce998c8`
  - `src/adapters/retention/manifest_entry_decoder.rs#78-81,#106-111@ce998c8`
  - `src/adapters/retention/root_header_decoder.rs#51-62,#69-87@ce998c8`
- Seal selection is right. Each digest row reseals only what lies downstream of it, in decoder order:
  - `tests/retention_root_decoding/mutation_laws.rs#270-275,#301-312,#420-454@ce998c8`
  - `tests/retention_manifest_codec/mutation_laws.rs#167-172,#200-211,#286-317@ce998c8`
  - `src/adapters/retention/root_decoder.rs#13-39@ce998c8`
  - `src/adapters/retention/manifest_decoder.rs#13-23@ce998c8`
- The tightened namespace causes are exact and match their source: `tests/retention_root_decoding/mutation_laws.rs#338-354@ce998c8`, `src/retention/namespace.rs#49-53@ce998c8`.

So #194's "the row proves that field's check and nothing upstream of it" (hearsay, PR body) is true of the rows. The production decoders also refuse as the ledger requires. The drift is in what the *evidence* is said to prove.

**D4: "each law" (`CHANGELOG.md#21@ce998c8`).**
- `reframe` laws change several fields at once: `tests/retention_root_decoding/mutation_laws.rs#396-418@ce998c8`, `tests/retention_manifest_codec/mutation_laws.rs#270-284@ce998c8`.
- One law in the head file asserts *admission*, not refusal: `tests/retention_head_codec/mutation_laws.rs#173-187@ce998c8`.

**D5: "exact named cause" (#211, hearsay).** #211 tightened namespace because it "accepted any `Namespace` refusal". The same looseness remains in other rows:
- `BlobId { index: 0, .. }` and `LayoutId { index: 0, .. }` accept any inner cause: `tests/retention_root_decoding/mutation_laws.rs#293,#299@ce998c8`. Those inner enums have several causes: `src/adapters/blob_id_binary_error.rs#10-29@ce998c8`, `src/adapters/layout_id_binary_error.rs#10-36@ce998c8`.
- `Semantic { .. }` rows and digest rows do not assert expected or observed values: `tests/retention_root_decoding/mutation_laws.rs#113-115,#274,#305,#311@ce998c8`.

The module header is candid about this ("variant-level oracles", `tests/retention_root_decoding/mutation_laws.rs#3@ce998c8`). The prose is not.

**D6: "Every … Implemented for whole-record decoding" (`docs/formats/segment-store-v2/requirements.md#14@ce998c8`).** The law files themselves say "**Selected** malformed … fields" (`#3` of all three). Law by law:

| Claimed element | Evidence | Finding |
| --- | --- | --- |
| Ordering law | Whole-record laws feed only *equal* neighbours: `tests/retention_root_decoding/mutation_laws.rs#362-368@ce998c8`. Prefix tests stop at length 118 and take a separate path: `src/adapters/retention/filesystem_retention_anchor_order_prefix_tests.rs#86-88@ce998c8`, `src/adapters/retention/root_anchor_decoder.rs#48-54@ce998c8`. | Deduction: mutating `observed <= prior` to `==` at `root_anchor_decoder.rs#73@ce998c8` survives. `RetentionRoot::new` sorts its anchors and refuses only duplicates (`src/retention/root.rs#56,#126@ce998c8`), so nothing downstream would catch a complete descending record. Manifest is covered, since its partial path shares `admit_order`: `src/adapters/retention/manifest_entry_decoder.rs#103,#114-122@ce998c8`. |
| Reserved byte | One byte per region: `tests/retention_root_decoding/mutation_laws.rs#224,#279@ce998c8`, `tests/retention_manifest_codec/mutation_laws.rs#137,#176@ce998c8`, `tests/retention_head_codec/mutation_laws.rs#140@ce998c8`. | The code checks every byte (`root_header_decoder.rs#86-87@ce998c8`, `head_decoder.rs#58-61@ce998c8`). A narrowed check survives, e.g. `require_zero(42, 1)` against `manifest_header_decoder.rs#69@ce998c8`. |
| Truncation boundary | Whole-record tests cover `len-1` only: `tests/retention_root_decoding.rs#48-56@ce998c8`, `tests/retention_manifest_codec/refusal_laws.rs#19-27@ce998c8`. | The below-header boundary (`root_header_decoder.rs#37@ce998c8`) is not exercised through `decode`. The prefix laws use a different API: `tests/retention_stage_prefix.rs#136-150@ce998c8`. |
| Overflow | `LengthOverflow` is never asserted in the retention decode law files. | Deduction: unreachable on 64-bit (`root_header_decoder.rs#102-110@ce998c8`). The refusal exists but is unproved. |
| Structural field | The head manifest digest is deliberately admitted; manifest entry digests and the anchor digest have no field refusal beyond set integrity. | "Every" is literally false for opaque fields, and the ledger does not say so. |
| Flag | Bit 0 only: `tests/retention_root_decoding/mutation_laws.rs#76@ce998c8`. | The code is `!= 0`; low risk. |
| Limits and bounds | No row for depth zero or nodes/encoded/physical above maximum. Head length above bound is absent from this matrix. | Covered elsewhere at prefix level: `src/adapters/retention/filesystem_retention_short_head_tests.rs#20@ce998c8`. |
| Fuzz target | Asserts no panic and round-trip only: `fuzz/fuzz_targets/retention_format.rs#19-38@ce998c8`. Seeded: `xtask/src/fuzz_seed_corpus/retention_seeds.rs#21@ce998c8`. | Gives no evidence of refusal *precision*. |

**D7: test names claim an ordering they cannot discriminate.** `namespace_length_bounds_refuse_after_complete_integrity` and `anchor_order_and_count_ceilings_refuse_after_complete_integrity` feed only fully sealed records: `tests/retention_root_decoding/mutation_laws.rs#334,#359,#416@ce998c8`, `tests/retention_manifest_codec/mutation_laws.rs#233,#282@ce998c8`. Moving those checks before integrity would pass them. The code does order them after integrity (`root_decoder.rs#14,#36-37@ce998c8`, `manifest_decoder.rs#14,#22@ce998c8`), but these laws do not establish it.

**D8: reverse drift in the ledger.** #194 calls citing the closed #19 a defect (hearsay; `gh` shows #19 CLOSED). `KEEP-RETENTION-006` still reads "In progress in #19": `docs/formats/segment-store-v2/requirements.md#17@ce998c8`.

---

## III. Reverse drift (code or tests more honest or stronger than prose)

- The code checks every reserved byte and every flag bit; the laws prove one of each (D6).
- Law headers say "Selected" and "variant-level"; the ledger and CHANGELOG say "per-field", "every" and "exact".
- The landing evidence dropped in salvage stated "not power-loss persistence" plainly: `docs/testing-evidence/m4-integration.md@66384de`, removed from the cherry-pick. The #190 PR body does not mention the removal (hearsay). The law header keeps the disclaimer, and the ledger notes the replacement: `docs/audits/pr-107-salvage.md#52@ce998c8`.

---

## IV. Verdict table

| # | Claim | Verdict | Evidence |
| --- | --- | --- | --- |
| 1 | Sync errors retain stage/target, I/O cause, outcome | CONFIRMED | §I |
| 2 | `Removed` / `Finalized` mean this invocation performed the effect | CONFIRMED | `filesystem_recovery_stage_discard_storage.rs#124-127@ce998c8`; `recovery_next_head_finalization_executor.rs#45-51@ce998c8` |
| 3 | `AlreadyAbsent` / `AlreadyFinalized` mean admitted without effect | CONFIRMED | `filesystem_recovery_stage_discard_storage.rs#104-108@ce998c8`; `filesystem_recovery_next_head_finalization_storage.rs#31-37@ce998c8` |
| 4 | A failed sync never yields a receipt or a rollback | CONFIRMED (in-process only) | `.../sync_failure_laws.rs#36-46@ce998c8` |
| 5 | Retry re-observes before proceeding | CONFIRMED | `filesystem_recovery_stage_discard_storage.rs#100-123@ce998c8` |
| 6 | No format, publication-order, or authority change | CONFIRMED | Diff touches only error fields, Display, docs, tests: `git diff e316d71..ce998c8 -- src@ce998c8` |
| 7 | Public additive field; exhaustive patterns must adapt | CONFIRMED | `src/lib.rs#115-116,#127-128@ce998c8`; `tests/recovery_stage_discard/execution_laws.rs#105-109@ce998c8` |
| 8 | #190: files byte-identical on main to the fix's parent | CONFIRMED | Five blob hashes equal across `66384de^`, `3887f2a`, `e316d71`, e.g. `src/adapters/recovery/recovery_stage_discard_error.rs` = `3c84022` |
| 9 | #190: cherry-picks with original authorship | CONFIRMED | Patch-IDs equal for two commits. Third differs only by dropped evidence files, with the same author date: `66384de`, `d42fa77` |
| 10 | #190: four sync laws, 7/7 discard tests | CONFIRMED by count; pass UNVERIFIABLE | `tests/recovery_stage_discard/execution_laws.rs@ce998c8` (4), `planning_laws.rs@ce998c8` (3) |
| 11 | #190 RED/GREEN and #194 RED, suite and xtask results | UNVERIFIABLE | Runtime receipts; hearsay. RED's source form is consistent: `src/adapters/filesystem_recovery_stage_discard_tests/sync_failure_laws.rs@5108cdd` |
| 12 | "Recovery directory-sync errors" retain outcomes | **DRIFT (D1, worst)** | `recovery_stage_completion_error.rs#49-54,#82-85@ce998c8` |
| 13 | Removal/replacement errors "make no claim" of no effect | **DRIFT (D2)** | `recovery_stage_discard_error.rs#32@ce998c8`; `recovery_next_head_finalization_error.rs#29-32@ce998c8` |
| 14 | Durability language corrected | **DRIFT (D3)** | `recovery_next_head_finalization_readiness.rs#8@ce998c8` |
| 15 | Retry receipt after a real sync failure proves durability | UNVERIFIABLE (conjecture of risk) | `recovery_next_head_finalization_receipt.rs#8@ce998c8`; `filesystem_catalog_artifact.rs#27-31@ce998c8` |
| 16 | Rows mutate, reseal correctly, name the refusal variant | CONFIRMED | §II |
| 17 | #194 touches no production code; helpers as listed | CONFIRMED | `git diff 5f0c518^1 5f0c518 -- src` is empty; `tests/support/byte_patches.rs#32-79@ce998c8` |
| 18 | "Each law" mutates one field and asserts refusal | **DRIFT (D4)** | `tests/retention_head_codec/mutation_laws.rs#173-187@ce998c8` |
| 19 | "Exact named cause" | **DRIFT (D5)** | `tests/retention_root_decoding/mutation_laws.rs#293,#299@ce998c8` |
| 20 | KEEP-RETENTION-003 "Every … Implemented" | **DRIFT (D6)** | §II table |
| 21 | "After complete integrity" law names | **DRIFT (D7)** | `tests/retention_root_decoding/mutation_laws.rs#334,#359@ce998c8` |
| 22 | The stale #19 citation was cured | **DRIFT (D8)** | `requirements.md#17@ce998c8`; closure is hearsay |
| 23 | `mutations.tsv` ledger deferred | PLANNED | `requirements.md#14@ce998c8`; `docs/audits/pr-107-salvage.md#75@ce998c8` |
| 24 | #211: PR heads merged unchanged; conflicts only in CHANGELOG; two assertions tightened | CONFIRMED | Second parents `d42fa77`, `7880d8a`; remerge-diff shows CHANGELOG only; `git show ce998c8`. `ce998c8` also adds one undeclared CHANGELOG blank line, which is harmless |

---

## VERDICT

```text
TARGET:        flyingrobots/keep#211, e316d71..ce998c8 (code section)
CLAIMS:        24 rows: 11 CONFIRMED, 7 DRIFT, 1 PLANNED, 2 UNVERIFIABLE,
               plus 3 partial or qualified (rows 4 and 10 CONFIRMED with limits;
               row 22 rests partly on hearsay)
DRIFT COUNT:   8 (D1-D8; D1-D3 in row 12-14, D4-D8 in row 18-22, with D4/D5/D6/D7/D8
               as distinct rows)
WORST DRIFT:   D1. "Recovery directory-sync errors" retain outcomes, but
               RecoveryStageCompletionError::SynchronizeStaging discards stage_outcome
               and always prints "after {stage} removal", including on AlreadyAbsent.
               src/adapters/recovery/recovery_stage_completion_error.rs#49-54,#82-85@ce998c8
               src/adapters/recovery/recovery_stage_completion_executor.rs#49-54@ce998c8
FALSE-EFFECT RISK:
  - Amended sync errors: none found. Outcomes are true by construction.
  - Claims effect that may not have occurred in this call: completion SynchronizeStaging (D1).
  - Implies no effect when one may have occurred: discard Remove "was refused"; Replace
    "could not" (D2).
KEEP-RETENTION-003:
  Decoders uphold the law. Laws prove selected fields, not "every". Named gaps: complete
  descending anchors (deduced surviving mutant), single-byte reserved checks, one truncation
  boundary, overflow unproved, opaque fields carry no refusal.
OPEN RISK:     Retry receipt after a genuine fsync failure: unverifiable, conjecture.
FIDELITY:      C+
  Core recovery behaviour and row mechanics are faithful; the prose overstates scope and
  completeness in both headline claims.
BASIS:         Static reading only. Nothing compiled or run. CI pending at reading (hearsay).
```
