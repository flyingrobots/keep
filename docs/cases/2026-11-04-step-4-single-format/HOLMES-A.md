# Testimony of Holmes, Scope A: PR #220, `integrate/open-prs`, at `fb355d66`

Watson, the central claim holds. Every production lookup of a protocol name passes through the exact-name check, and I find no bypass. #216's macOS laws survived the merge. The prose around them has not kept pace: one stack law stayed Linux-only without being declared, one evidence count is stale, and the platform documents still describe the pre-merge design.

Method: read-only static enumeration, plus `gh` read-only queries of CI run 41953913643. That run's `headSha` is `fb355d664ab8…` and its conclusion is `success`. I ran no cargo or build, and made no repository edits or commits. Citations without a revision are at `@fb355d66`.

## I. Exact-name coverage: every raw directory in `src/`

**Claim:** "all store code uses `FilesystemDirectory`" (PR body, Approach). The wrapper checks every named operation at `src/adapters/filesystem_directory.rs#57-106` through `require_exact` at `#108-125`. Nested paths refuse at `#113-122`.

Alias census: 37 production files import `FilesystemDirectory as Dir`. Two files that imported it at base no longer do, `filesystem_catalog_publisher.rs` and `filesystem_recovery_stage_discarder.rs`. A search of both finds no `Dir`, `cap_std` or `open_with`, so they no longer touch a directory.

| Production site | Use | Ruling |
| --- | --- | --- |
| `src/adapters/filesystem_directory.rs#14,#19` | The wrapper's `RawDirectory` | Checked; it is the checkpoint |
| `src/adapters/filesystem_entry_spelling.rs#5,#19,#24` | Raw `Dir` for `symlink_metadata` and `entries` | Exempt; it is the check, and must not recurse |
| `src/adapters/platform_filesystem.rs#8,#14,#17` | Trait signature only | Exempt; no lookup |
| `src/adapters/darwin_filesystem.rs#21-34` | `openat` per path component | Exempt; a caller-supplied root path, not a protocol name, no-follow at `#64` |
| `src/adapters/darwin_filesystem.rs#42,#53` | `fstatfs`, `dir_metadata` | Exempt; descriptor-bound, no name |
| `src/adapters/linux_filesystem.rs#15,#58` | `openat2` on the root, `statx(".")` | Exempt; a root path and no entry name. The check is a Linux no-op by design, `filesystem_entry_spelling.rs#37-47` |
| `src/adapters/unsupported_filesystem.rs#5` | Refusal stubs | Exempt |
| `src/adapters/filesystem_platform_profile.rs#43,#53,#60` | `from_raw`, `.raw()` | Exempt; these are the only `.raw()` and `from_raw` callers, and they feed `admit_directory`, which does no name lookup |
| `src/adapters/filesystem_writer_lock.rs#138-140` | `openat(directory, ".")` | Exempt; `.` has no entry to spell |
| `src/adapters/sync_capable_directory.rs#18,#25` | `from_std_file` | Checked; the descriptor comes from the checked `open_with` |
| `src/adapters/filesystem_catalog_artifact.rs#30`, `filesystem_platform_profile.rs#63` | `into_std_file` | Exempt; `sync_all` only |
| `src/adapters/filesystem_writer_lock.rs#75`, `catalog_restart_io.rs#14` | `open_ambient_dir` on the root | No spelling bypass, because later lookups are checked. They do skip the platform profile; see drift 4 |
| `filesystem_store_admission.rs#143,#194`, `filesystem_recovery_admission.rs#60`, `filesystem_recovery_inventory_reader.rs#77` | `open_ambient_dir` | Exempt; gated `cfg(any(test, feature = "repository-tasks"))` |
| `entries()` callers, such as `retention/filesystem_retention_namespace.rs#88,#167` | `DirEntry::file_type`, `file_name` | Safe; names come from the listing, and no `DirEntry::open` or `remove` exists in production |

**Verdict: no bypasses.** One caveat follows from `#62-63`: the check and the operation are two steps, not one atomic step. "Never accepts" therefore holds within the disclaimer at `README.md#84`, which excludes out-of-band mutation. A rename during the scan fails closed; the ADR records that race at `docs/adr/filesystem-platform-adapters.md#92-95` and its stress law exists at `src/adapters/darwin_filesystem_tests.rs#153`.

## II. Root and platform claims

