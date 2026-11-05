# Testimony of Dr. Watson: PR #220, first deposition (blind)

**Scope.** I read at HEAD `fb355d66`, base `27d5c38c`, and changed nothing. I judged whether a reader can follow the documents, not whether the code is correct. A few live facts come from `gh` and may change: the PR is a draft with no review decision, the listed checks pass or skip, and #198 and #205 are open. PR body citations name a section, because the body has no line numbers. Conjecture is labelled.

## Overall impression

The PR body is more candid than most. It says plainly that `KEEP-CRASH-079` and `-080` are not implemented, that a `Catalog` expectation misses rollbacks that leave the catalog unchanged, and that the crash matrix reports no row count (PR body §Recovery, §Failure modes, §Tests).

ADR-0010 is the best document here. Its rejected alternatives are concrete, and it refuses to let the record stand as proof: "Its laws and evidence, not this record, establish that they shipped" (`docs/adr/0010-one-storage-format.md#303@fb355d66`).

Three problems remain:
- The merge decision is buried.
- The docs teach only the weak `StoreExpectation` choice.
- Several documents were written step by step and never re-read as a whole, so stale tenses and contradictions survive.

## Can a reader tell plans from shipped fact?

**Mostly in the PR body, unevenly elsewhere.**
- **Good:** the crash table's Status column separates Implemented, Retired and Planned (`docs/formats/store/catalog-publication.md#173-175@fb355d66`).
- **Bad:** the CHANGELOG says "Main's behavior is unchanged until the units tracked in #198 land" (`CHANGELOG.md#29@fb355d66`). The next line says version 1 is deleted (`CHANGELOG.md#30@fb355d66`).
- **Bad:** the rationale page, added by this PR, says "this change adds no filesystem version-2 catalog publisher" (`docs/formats/store/rationale.md#122-123@fb355d66`). The store README lists that publisher as implemented (`docs/formats/store/README.md#154-156@fb355d66`).

## Can a reader tell what this PR does not do?

