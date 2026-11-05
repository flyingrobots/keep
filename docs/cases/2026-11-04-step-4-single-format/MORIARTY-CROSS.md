# MORIARTY: cross-examination of Holmes's five rows

Everything here is static reading of `git show` and `git grep` at fb355d66, and the worktree stayed clean. I ran no builds and no tests, so every experiment below is proposed, not performed.

No row is refuted. One stands intact. Four are true but narrower than the good doctor's colleague wrote them, usually because his evidence proves less than his sentence claims.

## Row 1: "One durable commit point, no admission before it": STANDS

**What I tried, and failed at:**
- **Every admission gate.** All of them reach `admit_committed`, via `src/adapters/filesystem_store_namespace.rs#81@fb355d66` and `#61,#74`. The callers are:
  - the writer, `src/adapters/filesystem_store_admission.rs#176@fb355d66`
  - recovery, `src/adapters/filesystem_recovery_admission.rs#71-74@fb355d66`
  - the inventory reader, `src/adapters/filesystem_recovery_inventory_reader.rs#56,#88@fb355d66`
  - the fenced reader, `src/adapters/retention/filesystem_retention_snapshot.rs#154@fb355d66`
  - the catalog reader, `src/adapters/catalog_restart_loader.rs#29-32@fb355d66`
- **The one law that tolerates `FORMAT.next`.** `src/adapters/filesystem_store_namespace.rs#184@fb355d66` has one caller, `src/adapters/store_initialization/initialization_namespace.rs#39@fb355d66`. It only yields a `Prefix::MarkerLinked` classification (`src/adapters/store_initialization/initialization_observation.rs#207-208@fb355d66`) and grants no authority.
- **A bare writer lock.** `src/adapters/filesystem_writer_lock.rs#73@fb355d66` is public. It reaches nothing, though:
  - the publisher takes only an admission (`src/adapters/filesystem_catalog_publisher.rs#57-65,#75-78@fb355d66`, with a `compile_fail` doctest)
  - `from_lock` is `pub(super)` and re-admits the root (`src/adapters/filesystem_recovery_stage_discarder.rs#40,#46@fb355d66`)
- **Ordering.** Sync comes before unlink (`src/adapters/store_initialization/initialization_execution.rs#131-132@fb355d66`). I found no other production route that removes a complete stage. `discard_incomplete_stage` is gated by the strict-prefix classification, `src/adapters/store_initialization/initialization_entries.rs#96-101@fb355d66`.

**Blemishes that do not change the ruling:**
- The cited test injects its failure at `Boundary::Before`, so the sync never runs. It also asserts only `Namespace { .. }` (`src/adapters/store_initialization/initialization_resume_tests.rs#120,#140-143@fb355d66`). The exact variant is asserted elsewhere, at `tests/single_format/initialization_laws.rs#118@fb355d66`.
- The ledger contradicts the doctrine. It says "the marker link is the commit point" at `docs/formats/store/retired-identifiers.md#69@fb355d66`. The protocol doc says the root synchronization after it is, at `docs/formats/store/initialization.md#77@fb355d66`.

**CONJECTURE:** nothing witnesses the fsync itself. `synchronize_root` is called directly and the observer wraps it, so a no-op body would keep every phase notification.
*Experiment:* mutate `src/adapters/store_initialization/initialization_entries.rs#137@fb355d66` to `Ok(())` and run the suite. Survival would show the ordering is proved by reading alone.

## Row 2: "No exact-name bypass": WEAKENED

**The census holds.**
- Raw `cap_std::fs::Dir` appears only in the wrapper, the spelling check and the platform adapters.
- `.raw()` appears only at `src/adapters/filesystem_platform_profile.rs#53,#60@fb355d66`, and those paths make no name lookups (`src/adapters/darwin_filesystem.rs#42,#53@fb355d66`, `src/adapters/linux_filesystem.rs#54-58@fb355d66`).
- Line 18 of `src/adapters/sync_capable_directory.rs@fb355d66` uses the wrapper, and line 25 has no name.
- `src/adapters/filesystem_writer_lock.rs#138-140@fb355d66` opens `"."`.
- The raw `openat` at `src/adapters/darwin_filesystem.rs#26@fb355d66` walks the caller's path, not a protocol name.
- The non-test `DirEntry` uses I found are `file_name` and `file_type`, such as `src/adapters/retention/filesystem_retention_namespace.rs#87-88@fb355d66`.
- The non-test `std::fs` use I found is confined to handle types (`src/adapters/filesystem_writer_lock.rs#3@fb355d66`). The two `use std::fs;` modules are test fixtures.