- **CONFIRMED: replacement type.** `FilesystemRootIdentity` held device, mount and file (`git show 27d5c38c:src/adapters/filesystem_root_identity.rs#4-8`). The new type holds device and mount only (`src/adapters/filesystem_directory_coordinate.rs#9-12`).
- **CONFIRMED: never persisted.** The type is `pub(super)` (`#9`). Its only non-constructor consumer is `admit_child_identity` (`filesystem_platform_profile.rs#80-103`).
- **CONFIRMED: Darwin domain removed.** A search for `DARWIN_COORDINATE_DOMAIN` and `restart_stable_device` across `src`, `tests` and `xtask` returns nothing. The device is now plain `st_dev` (`darwin_filesystem.rs#58-59`).
- **CONFIRMED: lenient Linux mount policy removed.** `MountIdentityPolicy` and `root_identity_lenient` are deleted in the diff. `linux_filesystem.rs#57-62` refuses a missing `MNT_ID` on every path. `filesystem_platform_profile_policy_tests` is gone.
- **CONFIRMED: nine directories and native selection.** `filesystem_platform_profile.rs#17-27,#34-36`, `native_filesystem.rs#3-10`.
- **CONFIRMED: one-at-a-time nested traversal.** `store_initialization/initialization_observation.rs#135-151`, `filesystem_platform_profile.rs#66-78`.
- **UNVERIFIABLE: "which macOS's native tests found."** The tree holds no receipt. The single-component refusal is not macOS-gated (`filesystem_directory.rs#113-122`), so conjecture says Linux would have caught it as well.

## III. Test gates and what #216 merged

**Widened at the merge, from Linux-only at stack tip `ee5d60ae` to Linux and macOS at head:**
- `tests/catalog_restart.rs#3`
- `tests/catalog_writer.rs#7`
- `tests/single_format.rs#8`
- `tests/durable_locator.rs#8`
- `tests/segment_filesystem_stage.rs#5`
- `tests/golden_file_worldline.rs#8` and `golden_file_worldline/suite.rs`
- `tests/reader_fence_process.rs#4`
- `tests/recovery_inventory.rs`
- `tests/observed_segment_stage.rs` (its stack-tip gate named Linux; its head gate is no longer matched by my single-line search, but I did not read the multi-line head gate)
- `src/adapters/retention/filesystem_store_admission_tests.rs#76,#97`

A catch in the narrative: `tests/catalog_restart.rs` had no OS gate at base. For that file the merge narrows from "any OS" to Linux and macOS. This is harmless, since no other platform has an adapter.

**Still Linux-only:**
- `reader_platform_law_tests` (`retention.rs#200`) and `filesystem_retention_fifo_tests` (`retention.rs#43`)
- O_PATH sync (`filesystem_catalog_artifact.rs#145`)
- Raw names (`filesystem_recovery_inventory_tests.rs#50`)
- tmpfs laws (`tests/single_format/suite.rs#43`, `initialization_laws.rs#47`)
- `filesystem_catalog_recovery_tests` (`src/adapters/mod.rs#91-92`); see drift 1

**macOS keeps #216's laws: CONFIRMED.** I compared functions added by #216 against head. Only these names are gone:
- Renamed: `apfs_root_coordinates_bind_the_opened_directory`, `a_reattached_volume_reopens_its_migrated_store`.
- Deleted by design: `a_renumbered_volume_keeps_its_restart_coordinate_…`, `a_distinct_child_inode_on_the_same_mount_is_admitted`.
- Helpers: `watchdog_shell`, `linux_root_identity`, `linux_file_identity`.

The CI log shows the spelling, FIFO, Finder and reattach laws passing natively. The native suite gained three direct laws (`darwin_filesystem_tests.rs#223,#240,#256`) and a public initialization law (`tests/darwin_store_spelling.rs`).

One genuine net loss, disclosed in the ledger at `docs/formats/store/retired-identifiers.md#90`: `tests/migration_descriptor_exhaustion.rs` left no successor exhausting descriptors at the root-clone boundary. A search for `root_clone|descriptor_exhaust|ulimit -n|RLIMIT` in `src`, `tests` and `xtask/src` returns nothing, while `filesystem_store_admission.rs` still has a clone-failure branch in `require_expectation`.

**PR test table: CONFIRMED, better than local.** The CI log at this head gives 937 passed, 0 failed for default debug, default release and case-sensitive debug, and the reattach step shows 1 passed. The Linux gates passing is consistent with the "9 environmental" xtask failures, though each diagnosis stays unverified. Each of those 937 runs includes one inert reattach pass, since the law returns `Ok(())` without its environment (`tests/darwin_volume_reattach.rs#72-79`). The file header discloses this.

