# Repairs after the verdict

- **Date:** 2026-11-05
- **Verdict repaired:** REQUIRES INVESTIGATION at `fb355d66` ([case file](CASE_FILE.md))
- **Repaired head:** `0ab39da6` on `integrate/open-prs`, PR #220, CI run 42005696619
- **First repaired head:** `bea75ae9`, CI run 41999700415

This page records what was done about each item in the Repair Menu. It is the
author's own account. The Agency has not re-examined the repaired head, so
nothing here changes the verdict above, which stays valid only for `fb355d66`.
One fresh reviewer read the first six repair commits; see the second pass below.

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

## Second pass

A reviewer with no shared context read `fb355d66..bea75ae9` statically and ran
nothing. It found nothing blocking, four items to fix and six minor ones. All
ten were addressed in four further commits.

| Finding | Outcome | Commit |
| --- | --- | --- |
| `load` compared after loading the whole store, so damage or a limit could mask a mismatch | Compares what `HEAD` names before the catalog is read. A law pins the order. | `a172b4cd` |
| The torn-copy refusal list accepted policy and resource errors | Accepts only errors about a segment's own bytes. | `e1fb91c3` |
| A mix that behaves as the older state could satisfy both torn-copy guards | The laws also require a refusal at an absent or malformed artifact. | `e1fb91c3` |
| The recorded catalog was shown to admit in six of twelve crash cases | Checked before recovery in every case that left `HEAD` at generation one. | `33a99772` |
| Six wording and assertion items | Corrected. | `a172b4cd`, `0ab39da6` |

The message of `7c03b179` says initialization delegates every root
synchronization. That is false: only the lock phase and the three
root-synchronization phases are delegated. The code comment and the docs are
corrected in `0ab39da6`; the commit message stands, because the branch is not
rewritten. The message of `0037255f` states the crash check more broadly than
that commit implemented; `33a99772` makes it true.

The reviewer did not reach most of `docs/architecture/filesystem-platforms.md`
or the rest of `ROADMAP.md`.

## Evidence at the repaired head

- Native macOS, Rust 1.96: 946 passed and 0 failed on default and on
  case-sensitive APFS; workspace clippy passes; `source-structure-check` passes.
- Crash matrix: the full matrix exited 0 on both variants on `bea75ae9`'s source.
  The `retained-catalog` sequence, the only crash-matrix code changed since,
  exited 0 at `0ab39da6` on default APFS. CI runs the full matrix at this head.
- `cargo test -p xtask` locally: 419 passed, 7 failed, all seven caused by the
  local runner or a missing `b3sum`.
- Not run locally: Linux, and the checks that need pinned documentation tools.
  CI covers them.

## Still open

- #221 and #205 block the first release.
- #222 collects the follow-ups, R22 to R34 and others found during repair.
- A no-op production synchronizer still survives every test. Only a power-loss
  test could kill it, and none exists.