**Only by collecting it from several places.** There is no nonclaims section. Four gaps stand out:
- **#198 closure:** the PR links no closing issues and never says whether #198 closes. Its commits are tagged #198 while its title says #204.
- **#205:** ADR-0010 says #205 must close before the first release, because the digest does not cover the grammars (`docs/adr/0010-one-storage-format.md#89-93@fb355d66`). The PR body says "the definition digest changes" and never mentions the gap.
- **Ingestion:** the body says the stack fixes "a migrated store cannot ingest". The store README then says production durable ingestion (#82) is absent (`docs/formats/store/README.md#177-180@fb355d66`). I had to re-read to infer that one-segment publication is not bounded production ingestion, and no document draws that line.
- **Plan departure:** the most important sentence for a merger is "The agreed plan wanted them before version 1 is deleted, so this PR does not satisfy that yet". It sits in the second-to-last section. It cites no plan, follow-up issue or owner waiver. I found no `-079` or `-080` requirement in the ADR, ROADMAP or #198 body, though my search was narrow.

## Could a reader use `StoreExpectation` correctly?

**`NoExpectation`: yes. `Catalog`: not from the docs.** The enum's rationale and its limit are clear (`src/adapters/store_expectation.rs#37-47@fb355d66`), but using `Catalog` runs into these gaps:

1. **No worked example.** The only caller examples, the README and the rustdoc, pass `NoExpectation` (`README.md#222@fb355d66`, `src/adapters/durable/store.rs#52@fb355d66`). `StoreExpectation::Catalog` appears only in tests (`tests/single_format/expectation_laws.rs#53@fb355d66`). The safety feature is taught by showing how to decline it.
2. **Keeping the value outside the store has no documented route.** The docs say to keep it externally, but `CatalogDigest` exposes only `as_bytes` publicly, and its constructor is `pub(crate)` (`src/catalog/digest.rs#14-20@fb355d66`). The one route I found is storing the encoded `HEAD` and decoding it with `ChecksummedPublicationHead::decode` (`src/adapters/checksummed_publication_head.rs#32@fb355d66`). Nothing documents this. I did not compile it.
3. **"Every open" is too broad.** The rustdoc says "Every open therefore requires this choice" (`src/adapters/store_expectation.rs#39-40@fb355d66`). Recovery and verification opens take none (`docs/formats/store/recovery.md#138-139@fb355d66`), and neither does the feature-gated `reopen_unchecked_for_repository_tasks` (`src/adapters/filesystem_store_admission.rs#140@fb355d66`). The enum's own docs omit the exception.
4. **`DurableStore::open` checks nothing.** It takes the expectation but does not open the store; the check happens at `snapshot()`. Its rustdoc is honest about this (`src/adapters/durable/store.rs#71-76@fb355d66`), and the law confirms it (`tests/single_format/expectation_laws.rs#61-62@fb355d66`). The CHANGELOG and `recovery.md` group all four opens together. The `snapshot()` error list then omits expectation (`src/adapters/durable/store.rs#109-110@fb355d66`), though `DurableSnapshot::open` includes it (`src/adapters/durable/snapshot.rs#93-95@fb355d66`).
5. **The update cycle is unwritten.** Nothing says that a writer must record the new value after each publish. The stale-expectation warning appears only on `DurableStore::open`. *Conjecture:* a crash between publishing and recording, or recovery finalizing `head.next`, leaves a healthy store that refuses, and the docs give no way out.
6. **"Expected nothing published" cannot be said.** `Catalog` needs a catalog, and `NoExpectation` accepts anything. *Conjecture:* a caller who initialized a store and wants to confirm it is still empty has no way to express that. No document names the gap.
7. **`NoExpectation` overstates what it accepts.** It "accepts any valid store" (`src/adapters/store_expectation.rs#48-49@fb355d66`), yet an initialized store with no catalog refuses on the read path as `Unpublished` (`src/adapters/retention/filesystem_retention_snapshot.rs#120-121@fb355d66`). The caveat is on `load`, not on the variant.
8. **The observed value has a misleading type.** `observed()` returns `Option<ExpectedCatalog>` (`src/adapters/store_expectation.rs#84@fb355d66`). The error's `Display` omits both values (`src/adapters/store_expectation.rs#91@fb355d66`), so "refuses with the expected and observed values" holds only through accessors.
9. **Wrong-store detection has an unstated edge in the main docs.** Two stores with the same bytes are "indistinguishable by design", but only the rationale says so (`docs/formats/store/rationale.md#95-97@fb355d66`). The enum's "the store the caller meant" (`src/adapters/store_expectation.rs#37-38@fb355d66`) does not qualify it.

## Stumble list

**Misled, needed cross-checking**
- **S1.** "A quiescent copy is the same store" (`docs/formats/store/README.md#57-58@fb355d66`, also `docs/formats/store/initialization.md#30@fb355d66`). Against that: "A copy and its original are two stores" (`docs/formats/store/recovery.md#122@fb355d66`) and "a second, independent store" (`docs/formats/store/requirements.md#108-109@fb355d66`). This is a direct contradiction in normative pages.
- **S2.** The store README's Status section never mentions `StoreExpectation`. It describes `reopen` as if no expectation existed (`docs/formats/store/README.md#152-153@fb355d66`). A new engineer finds the concept only in the Portable stores section of `recovery.md`.
- **S3.** "The format contract is frozen" (`docs/formats/store/README.md#141@fb355d66`). The ADR says a pre-release grammar change only regenerates fixtures, and that the digest does not yet cover the grammars (`docs/adr/0010-one-storage-format.md#89-94@fb355d66`).
- **S4.** The torn-copy claims are broader than the evidence. The ledger says "every partly copied file" (`docs/formats/store/catalog-protocol-requirements.md#36@fb355d66`). `recovery.md#118-119` says a torn copy may hold any mix of two published states. The PR body admits "two adjacent published states, and half-length files only", and the test agrees (`tests/single_format/torn_copy_laws.rs#154-159@fb355d66`). *Conjecture:* a slow copy can span more than two publications.
- **S5.** "All 9 look environmental", yet the same list says 2 `process_group` laws are "not diagnosed" (PR body §Tests). Failing the same way on main shows the failure predates this PR, not that it is environmental. *Conjecture:* Linux CI may also not settle a failure that only happens on macOS.
- **S6.** The CHANGELOG names `FilesystemVersionTwoAdmission::initialize` and `FilesystemVersionTwoCatalogPublisher` (`CHANGELOG.md#32@fb355d66`, `CHANGELOG.md#34@fb355d66`). These are renamed by the entry above them (`CHANGELOG.md#30@fb355d66`). Read top-down, the entries run backward and name types that no longer exist.
- **S7.** The CHANGELOG entry for the expectation ends on "refuses or returns exact bytes" (`CHANGELOG.md#11@fb355d66`). It omits the limit stated in the PR body: without an expectation, a torn copy can silently be the older state.
- **S8.** The README still advertises "Durable version-1 segment store" and "version-1 publication" (`README.md#48@fb355d66`, `README.md#58@fb355d66`), and later says version 1 is deleted (`README.md#98@fb355d66`).

**Stopped and re-read**
- **S9.** "Step 4" has two meanings: the PR's backlog step and ADR-0010's writer-before-deletion step 4 (`docs/adr/0010-one-storage-format.md#180@fb355d66`). The ledger also uses "ADR-0010 step 5" (`docs/formats/store/retired-identifiers.md#105@fb355d66`). The PR body never says which plan its steps belong to.
- **S10.** "Before the first release, opening a store therefore requires a typed choice" (`docs/adr/0010-one-storage-format.md#152@fb355d66`, repeated at `CHANGELOG.md#29@fb355d66`). This could mean a deadline or a requirement that lapses at release. I read it as a deadline, but only by inference.
- **S11.** The retirement ledger contradicts itself. Line 83 says `-078` to `-080` "are reserved"; line 164 retires `-078` (`docs/formats/store/retired-identifiers.md#83@fb355d66`, `#164@fb355d66`). Dispositions at lines 26 and 33 say "until version-1 admission is deleted", and only line 136 corrects them. Line 26 points at `KEEP-RETENTION-014`, which line 132 says was "Never on `main`".
- **S12.** `requirements.md` summarizes retirement as only the migration IDs and `-053` to `-073` (`docs/formats/store/requirements.md#71@fb355d66`). It omits the version-1 retirements.
- **S13.** "Expected state" means several things: this feature, a retention transition's precondition (`docs/formats/store/retention-publication.md#37@fb355d66`), and a test oracle (`docs/formats/store/stage-recovery.md#150@fb355d66`). A separate `catalog_publication_expectation.rs` adds to the confusion.
- **S14.** ADR-0010 offers "a retention head digest" as a possible expectation (`docs/adr/0010-one-storage-format.md#148-149@fb355d66`), then narrows to catalog only (`#156`). The PR body explains why, but the ADR does not. The ADR is marked "amended 2026-11-04" with no list of what the amendment changed.
- **S15.** The ADR's version-1-era wording remains, such as "Main supports two durable formats" and "once version-1 admission is removed" (`docs/adr/0010-one-storage-format.md#18@fb355d66`, `#86-87@fb355d66`). That is acceptable as history, but the store README sends readers there as a current authority.

**Maintainer friction**
- **S16.** "Please review the merge commit's conflict resolutions separately" comes with no command, list of conflicted files or route through the 461 changed files (13,564 added, 19,947 deleted). The body does name the four follow-up subjects, and the first-parent history is clean at five commits. A reviewer still has to build the rest.
- **S17.** The body uses unglossed terms: "#216", "the owner", "Darwin's restart coordinate domain", "the lenient Linux mount policy". The body gives no result for the CI it relies on.
- **S18.** The store README says "Every page describes the behaviour of `main`" (`docs/formats/store/README.md#17@fb355d66`). That is untrue until merge, and S1, S3 and the rationale's stale publisher sentence strain it afterward.

## What held up

- All 19 ledger commits I spot-checked exist. 18 are ancestors of main, and `e3930f0` is honestly marked as within #198.
- The CHANGELOG digest prefix and suffix match the full digest in `conformance/segment-store/v2/README.md#42@fb355d66`.
- The `Root identity across restart` anchor resolves (`docs/formats/store/recovery.md#141@fb355d66`).
- ADR-0009 carries its "amended by" notice (`docs/adr/0009-retention-roots-release-and-gc-liveness.md#8@fb355d66`).
- The README's explanation of why `conformance/.../v1` keeps its name (`docs/formats/store/README.md#135-137@fb355d66`) saved me a real confusion.

```text
VERDICT
Witness: Watson, comprehension pass, first deposition (blind)
Clarity grade: C+
  PR body (maintainer):                B-   candid, but the plan departure is buried
  ADR-0010:                            B+   clear decision; stale tenses, unmarked amendment
  Store README (new engineer):         C    well organized; S1, S2, S3 mislead
  Retired identifiers:                 B-   sound rule, sections layered without reconciling
  StoreExpectation rustdoc:            C-   explains why; does not show how to use Catalog
  CHANGELOG:                           D+   backward story, dead names, a stale line
Plans vs shipped fact: mostly distinguishable; CHANGELOG#29 and rationale#122-123 fail
What this PR does not do: stated, but scattered; no nonclaims section
StoreExpectation usable from docs: NoExpectation yes; Catalog no
Worst stumbles: the unanchored -079/-080 departure, S1, Catalog gaps 1, 2 and 5, S4, S5, S6
Limits: comprehension only; no build, no code verification; live gh facts may change
```

## Second Deposition — The Repair Menu

**Plainly stated:** no strike is fatal to the Core Law. No witness made Keep return wrong bytes or admit an incomplete store. I would not merge at `fb355d66` all the same. Three claims cannot be repaired with wording:

| Claim | Why rewording fails | Repair |
|---|---|---|
| "No caller loses the check by omission" (`src/adapters/store_expectation.rs#39-40@fb355d66`) | It is false while `FilesystemCatalogSnapshot::load` stays public and takes no expectation (`src/adapters/filesystem_catalog_snapshot.rs#42@fb355d66`). | R1 |
| `Catalog` protects a caller across restarts and crashes | It cannot do this as shipped; see the new finding below and Moriarty's S2. | R2, R3 |
| The commit-point synchronization is witnessed | The test's failure fires before the sync runs (`src/adapters/store_initialization/initialization_resume_tests.rs#119-125@fb355d66`). If the mutant survives, the commit point is certified by a label. | R5, R14 |

**New finding, not yet cross-examined or compiled:** a caller seems unable to rebuild an expectation in a later process.
- `CatalogDigest`'s only constructor is `pub(crate)` (`src/catalog/digest.rs#19@fb355d66`), and I found no conversion trait for it.
- The receipt and view I read return the typed digest but no head bytes (`src/adapters/catalog_publication_receipt.rs#22@fb355d66`, `src/adapters/durable/view.rs#34@fb355d66`).
- The only route I found is to copy the raw `HEAD` file and pass it to `ChecksummedPublicationHead::decode` (`src/adapters/checksummed_publication_head.rs#32@fb355d66`).
- The laws miss this because they build the expectation inside one process (`tests/single_format/expectation_laws.rs#25-37@fb355d66`).

**Reading this menu:**
- Repairs run in order of yield per effort.
- Effort figures are my estimates, not measurements: XS is under 15 minutes, S under 1 hour, M about half a day, L 1 to 2 days.
- Any cargo experiment must use the project's reusable worker, guarded runner and git-locks keys. Mutants are never committed.

### Owner decisions, needed first

**R1. Does `FilesystemCatalogSnapshot::load` take an expectation?** NEEDS OWNER DECISION. Decision S; edit depends on the option.
- **Option A, add the parameter (recommended):** `load(root, policy, expectation)` compares the loaded generation and digest. It touches about 27 call sites, mechanically, including 5 in `xtask` restart code, and needs one new law. Effort M. This keeps the promise true.
- **Option B, name a third exemption:** call it "observational, unfenced" in ADR-0010 and retract the universal claim. Effort S. It is cheap and honest, but it concedes that a public read can lose the check by omission.
- **Option C, make it non-public:** rejected on the evidence, because `xtask` and many integration laws use it from outside the crate.
- **Under any option:** state in the ADR that `initialize` on a marker-linked populated root (`docs/formats/store/initialization.md#133@fb355d66`) and `reopen_unchecked_for_repository_tasks` (`src/adapters/filesystem_store_admission.rs#140@fb355d66`) also take no expectation.

**R2. How does a caller recover an expectation after a crash?** NEEDS OWNER DECISION. Decision S; the minimum documentation in option A is S; option B is M to L.

After death past `HEAD` replacement, or after gated finalization, the old `Catalog(E)` refuses forever. `NoExpectation` also accepts a stranger.
- **Option A, caller-owned protocol, documented now (minimum to merge):** add a "Recovering a stale expectation" section to `docs/formats/store/recovery.md`. It must state that:
  - the mismatch is expected after a crash;
  - Keep cannot tell a successor from a stranger through this API;
  - re-pinning through `NoExpectation` is an explicit trust decision made out of band.
  
  Add a law at `KEEP-CRASH-077` that pins this refusal.
- **Option B, a lineage variant (my recommended follow-up):** add `CatalogOrImmediateSuccessor(ExpectedCatalog)`. It accepts the exact catalog, or generation + 1 whose verified `previous_catalog_digest` equals the expected digest (`src/adapters/checksummed_catalog.rs#101@fb355d66`). Divergent copies of the same predecessor still pass, and the docs must say so.
- **Not recommended:** silently accepting descendants. That rewrites what `Catalog` promises.

**R3. Can `KEEP-CRASH-079` and `-080` follow version 1's deletion?** NEEDS OWNER DECISION. Recording it is XS; implementing the rows is L.
- **Accept as a follow-up (my recommendation):** nothing is released, no stores of value exist, the routes fail closed, and main never had these rows, so no evidence is lost. They should land as their own PR, though the owner must accept this departure from the agreed plan in writing.
- **The cost of accepting:** these remain unproven: death during recovery's completion link, death during gated `HEAD` replacement, and "the gate reruns on retry". The issue should block the first release.
- **Edit:** replace "does not satisfy that yet" with the decision, its date and the issue number. Add the same dependency to ADR-0010's Writer before deletion section.
- **If the owner refuses:** this becomes BLOCKS MERGE, and the rows run through the existing `retained-catalog` sequence.

**R4. Who gives up `KEEP-CRASH-074` to `-087`?** NEEDS OWNER DECISION. XS to decide, S to edit.

The GC salvage binds these IDs to GC, while head assigns them elsewhere (`ROADMAP.md#1249@fb355d66`, `docs/formats/store/catalog-publication.md#169@fb355d66`).
- **Recommended:** GC renumbers from `-092`, since head's assignments would be on main.
- **Edit:** correct `ROADMAP.md#1249,#1416,#1422@fb355d66`, and add a warning to `docs/audits/pr-107-salvage.md` at the GC row.

### Blocks merge

**R5. Run the three mutants.** BLOCKS MERGE. S.
1. Make `synchronize_root` return `Ok(())` (`src/adapters/store_initialization/initialization_entries.rs#137@fb355d66`).
2. Remove the gate re-entry at `src/adapters/filesystem_recovery_next_head_finalization_storage.rs#53@fb355d66`.
3. Delete the lock replacement at `tests/single_format/portability_laws.rs#93-94@fb355d66`.

Survivors decide the scope of R14, R15 and R24. Record the results in `docs/testing-evidence/`.

**R6. Make an expectation recordable across processes.** BLOCKS MERGE. S to M.
- Add a checked constructor: `ExpectedCatalog::from_recorded(CatalogGeneration, [u8; 32])`.
- Do not expose a public `CatalogDigest` constructor, because its docs call it a verified digest (`src/catalog/digest.rs#3@fb355d66`). Keep the claim-versus-verified distinction inside `ExpectedCatalog`.
- Add a law that records the two values as plain bytes, reconstructs them, and both admits and refuses through all four opens.
- Keep's durable formats are not involved. Persistence stays the caller's responsibility, and the docs must say so.
- First have someone confirm or refute the new finding above.

**R7. Fix the torn-copy guards and narrow their claims.** BLOCKS MERGE. S to M.
- **Guards:** exclude masks 0 and all-ones from the `Outcomes` counts at `tests/single_format/torn_copy_laws.rs#137-138@fb355d66`.
- **Refusals:** replace `Err(_refusal)` with an exhaustive list of lawful refusal types at `#106`. An operational failure must fail the test.
- **Name:** rename `every_partly_copied_file` at `#143` to `a_half_length_copy_of_each_changed_file_...`.
- **Ledger:** reword `docs/formats/store/catalog-protocol-requirements.md#36@fb355d66` to: "Every mix of two adjacent quiescent published states, and each changed file at half length over the newer state, refuses or returns exactly the named bytes."
- **Spec:** at `docs/formats/store/recovery.md#117-120@fb355d66`, separate what a torn copy may contain from what is tested, and say in-flight stages are untested.
- **PR body:** delete "covers the outcomes deterministically".

**R8. Replace every false universal.** BLOCKS MERGE. S, after R1.

Under option A, change the sites below to "every normal open" with the same list of exemptions. Under option B, use this text:

> "`reopen`, `load`, `DurableSnapshot::open` and `DurableStore::open` require this choice. Recovery, verification, and the observational `FilesystemCatalogSnapshot::load` take none, so a caller of those routes does not get this check."

Sites:
- `src/adapters/store_expectation.rs#39-40@fb355d66`
- `docs/formats/store/requirements.md#47,#108@fb355d66`
- `docs/formats/store/catalog-protocol-requirements.md#35@fb355d66`
- `tests/single_format/expectation_laws.rs#1@fb355d66`
- `docs/formats/store/publication.md#156@fb355d66`
- the PR body's invariant and failure modes

**R9. Resolve "same store" against "two stores".** BLOCKS MERGE. XS.
- At `docs/formats/store/README.md#57-58@fb355d66` and `docs/formats/store/initialization.md#30@fb355d66`, write: "a quiescent copy opens and holds the same content, and it is a second, independent store".
- This matches `docs/formats/store/recovery.md#122@fb355d66`.

**R10. Repair the CHANGELOG.** BLOCKS MERGE. S.
- Delete "Main's behavior is unchanged until the units tracked in #198 land" (`CHANGELOG.md#29@fb355d66`).
- Fold lines 30 to 36 into one entry, in order, using only head names: `FilesystemStoreAdmission::initialize` and `FilesystemCatalogPublisher`.
- To `CHANGELOG.md#11@fb355d66`, add #198 and this limit: "Without an expectation, a torn copy may silently be the older state. A `Catalog` expectation misses rollbacks that leave the catalog unchanged."
- Add a line for the coordinate type, the removal of the Darwin domain, and the unimplemented `-079` and `-080`.

**R11. Reconcile the retirement ledger.** BLOCKS MERGE. S. All edits are in `docs/formats/store/retired-identifiers.md`.
- `#7`: define the column truthfully: "a commit on `main` carrying the evidence", or re-derive every row. Moriarty's spot check shows "last commit" is false for at least one row.
- `#26,#33`: state the final disposition (`KEEP-INITIALIZATION-003`), not the intermediate one.
- `#69`: "the root synchronization after the marker link is the commit point", matching `docs/formats/store/initialization.md#77@fb355d66`.
- `#83`: say `-078` is retired and `-079` and `-080` are reserved.
- `#109,#173`: say "replaced by format-neutral successors with these names" where the names persist.
- `#132`: cite `tests/version_two_catalog_writer/version_laws.rs` at `e3930f0`, and replace "Never on `main`" with "introduced and retired within this PR".

Also add the version-1 retirements to `docs/formats/store/requirements.md#71@fb355d66`.

**R12. Correct the stale evidence record.** BLOCKS MERGE. S. All edits are in `docs/testing-evidence/filesystem-platform-adapters.md`.
- `#22`: change 963 to 937, and pin CI run `41953913643`.
- `#71`: name the compile-fail doctest successor (`src/adapters/filesystem_catalog_publisher.rs#61-65@fb355d66`).
- `#72-73,#119`: record the public-boundary law (`tests/single_format/initialization_laws.rs#64@fb355d66`).
- `#49,#57`: stop listing laws as lookup witnesses after line 49 says they are not.

**R13. Decide the Linux gate on `filesystem_catalog_recovery_tests`.** BLOCKS MERGE. S.
- Try removing the gate at `src/adapters/mod.rs#91-92@fb355d66`, and let macOS CI answer.
- If it fails for a real reason, add it to the Linux-only list at `docs/testing-evidence/filesystem-platform-adapters.md#69-78@fb355d66`, with that reason.

**R14. Witness the commit-point sync, or stop saying it is witnessed.** BLOCKS MERGE. Minimum S; proper M.
- **Minimum:**
  - Rename `a_failed_commit_synchronization_...` to name what it does, an observer refusal before the commit phase.
  - Assert the exact namespace variant.
  - Downgrade the sync clause in `docs/formats/store/requirements.md#45@fb355d66` from Implemented.
- **Proper (recommended):** restore a recording witness like base's `RecordingStorage` (`tests/store_initialization.rs#83@27d5c38c`). It should prove link, then root sync, then stage removal at the storage boundary, with one real failed sync.
- A process-kill matrix cannot prove this, because the page cache survives process death.

**R15. Test the gate's second entry.** BLOCKS MERGE if the R5 mutant survives; otherwise FOLLOW-UP. S to M.
- Swap the retention head between `synchronize_candidate` and `replace_head`, and assert `RetainedClosure`. The existing law only proves the first entry (`tests/catalog_writer/finalization_laws.rs#93@fb355d66`).
- Correct `tests/...` citations that imply otherwise, and note that `AlreadyFinalized` skips the gate without moving `HEAD` (`src/adapters/filesystem_recovery_next_head_finalization_storage.rs#31-37@fb355d66`).

**R16. Name and classify the nine `xtask` failures.** BLOCKS MERGE. S.
- List the two `process_group` law names, and run them natively outside the sandbox.
- Replace "All 9 look environmental" with: "3 reproduced identically on main, 2 of those undiagnosed; 6 read from their messages; Linux CI passes, which clears none of the macOS-only causes."
- This matters because the crash matrix terminates children through this same `ProcessGroup` (`xtask/src/durability_crash_matrix/process.rs#11@fb355d66`).

**R17. Restructure the PR body for the merging reader.** BLOCKS MERGE. S.

Add a short section directly after Problem, "Before you merge":
- the R1 to R4 decisions and their status;
- whether #198 closes (it links no closing issue);
- #205 remains open and blocks the first release;
- #82 production ingestion stays absent, while one-segment publication works;
- the path to the merge review: `git show --remerge-diff 5360a804`, the list of conflicted files, and the first-parent order.

Also:
- rename "Step 4" to "backlog step 4", to avoid colliding with ADR-0010's step 4;
- define #216 and the Darwin and Linux terms in a clause each;
- report the three 937 CI runs at this head;
- correct the benchmark line: the reader's `Catalog` adds no read, and the writer adds a clone, stats, an open and two reads, plus scans on macOS (`src/adapters/catalog_restart_io.rs#27,#30,#67,#81@fb355d66`);
- disclose the feature-gated acceptance law (`tests/single_format/suite.rs#3@fb355d66`), the lost descriptor-exhaustion coverage (`docs/formats/store/retired-identifiers.md#90@fb355d66`), and the injected, non-process-death interruption;
- remove or evidence "which macOS's native tests found".

**R18. Make the `StoreExpectation` rustdoc usable.** BLOCKS MERGE. M, after R1, R2 and R6.
- Add a compiling `Catalog` doctest: record from a receipt, reopen in a later process, match the mismatch.
- On the type, document the update cycle, the crash consequence and the link to R2's section.
- On `DurableStore::open`, say in its first sentence that it checks nothing until `snapshot()`. Add expectation to `src/adapters/durable/store.rs#109-110@fb355d66`.
- On `NoExpectation`, add that an initialized store with no catalog still refuses on the read path (`src/adapters/retention/filesystem_retention_snapshot.rs#120-121@fb355d66`).
- State that two stores with identical bytes cannot be distinguished, and that "expected empty" has no representation.
- Change the README's headline example to `Catalog`, or add an adjacent one that explains when `NoExpectation` is acceptable (`README.md#222@fb355d66`).
- Rename the observed value's type to `PublishedCatalog`, used on both sides, and put both values in `Display` (`src/adapters/store_expectation.rs#84,#91@fb355d66`).

**R19. Correct the store README.** BLOCKS MERGE. S.
- `docs/formats/store/README.md#141@fb355d66`: replace "frozen" with "The current contract. Before the first release a grammar change regenerates fixtures. #205 remains open."
- `#150-153`: mention the expectation, linking `recovery.md`'s Portable stores.
- `#177-180`: separate one-segment publication from #82.
- `#17`: "describes `main` after this change", with nonclaims linked.

Also fix `README.md#48,#56-58@fb355d66`: the current format, and 141 coordinates over 47 rows, per Moriarty's arithmetic. Confirm the number before writing it.

**R20. Correct the remaining stale specifications.** BLOCKS MERGE. S to M.

These are false present-tense normative statements:
- `docs/formats/store/rationale.md#122-123@fb355d66`: the publisher now exists.
- `docs/formats/store/publication.md#29@fb355d66`: default APFS is admitted.
- `docs/formats/store/stage-recovery.md#384@fb355d66`: nine directories, not three.
- `docs/formats/store/stage-recovery.md#404-411@fb355d66` and `docs/architecture/filesystem-platforms.md#105,#114,#154-155@fb355d66`: restrict "every store path" to the routes that actually use the profile.
- `docs/architecture/filesystem-platforms.md#39,#76-84@fb355d66`: the two-method trait, no object coordinate.
- `docs/architecture/filesystem-platforms.md#185@fb355d66` and `docs/adr/filesystem-platform-adapters.md#82@fb355d66`: fixtures changed.
- `docs/architecture/filesystem-platforms.md#235,#249@fb355d66`: "copied roots", and the "Current" label on a deleted page.
- `docs/adr/0010-one-storage-format.md#76@fb355d66`: "no `FORMAT.next` in a committed store".

**R21. Remove the roadmap's dead acceptance criteria.** BLOCKS MERGE. S.
- Rewrite T-41.1 (`ROADMAP.md#2403-2410@fb355d66`) against live IDs, and put a macOS row in the ledgers or untick it.
- Relabel `ROADMAP.md#7-8@fb355d66`.
- The roughly 74 remaining migration mentions go to R34, except any that make a false statement about main.

### Follow-up issues

**R22. Clarify ADR-0010's language.** FOLLOW-UP ISSUE. S.
- `docs/adr/0010-one-storage-format.md#152@fb355d66`: change to "No later than the first release", and mirror it in the CHANGELOG.
- `#148-149`: add that `Catalog` alone ships.
- Add an amendment list to the header.
- Add a sentence at `#18`: "This section describes main as of the decision."

**R23. Add in-flight mixes to the torn-copy law.** FOLLOW-UP ISSUE. L.
- Capture trees at each `CatalogPublicationPhase`, overlay them, and run the reader and recovery tools.
- ADR-0010's step 4 is met only narrowly until then, so it needs an owner's nod in R3's note.
- Also add absent unchanged files, other truncation lengths, and tears over the older state.

**R24. Tighten weak witnesses.** FOLLOW-UP ISSUE. S each.
- Split the lock half of `KEEP-PORTABILITY-003` so it can fail on its stated cause (`tests/single_format/portability_laws.rs#92@fb355d66`).
- Replace the text-grep law at `tests/store_admission_contract.rs#24@fb355d66` with behavior.
- Assert a typed failure at `tests/darwin_store_spelling.rs#53@fb355d66`.
- Compare beyond `ErrorKind` at `tests/single_format/portability_laws.rs#75@fb355d66`, and use more than one damage case.

**R25. Make the backup law really verify.** FOLLOW-UP ISSUE. S.
- Call `load_for_verification` in `tests/single_format/acceptance_laws.rs#158@fb355d66`, or narrow `KEEP-PORTABILITY-005`.

**R26. Restore descriptor-exhaustion coverage.** FOLLOW-UP ISSUE. M.
- `require_expectation` has a clone-failure branch with no successor test, per Holmes A.

**R27. Run the exact-name casefold experiment.** FOLLOW-UP ISSUE. M.
- Use a casefolded ext4 loop image against `load`, `try_acquire` and an unprofiled namespace directory (`src/adapters/filesystem_platform_profile.rs#17-27@fb355d66`).
- Moriarty's prediction, verified bytes but no spelling refusal, remains conjecture.
- If it is confirmed, extend the profile to those routes. R1 option A can cover `load` in the same change.
- Separately, record the macOS check-then-use race's spurious refusal rate.

**R28. Close the wrapper route to the gate.** FOLLOW-UP ISSUE. M.
- Seal `RetainedCatalogPublicationStorage`, or weaken "can therefore never drive gated stages" (`src/adapters/filesystem_retained_catalog_stages.rs#8@fb355d66`).
- Reconsider the public `publish_catalog_generation` (`src/adapters/exports.rs#16@fb355d66`).
- Moriarty's and Holmes's wrapper route is conjecture and needs deliberate misuse.

**R29. Run `xtask` tests on macOS.** FOLLOW-UP ISSUE. M.
- This removes the blind spot behind Moriarty's S6 (`.github/workflows/ci.yml#123@fb355d66`).

**R30. Fix the `AlreadyInitialized` diagnostic.** FOLLOW-UP ISSUE. S.
- Validate `FORMAT` before saying so, or return an existence-only classification (`src/adapters/store_initialization/initialization_observation.rs#32@fb355d66`).

**R31. Narrow the initialization resume promise.** FOLLOW-UP ISSUE. S.
- At `src/adapters/store_initialization.rs#4@fb355d66`, exclude a damaged stage after power loss, which refuses.
- Block-level replay is a separate issue, because the failure itself is conjecture.

**R32. Disambiguate "expected state".** FOLLOW-UP ISSUE. S.
- Pick distinct terms for the feature, the retention precondition, and the test oracle.

**R33. Require the acceptance law by default.** FOLLOW-UP ISSUE. XS.
- Give a plain `cargo test` a visible signal, or document that `--all-features` is mandatory.

**R34. Sweep the roadmap.** FOLLOW-UP ISSUE. M.
- Clear the remaining migration language.

### The merge gate

- **Answer first:** R1 to R4. The answers set the content of R8, R17 and R18.
- **Then:** R5 to R21. If R1 goes to option A and R14 goes proper, I estimate 3 to 4 working days. With option B and the minimum R14, about 2.
- **Not required:** R22 to R34, which should land as ordinary issues with the R3 issue blocking the first release.

### Coverage

| Source | Repairs |
|---|---|
| Watson, usability gaps 1 to 9 | R6, R8, R18 |
| Watson S1, S2, S3 | R9, R18/R19, R19 |
| Watson S4, S5, S6, S7 | R7, R16, R10, R10 |
| Watson S8, S9, S10 | R19, R17, R22 |
| Watson S11, S12, S13 | R11, R11, R32 |
| Watson S14, S15, S16, S17, S18 | R22, R22, R17, R17, R19 |
| Holmes A drifts 1, 2, 3 | R13, R12, R12 |
| Holmes A drifts 4, 5, 6 | R20, R27; R20; R20 |
| Holmes A drifts 7, 8, 9, 10 | R12; R21; R19; R20 |
| Holmes A reverse drift | R17, R21, R26, R34 |
| Holmes B D1, D2, D3 | R1, R8; R7; R7 |
| Holmes B D4, D5, D6 | R11 |
| Holmes B D7, D8, D9 | R4, R21; R20; R28 |
| Holmes B reverse drift | R1, R17, R28 |
| Moriarty S1, S2, S3, S4, S5 | R1; R2, R6; R5, R14; R3; R7, R23 |
| Moriarty S6, S7, S8, S9 | R16, R29; R12, R19, R20; R5, R24; R24 |
| Moriarty S10, S11 | R30; R31 |
| Cross, row 1 | R5, R11, R14 |
| Cross, row 2 | R1, R27 |
| Cross, row 3 | R1, R17 |
| Cross, row 4 | R4, R11 |
| Cross, row 5 | R3, R5, R15 |

```text
VERDICT
Witness:        Watson, second deposition (Repair Menu)
Fatal to bytes: none. Core Law not breached in evidence.
Fatal to claims: "no caller loses the check by omission";
                 Catalog as a restart- and crash-safe check;
                 the commit-point sync witness (pending mutant, R5)
Owner decisions: R1 load expectation; R2 crash recovery; R3 -079/-080; R4 ID collision
Blocks merge:   R5-R21 (R15 only if its mutant survives)
Follow-ups:     R22-R34
New, unverified: no public route to rebuild an ExpectedCatalog from stored values (R6)
Recommendation: do not merge at fb355d66; decide R1-R4, then land the blocks
```