**RED calibration: UNVERIFIABLE.** The tree has no receipt. `docs/testing-evidence/filesystem-platform-adapters.md#44-48` records only the older calibration.

## IV. Drift findings

**1. Undeclared Linux-only stack law (most material).**
- Claim: the evidence section "Laws that remain Linux-only, and why" (`docs/testing-evidence/filesystem-platform-adapters.md#69-78`), and merge `5360a804`'s "Platform-neutral laws from the stack also run on macOS."
- Evidence: `filesystem_catalog_recovery_tests` is gated at `src/adapters/mod.rs#91-92`. It appears 0 times in the macOS log and 6 times in the Linux log. A search of the module and two of its fixtures for `linux|ext4|statx|/proc|O_PATH` finds nothing, and `KEEP-RETENTION-013` cites it at `docs/formats/store/requirements.md#24`.
- Conjecture: an omission rather than a decision. I cannot say it would pass on macOS.
- Mitigation: `tests/catalog_writer.rs` and the crash matrix do run on macOS.

**2. Stale evidence count.** `filesystem-platform-adapters.md#22` says 963 passed on both variants. CI at this head and the PR body both say 937. This PR edited that document and left the row, which carries no commit pin.

**3. Evidence cites a deleted law.** `filesystem-platform-adapters.md#71` names `a_writer_lock_cannot_mint_...` as Linux-only. The law is gone with `tests/catalog_platform_admission.rs`. Its successor is a compile-fail doctest (`src/adapters/filesystem_catalog_publisher.rs#61-65`), which is stronger and all-platform, but the document does not say so.

**4. "Every store path goes through the native platform adapter."**
- Claim: merge `5360a804`'s message (not the PR body), and `docs/architecture/filesystem-platforms.md#105,#114,#154-155`.
- Evidence: public, ungated `FilesystemCatalogSnapshot::load` (`filesystem_catalog_snapshot.rs#42`, `catalog_restart_loader.rs#28`, `catalog_restart_io.rs#14`), `load_for_verification` (`verification_ingress.rs#59-63`) and `FilesystemWriterLock::try_acquire` (`filesystem_writer_lock.rs#73-75`) all open ambiently. They take no profile check.
- The dependency source confirms no `NOFOLLOW` flag is set (`~/.cargo/registry/.../cap-primitives-4.0.3/src/rustix/fs/dir_utils.rs#111-118`).
- Pre-existing at base. The narrower `stage-recovery.md#404-411` carries the same over-claim on "symlinked selected paths."
- Exact-name still holds on these routes.

**5. Platform interface described as larger than it is.** `filesystem-platforms.md#39,#76-84` assigns the interface durability, locking, naming behaviour and "device/mount/object coordinates." The trait has two methods (`platform_filesystem.rs#12-18`), and the coordinate has no object field. `#98` and ADR `#69-71` contradict the same table. The object coordinate became false in this PR; the rest is older.

**6. "Golden fixtures remain stable."** `filesystem-platforms.md#185` and ADR `#82` remain in the present tense. Yet `conformance/segment-store/v2/format-marker.hex` is modified, the migration hex fixtures are deleted, and the PR body concedes the format change. The evidence document was narrowed correctly (`#13`); these two were not.

**7. Understated native evidence.** `filesystem-platform-adapters.md#72-73,#119` say non-APFS refusal is only at the adapter, with the public boundary "not measured." `tests/single_format/initialization_laws.rs#64-…` calls public `initialize("/dev")`, and it passes in the macOS log.

**8. ROADMAP T-41.1 is ticked on a retired criterion.** `ROADMAP.md#2403,#2409-2410` requires `KEEP-RECOVERY-003` to gain a macOS row and names a "105-case matrix." That ID is retired (`docs/formats/store/retired-identifiers.md#131`), and a search for macOS or Darwin in both requirement ledgers returns nothing. `#2405` also asks for capability probing, while `darwin_filesystem.rs#42-48` reads only type and flags.

**9. README contradicts itself.** `README.md#48` advertises a "Durable version-1 segment store." `#56-58` claims "`KEEP-CRASH-001`–`052`" for "version-1 publication." `#98` says version 1 is deleted, and `retired-identifiers.md#144-156` retires `-031` to `-035`.