**But "passes through `require_exact`" means less than it sounds:**
1. **Off macOS the check is a no-op.** See `src/adapters/filesystem_entry_spelling.rs#45-47@fb355d66`. Its justification is "Production Linux admission excludes casefold directories", at `#37`.
2. **A public route skips that admission.** `FilesystemCatalogSnapshot::load` is unconditionally compiled and exported (`src/adapters/filesystem_catalog_snapshot.rs#42@fb355d66`, `src/lib.rs#95@fb355d66`). It opens through ambient authority with no profile (`src/adapters/catalog_restart_io.rs#13-15@fb355d66`, `src/adapters/catalog_restart_loader.rs#28@fb355d66`). `FilesystemWriterLock::try_acquire` does the same (`src/adapters/filesystem_writer_lock.rs#73-78@fb355d66`).
   - On Linux, root names are still guarded by the listing at `src/adapters/filesystem_store_namespace.rs#152-168@fb355d66`.
   - Pool lookups such as `src/adapters/catalog_restart_loader.rs#103-105@fb355d66` have no such guard on this route.
3. **The admitted route has a hole too.** `src/adapters/filesystem_platform_profile.rs#17-27@fb355d66` omits `retention/roots/<namespace>`, which is then looked up at `src/adapters/retention/filesystem_retention_snapshot.rs#239-243@fb355d66`. The selected root is checked afterwards at `#288-311`, so identity survives. Exact naming does not.

**CONJECTURE (points 2 and 3):** on a casefolded ext4 loop image, respell a pool entry in another case. Then run `load` on a casefolded root, and on the admitted route set `chattr +F` on an empty namespace directory.
*Prediction:* the digest-checked bytes are served and no spelling refusal occurs.

**CONJECTURE:** the macOS check is not atomic with the lookup. The check and the use are separate calls, at `src/adapters/filesystem_directory.rs#62-63@fb355d66` and `src/adapters/filesystem_entry_spelling.rs#19-28@fb355d66`. Publication runs beside readers (`src/adapters/retention/reader_fence.rs#22-23@fb355d66`), so a scan racing a rename might refuse spuriously. Any such failure would be fail-closed.
*Experiment:* loop `rename(head.next, HEAD)` against reader opens on APFS and count `EntrySpellingMismatch`.

## Row 3: "`NoExpectation` adds no I/O; `Catalog` adds one 128-byte HEAD read": WEAKENED

- **`NoExpectation` holds.** The writer returns early (`src/adapters/filesystem_store_admission.rs#152-154@fb355d66`). The reader's comparison is pure (`src/adapters/store_expectation.rs#59-64@fb355d66`). `require_published` runs under either expectation, so the expectation adds nothing there.
- **"One read" is wrong for the reader.** `Catalog` adds no read at all. It compares the view it already collected, at `src/adapters/retention/filesystem_retention_snapshot.rs#175-180@fb355d66`.
- **"One read" undercounts the writer.** The 128 bytes are right; the operation count is not:
  - a directory clone, `src/adapters/filesystem_store_admission.rs#157@fb355d66`
  - an lstat, `src/adapters/catalog_restart_loader.rs#45@fb355d66`
  - an open and an fstat, `src/adapters/catalog_restart_io.rs#27,#30@fb355d66`
  - the read, then a second read that probes for trailing bytes, `src/adapters/catalog_restart_io.rs#67,#81@fb355d66`
  - on macOS, a further lstat and full root scan for each of the lstat and the open, `src/adapters/filesystem_entry_spelling.rs#19,#24@fb355d66`
- **Compared versus served, reader: holds, but not because of the fence.** The compared object is the served object (`src/adapters/retention/filesystem_retention_snapshot.rs#176-178,#184@fb355d66`). The fence does not freeze `HEAD`, since publication proceeds beside it (`src/adapters/retention/reader_fence.rs#22-23@fb355d66`). Double collection and immutable retained bytes supply the guarantee.
- **Compared versus served, writer: only by exclusion.** The comparison "neither opens nor verifies" the catalog (`src/adapters/catalog_restart_loader.rs#41@fb355d66`).
  - `StoreExpectation` is never rechecked. Later publication re-reads `HEAD` and checks a different expectation, one the caller supplies (`src/adapters/filesystem_catalog_current.rs#24-35@fb355d66`).
  - Agreement between the two reads rests on advisory locks (`src/adapters/filesystem_writer_lock.rs#49-50@fb355d66`).
