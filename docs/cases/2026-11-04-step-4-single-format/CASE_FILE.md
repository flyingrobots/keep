# Case file: backlog step 4, one storage format (PR #220)

- **Date:** 2026-11-04
- **Subject:** PR #220, `integrate/open-prs` into `main`, at `fb355d66`
- **Evidence:** head `fb355d66`, base `27d5c38c`, CI run 41953913643
- **Testimony:** [Holmes A](HOLMES-A.md), [Holmes B](HOLMES-B.md),
  [Watson](WATSON.md), [Moriarty](MORIARTY.md),
  [Moriarty's cross-examination](MORIARTY-CROSS.md)

⚠️ ALL VERDICTS HEREIN ARE VALID ONLY FOR THE COMMITS ABOVE.

## Executive Deduction

The promise this PR adds does not yet hold: a public open takes no expectation,
a caller in a later process has no public way to rebuild one, and one crash
strands a caller holding one. No witness made Keep return wrong bytes or admit
an incomplete store, and the exact-name check has no bypass on macOS. Several
laws and documents claim more than they establish, so the PR should not merge at
this head.

## Report Card

| Measure | Witness | Grade |
| --- | --- | --- |
| Fidelity | Holmes A, exact names and macOS | B- |
| Fidelity | Holmes B, ADR, expectation and identifiers | B |
| Clarity | Watson | C+ |
| Resilience | Moriarty: 11 strikes (0 fatal, 5 major, 6 minor), 2 vectors surrendered | C+ |
| Cross-examination | 5 rows: 0 refuted, 4 weakened, 1 stands | n/a |
| **Overall** | Front office | **C+** |

## Adjudication

**Sound at this head**

- Direct initialization has one commit point, and no admission precedes it. This
  survived Holmes B, Moriarty and the cross-examination. The ordering rests on
  reading rather than a witness; see finding 4.
- No store path resolves a protocol name past the exact-name check on macOS. Holmes
  A's census, Moriarty's surrender on vector 1 and the cross-examination agree.
- No merge damage was proven from either parent, and Keep persists no device,
  inode or store identifier.
- Every retired ID is in the ledger, and none is reused. The cross-examination
  recomputed an empty residual.

**Findings, most consequential first**

1. **`FilesystemCatalogSnapshot::load` escapes the expectation.** Holmes B (D1),
   Moriarty (S1) and the cross-examination found this independently, which is
   strong evidence. The type's docs, two requirement pages and the law's header all
   claim otherwise.
2. **A caller can't rebuild an expectation in a later process.** Watson raised this
   after reading the other testimony, so no detective tested it. The front office
   checked it statically: `CatalogDigest::from_validated` is `pub(crate)`
   (`src/catalog/digest.rs#19@fb355d66`), and `ExpectedCatalog::new` requires a
   `CatalogDigest` (`src/adapters/store_expectation.rs#20@fb355d66`). Decoding a
   saved `HEAD` through the public `ChecksummedPublicationHead::decode` is the only
   route found. Nothing was compiled. **Ruled confirmed for the documented API.**
3. **A crash strands a holder of `Catalog`.** Moriarty (S2) and Watson agree. It
   fails closed, so bytes are safe, but the only exit is `NoExpectation`, which
   accepts a stranger. How often real callers would hit this is conjecture.
4. **The commit-point synchronization has no witness.** Moriarty (S3) and the
   cross-examination agree that the test named for a failed synchronization fails
   before it runs. That the mutant survives is **conjecture until it is run**.
5. **`KEEP-CRASH-079` and `-080` are unimplemented at version 1's deletion.** Holmes
   B and Moriarty (S4) agree, and the PR discloses it. The disclosure doesn't amount
   to the owner accepting the departure from the plan, which Watson rightly requires.
6. **The torn-copy laws are thinner than their names.** Holmes B (D2, D3), Moriarty
   (S5) and Watson (S4) agree. The guards can pass on an untorn store, refusal is
   untyped, and in-flight state is excluded.
7. **The `KEEP-CRASH-074` to `-087` range collides with the GC salvage.** Holmes B
   (D7) and the cross-examination agree. This isn't retired-ID reuse, and the
   front office rules it a coordination defect rather than a ledger defect.
8. **Stale documents.** Holmes A and B, Moriarty (S7) and Watson agree on the
   CHANGELOG, README, ledger wording, evidence counts and platform pages.

**Conflicts**

- **Holmes A's "no bypass" against the cross-examination's WEAKENED.** Both stand,
  at different scopes. On macOS, Holmes is right. Off macOS the check is a no-op
  whose justification is absent on unprofiled routes. The casefold experiment is
  conjecture, so this is a follow-up rather than a blocker.
- **Holmes B's "`Catalog` adds one 128-byte read" against the cross-examination.**
  The cross-examination prevails. The reader adds no read, and the writer adds
  several operations.
- **Holmes B's gate row against the cross-examination.** The code stands, and the
  evidence is narrower: the second entry has no test holding it in place.
- **Holmes A's "9 failures consistent with environmental" against Moriarty S6 and
  Watson S5.** Moriarty and Watson prevail. Linux CI can't clear a macOS-only
  failure, and the crash matrix terminates children through the same `ProcessGroup`.
- **Holmes A's "CI at this head proves 937" against Holmes B's "UNVERIFIABLE".**
  Holmes A read the CI log and Holmes B only read the tree, so CONFIRMED.
- **Watson's "blocks merge" list.** The front office keeps it but separates evidence
  and API repairs from documentation repairs below. **CONTESTED:** whether the
  documentation repairs must land in this PR rather than directly after it. Nothing
  is released, but these pages would describe `main`, so the front office leans
  towards landing them here.

## The Repair Menu

Watson's numbering is kept, in the deposition, so detail can be found there.

**Owner decisions, first**

1. R1: does `FilesystemCatalogSnapshot::load` take an expectation?
2. R2: how does a caller recover an expectation after a crash?
3. R3: can `KEEP-CRASH-079` and `-080` follow version 1's deletion?
4. R4: who gives up `KEEP-CRASH-074` to `-087`?

**Evidence and API, blocking**

5. R5: run the three mutants (synchronization, gate re-entry, lock replacement).
6. R6: a checked constructor so an expectation can be recorded and rebuilt.
7. R7: fix the torn-copy guards, type the refusals and narrow the claims.
8. R14: witness the commit-point synchronization, or stop claiming it.
9. R15: test the gate's second entry, if its mutant survives.
10. R16: name and classify the nine `xtask` failures, and diagnose `process_group`.
11. R13: decide the Linux gate on `filesystem_catalog_recovery_tests`.
12. R18: make the `StoreExpectation` rustdoc usable, with a compiling example.

**Documentation, blocking (contested as above)**

13. R8, R9, R10, R11, R12, R17, R19, R20, R21.

**Follow-up issues:** R22 to R34. The R3 issue, if accepted, blocks the first release.

## Ship Verdict

**REQUIRES INVESTIGATION**: sound after the named repairs.

## Chain of custody

- **Scope.** All detectives read statically, and none built or ran anything. No
  mutant or conjectured experiment has been executed. Moriarty disclosed that he did
  not read all 461 files, and his merge comparison was rename-blind plus selected
  source diffs, which is not a line-by-line audit of the 76 resolved files.
- **Holmes split.** The target exceeded the size limit, so Holmes ran as two scoped
  detectives with disjoint scopes.
- **Cross-examination selection.** The front office chose the five rows by judgement,
  favouring load-bearing ones, rather than sampling at random as the protocol says.
  That departure is recorded here. The rows came from both Holmes testimonies.
- **Timing.** The cross-examination began after both Holmes testimonies and ran
  alongside Moriarty, and neither saw the other.
- **Watson's new finding.** It arose in the second deposition, after he had read the
  others, so it isn't blind. The front office checked it statically.
- **Detective errors.**
  - Holmes B overstated the `Catalog` read count; see Conflicts.
  - Holmes A counted drift by splitting findings, which he disclosed.
  - Moriarty's `README.md` figure of 141 coordinates is arithmetic, and Watson says
    to confirm it before writing it down.
- **Author and reviewer.** The front office commissioned the work under review, and a
  fork of its session wrote the merge. The detectives were started fresh with no
  shared context.
- **CodeRabbit** skipped the draft, so no bot review counts here.
