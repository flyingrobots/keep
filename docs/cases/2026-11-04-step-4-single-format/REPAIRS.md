# Repairs after the verdict

- **Date:** 2026-11-05
- **Verdict repaired:** REQUIRES INVESTIGATION at `fb355d66` ([case file](CASE_FILE.md))
- **Repaired head:** `bea75ae9` on `integrate/open-prs`, PR #220, CI run 41999700415

This page records what was done about each item in the Repair Menu. It is the
author's own account. No detective has re-examined the repaired head, so nothing
here changes the verdict above, which stays valid only for `fb355d66`.

## Owner decisions

The owner accepted all four recommendations on 2026-11-05.

| Item | Decision |
| --- | --- |
| R1 | `FilesystemCatalogSnapshot::load` takes an expectation. |
| R2 | Caller-owned recovery is documented. A successor-accepting variant is a follow-up in #222. |
| R3 | `KEEP-CRASH-079` and `-080` follow version 1's deletion. #221 tracks them and blocks the first release. |
| R4 | The GC salvage renumbers from `KEEP-CRASH-092`. |

## Mutants (R5)

Run by hand, never committed. Full detail is in
`docs/testing-evidence/single-format-mutants.md` at the repaired head.

| Mutant, at `fb355d66` | Prediction | Result |
| --- | --- | --- |
| Commit synchronization is a no-op | Survives | Survived: 936 passed, and the 1 failure belonged to the next mutant |
| Gate re-entry removed from `replace_head` | Unknown | Killed by `changed_candidate_is_refused_at_the_replacement_boundary` |
| Lock replacement removed from the portability law | Survives | Survived |

Moriarty's conjecture about the commit synchronization is confirmed. The gate
mutant's death moves R15 from blocking to follow-up, as the menu provided.

A second round against the repaired tree killed all three of: the protocol not
requesting the synchronization, a shared root lock, and `load` ignoring its
expectation. The last also fails the retained-catalog crash sequence.

## Blocking repairs

| Item | Outcome | Commit |
| --- | --- | --- |
| R1, R6, R18 | `load` takes the expectation. `ExpectedCatalog::from_recorded` and `digest_bytes` make it recordable. `ObservedCatalog` is the observed side. Rustdoc has two compiled examples. | `0037255f` |
| R2 | "Recovering a stale expectation" in `recovery.md`. The retained-catalog crash sequence requires the recorded catalog to admit before `HEAD` moves and refuse after. | `0037255f`, `bea75ae9` |
| R7 | Typed refusals, guards that count only real mixes, renamed laws, narrowed claims. | `0694ebff`, `bea75ae9` |
| R13 | The Linux gate is removed. The three laws pass on both APFS variants. | `525071e3` |
| R14 | Proper form: an injected root synchronizer and two laws. | `7c03b179` |
| R15 | Not done. Follow-up in #222, because its mutant was killed. The docs say what holds the re-entry in place and what does not. | `bea75ae9` |
| R16 | The two `process_group` failures were a socket path over macOS's 104-byte bound, fixed. The other seven are classified by cause. | `a389a801`, `bea75ae9` |
| R8 to R12, R19 to R21 | Documentation corrected. | `bea75ae9` |
| R17 | PR body rewritten. | not a commit |

## Departures from the menu

- **R6's "first have someone confirm or refute the new finding".** Nobody
  compiled a refutation. The front office's static check stood, and the repair
  was made on that basis.
- **R18's rename to `PublishedCatalog`.** Not taken. `VerificationSubject`
  already has a `PublishedCatalog` variant, and a recorded claim is not a
  published fact. `ExpectedCatalog` now holds plain bytes and `ObservedCatalog`
  holds the verified side, which keeps the distinction R6 asked for.
- **R21's T-41.1.** Unticked. Neither ledger has a macOS row.
- **A contract law changed.** `xtask`'s protocol contract pinned the phrase
  "case-sensitive, byte-preserving directory names", which stopped being true
  when default APFS was admitted. The law now pins the rule that replaced it.

## Evidence at the repaired head

- Native macOS, Rust 1.96: 945 passed and 0 failed on default and on
  case-sensitive APFS; workspace clippy passes; `source-structure-check` passes.
- Crash matrix: exit 0 on both variants, run on the repaired source before it was
  committed. The Rust changes after that run are one assertion message and one
  documentation-contract law.
- `cargo test -p xtask` locally: 419 passed, 7 failed, all seven caused by the
  local runner or a missing `b3sum`.
- Not run locally: Linux, and the checks that need pinned documentation tools.
  CI covers them.

## Still open

- #221 and #205 block the first release.
- #222 collects the follow-ups, R22 to R34 and others found during repair.
- A no-op production synchronizer still survives every test. Only a power-loss
  test could kill it, and none exists.