- **"The reader" is one reader among several.** The type's doc says "Every open therefore requires this choice… no caller loses the check by omission" (`src/adapters/store_expectation.rs#39-40@fb355d66`). These opens contradict it:
  - `src/adapters/filesystem_catalog_snapshot.rs#42@fb355d66` takes no expectation, and its doc at `#31-32` says no lock and no fence
  - `src/adapters/filesystem_recovery_admission.rs#41@fb355d66`
  - `src/adapters/filesystem_recovery_inventory_reader.rs#54@fb355d66`
  - `src/adapters/retention/filesystem_retention_verification.rs#43@fb355d66`, which hard-codes `NoExpectation`

## Row 4: "Every retired ID is in the ledger; comm residual is empty": WEAKENED

The arithmetic holds. I excluded the ledger from "live", because leaving it in would make the check vacuous.

```sh
P='KEEP-[A-Z]+-[0-9]+'
git grep -hoE "$P" 27d5c38c -- docs/formats conformance xtask src tests | sort -u > base.txt   # 174
git grep -hoE "$P" fb355d66 -- docs/formats conformance xtask src tests \
  ':!docs/formats/store/retired-identifiers.md' | sort -u > live.txt                           # 181
git show fb355d66:docs/formats/store/retired-identifiers.md | grep -oE "$P" | sort -u > ledger.txt   # 61
comm -23 base.txt live.txt | comm -23 - ledger.txt
```

- **Residual: empty.** 29 IDs left the live set: `KEEP-CRASH-032`–`034`, `-054`–`-073`, and `KEEP-MIGRATION-003`–`008`.
- **Stricter form: also empty.** I counted only table rows as ledger entries (38 rows), and subtracted both a mention-based live set and a head set of defining rows. The only survivors of the mention-based form against that head set were `KEEP-CRASH-000`, a test string, and `KEEP-RECOVERY-001`, which is still live.
- **Reuse of a ledgered ID: none.** No ledger-row ID is defined outside the ledger at head. Mentions in `conformance` and `xtask/tests` are retirement prose (`xtask/tests/segment_store_protocol_contract/transition_laws.rs#56@fb355d66`).

**Where the row is narrower than it reads:**
1. **The scope hides an ID collision.** The comm covers neither `ROADMAP.md`, `docs/audits`, nor non-main history.
   - At `57b6c37`, `conformance` defines `KEEP-CRASH-074` as `gc write-intent-stage` and `-081` as `gc sync-segment-pool` (see also `xtask/src/durability_crash_point_identity.rs#83,#90@57b6c37`).
   - At head, `-074` means segment link (`docs/formats/store/catalog-publication.md#169@fb355d66`). `-081` means the initialization writer lock (`docs/formats/store/retired-identifiers.md#81@fb355d66`).
   - The roadmap still binds them to GC and still says "no `KEEP-CRASH-074` or later point exists" on main (`ROADMAP.md#1249,#1416,#1422@fb355d66`), which is now false. `docs/audits/pr-107-salvage.md#62@27d5c38c` shows the same binding. The roadmap also records salvage as tracked and in progress (`ROADMAP.md#1251-1252@fb355d66`), so landing the GC rows would collide.
   - These IDs were never on main or retired, so this is not reuse of a retired ID in the ledger's sense. It is the same harm under a technicality.
2. **The ledger misdescribes its own evidence column.** It promises "the last commit on `main` that carried the identifier's evidence" (`docs/formats/store/retired-identifiers.md#7@fb355d66`).
   - `KEEP-MIGRATION-001` cites `tests/store_migration_recovery.rs` at `a58f237` (`#26`).
   - That file still exists at base `27d5c38c`, which is an ancestor of `origin/main`.
   - `a58f2374` is only the file's last-modifying commit. I spot-checked this one row and five sibling paths that also exist at base, and did not audit every row.