**10. Three smaller stale items.**
- `docs/formats/store/stage-recovery.md#384` names three protocol directories; admission checks nine.
- `filesystem-platforms.md#235` lists "copied roots" among boundary failures, though `tests/single_format/portability_laws.rs#35` requires a copy to open. The wording is slightly ambiguous.
- `filesystem-platforms.md#249` labels a deleted v1 page, pinned at `3887f2a`, as the "Current filesystem contract."

**Confirmed documents:** the CI macOS job matches the evidence record (`.github/workflows/ci.yml#90-149`, `filesystem-platform-adapters.md#29-30`). The reattach law really forces renumbering (`tests/darwin_volume_reattach.rs#89-95`). `folds_case` enforces the volume variant (`darwin_filesystem_tests.rs#27-38`). `Cargo.toml` and `Cargo.lock` are unchanged in the diff. `scripts/native/check.sh` exists. The APFS sentence at `README.md#53` and the superseded status at `docs/adr/retention-reader-root-identity.md#3` are true.

## V. Reverse drift

- `ROADMAP.md` changes only two link lines in this PR. The labels at `#7-8` still read `segment-store-v1/…` while pointing into `store/`, and 74 lines still mention migration. T-41.1 is not alone.
- The PR does not mention losing the descriptor-exhaustion coverage, nor the gate narrowing on `tests/catalog_restart.rs`.
- The three-at-once CI proof of 937 is not in the PR body, which reports only local runs. That would strengthen it.

## VERDICT

| Claim | Verdict | Evidence |
| --- | --- | --- |
| All store code uses `FilesystemDirectory`; no exact-name bypass | CONFIRMED | Section I census |
| "Every store path" uses the native adapter (commit message) | DRIFT | `catalog_restart_io.rs#14`, `filesystem_writer_lock.rs#75` |
| Coordinate holds device and mount, never persisted | CONFIRMED | `filesystem_directory_coordinate.rs#9-12` |
| Darwin domain and lenient policy removed | CONFIRMED | `darwin_filesystem.rs#58-59`, `linux_filesystem.rs#57-62` |
| Nine directories, native selection | CONFIRMED | `filesystem_platform_profile.rs#17-36` |
| Nested traversal one entry at a time | CONFIRMED | `initialization_observation.rs#135-151` |
| "macOS's native tests found" it | UNVERIFIABLE | No receipt |
| #216's macOS laws survive | CONFIRMED | Function comparison, CI log |
| Platform-neutral laws run on macOS; Linux-only list is complete | DRIFT | `src/adapters/mod.rs#91-92`, evidence `#69-78` |
| 937 passed on both variants, crash matrix, Clippy | CONFIRMED | CI run 41953913643 |
| 9 xtask failures are environmental | CONFIRMED by outcome, diagnosis UNVERIFIABLE | Linux job passes |
| RED calibrations | UNVERIFIABLE | No receipt |
| Evidence doc count, 963 | DRIFT | `#22` against CI's 937 |
| Evidence doc law citation and public-boundary note | DRIFT (2) | `#71`, `#72-73,#119` |
| Architecture interface and coordinates | DRIFT | `filesystem-platforms.md#39,#76-84` against `platform_filesystem.rs#12-18` |
| Fixtures and bytes "stable" | DRIFT | `filesystem-platforms.md#185`, ADR `#82` |
| ROADMAP T-41.1 acceptance | DRIFT | `ROADMAP.md#2409-2410`, `retired-identifiers.md#131` |
| README v1 and crash claims | DRIFT | `README.md#48,#56-58,#98` |
| APFS sentence in README, CI macOS job, reattach law | CONFIRMED | See the confirmed-documents note |
| Three smaller stale items | DRIFT (3) | Item 10 |
| No power-loss claim on APFS; T-41.2 | PLANNED | `filesystem-platform-adapters.md#10`, `ROADMAP.md` T-41.2 |

- **Drift count:** 12 (9 numbered findings, with findings 7 and 10 split into their parts). Four are pre-existing at base: the ambient root opens, most of the architecture table, the T-41.1 acceptance text, and the capability-probing line.
- **Worst drift:** `filesystem_catalog_recovery_tests` is Linux-only with no stated reason and no entry in the evidence list that exists to declare exactly that.
- **Fidelity grade: B-.** The PR body is scrupulous, and its load-bearing safety claim survives a full census. The grade falls on the surrounding documents, which were edited in this PR and still say things the code and CI now contradict.