3. **The ledger contradicts itself on `-078`.** It is "reserved… not assigned" at `docs/formats/store/retired-identifiers.md#83@fb355d66` and "retired" at `#164`. It has no table row, and the comm cannot see it because it never appeared at base in scope.

**Not reuse, but noted:** `KEEP-RECOVERY-001` narrowed from `-035` to `-030`, and `KEEP-RECOVERY-007` and `-021` were reworded (`docs/formats/segment-store-v1/requirements.md#118@27d5c38c`, `docs/formats/store/catalog-protocol-requirements.md#121@fb355d66`).

## Row 5: "#125 gate, recovery route": WEAKENED

**The code claim holds.**
- `verify` runs the gate (`src/adapters/filesystem_recovery_next_head_candidate.rs#41@fb355d66`).
- `replace_head` enters `verify_current` (`src/adapters/filesystem_recovery_next_head_finalization_storage.rs#53@fb355d66`), which runs `verify` on `Ready` (`#28-29`).
- Only an evidence re-fingerprint separates the gate from the rename (`src/adapters/filesystem_recovery_next_head_candidate.rs#42@fb355d66`, then `src/adapters/filesystem_recovery_next_head_finalization_storage.rs#56@fb355d66`).
- That is the only recovery rename of `HEAD`. The other production renames are the forward route and the retention stage (`src/adapters/filesystem_catalog_head.rs#78@fb355d66`, `src/adapters/retention/filesystem_retention_stage.rs#157@fb355d66`).

**The evidence is thinner than the sentence:**
1. **The cited test proves only the first entry.** It refuses inside `Verify` (`tests/catalog_writer/finalization_laws.rs#93@fb355d66`), which is the executor's first call (`src/adapters/recovery/recovery_next_head_finalization_executor.rs#31-36@fb355d66`). `replace_head` at `#45` is never reached.
2. **That is the only test naming the refusal.** `RetainedClosure` appears in no other test I could find (`tests/catalog_writer/finalization_laws.rs#98@fb355d66`).
3. **The replace-boundary test stops short of the gate.** It fails on `EvidenceMismatch` from the first evidence check, `src/adapters/filesystem_recovery_next_head_finalization_tests/refusal_laws.rs#48-66@fb355d66`, before line 41 runs.
4. **No crash evidence exists yet.** `KEEP-CRASH-080`, "the gate reruns on retry", is `Planned` (`docs/formats/store/catalog-publication.md#175@fb355d66`).
5. **The gate covers only the `Ready` branch.** `AlreadyFinalized` skips it and still issues a receipt (`src/adapters/filesystem_recovery_next_head_finalization_storage.rs#31-37@fb355d66`, `src/adapters/recovery/recovery_next_head_finalization_executor.rs#53-54@fb355d66`). `HEAD` does not change there, so this is not a bypass, but "verify runs the gate" is not unconditional.

**CONJECTURE:** the re-entry at replacement is untested.
*Experiment:* change line 53 to check only stage evidence and run the suite. Separately, use a storage wrapper to swap the retention head between `synchronize_candidate` and `replace_head`, and assert `RetainedClosure`. A surviving mutant would show the second gate has no test holding it in place.

## VERDICT

| Row | Claim | Ruling | How it narrows |
|---|---|---|---|
| 1 | One commit point, no admission before it | **STANDS** | Every gate holds. Blemishes are a loose cited test and the contradiction at `docs/formats/store/retired-identifiers.md#69@fb355d66` |
| 2 | No exact-name bypass | **WEAKENED** | Coverage holds, but off macOS the check is a no-op. Its casefold justification is absent on public `FilesystemCatalogSnapshot::load` and `try_acquire`, and on nested namespace directories |
| 3 | `NoExpectation` no I/O; `Catalog` one read | **WEAKENED** | Reader adds no read; writer adds several operations. Only double-collection protects the reader, and exclusion the writer. Several opens take no expectation, against `src/adapters/store_expectation.rs#39-40@fb355d66` |
| 4 | Every retired ID in ledger; residual empty | **WEAKENED** | Residual is empty and no ledgered ID is reused. The scope hides a `-074`..`-087` collision with GC, the ledger's "last commit" definition is false for the row checked, and it contradicts itself on `-078` |
| 5 | Gate on verify and replace | **WEAKENED** | Code is correct, but the test witnesses only the first entry. Re-entry at replacement has no test, and its crash row is `Planned` |
