# Single-format mutation record

Change kind: evidence calibration for #198, on PR #220. Owner: `@flyingrobots`. Six source mutants were run by hand to find out whether named laws fail when the behavior they describe is removed. No mutant was committed. This is not a mutation score: the mutants were chosen because a review predicted they would survive.

## Why these mutants

The review of PR #220 at `fb355d66` predicted that three laws passed for a reason other than the one they state. Running the prediction was cheaper than arguing it. The first round ran against `fb355d66`. The second round ran against the repaired tree, to check that the repairs fail on their stated cause.

## First round, against `fb355d66`

All three mutants were applied together and the complete root-package suite ran once, natively on default APFS with Rust 1.96.0: `cargo test --all-features --locked --offline --no-fail-fast`. The mutants touch separate subsystems, so one failing law identifies its mutant. Result: 936 passed, 1 failed.

| Mutant | Outcome | What it showed |
| --- | --- | --- |
| `initialization_entries::synchronize_root` returns `Ok(())` without synchronizing | Survived | No law observed any root synchronization during initialization, including the commit point. The law then named for a failed commit synchronization failed its observer before the synchronization ran. |
| `replace_head` in `filesystem_recovery_next_head_finalization_storage.rs` renames without calling `verify_current` again | Killed by `changed_candidate_is_refused_at_the_replacement_boundary` | The gate's re-entry at replacement is held in place. The killing law changes the candidate. No law changes the retention head between `synchronize_candidate` and `replace_head`. |
| The lock replacement removed from `a_replaced_lock_or_protocol_directory_during_an_operation_still_refuses` | Survived | The second writer is refused as `Busy` by the root-inode lock, which the first writer holds whether or not `writer.lock` is replaced. The assertion cannot tell the two cases apart. |

## Second round, against the repaired tree

All three were applied together and `cargo test --all-features --locked --offline --no-fail-fast --lib --test single_format --test catalog_restart` ran once, natively on default APFS. Result: library 362 passed and 2 failed; `single_format` 15 passed and 5 failed; `catalog_restart` 10 passed.

| Mutant | Outcome | Killing laws |
| --- | --- | --- |
| `Execution::synchronize` skips the root synchronizer and keeps both observer notifications | Killed | `the_commit_synchronization_runs_after_the_link_and_before_stage_removal` and `a_failed_commit_synchronization_leaves_an_unadmitted_resumable_store`, and no other law |
| `acquire_root_lock` takes a shared lock, so the root lock excludes nobody | Killed | `a_replaced_lock_or_protocol_directory_during_an_operation_still_refuses`, and no other law |
| `catalog_restart_loader::load` ignores its expectation | Killed | `a_store_with_no_catalog_refuses_an_expected_catalog`, `a_different_valid_store_refuses_the_expected_catalog`, `an_older_snapshot_restored_in_place_refuses_newer_expected_state`, and `an_expectation_rebuilt_from_recorded_values_admits_and_refuses_as_the_original` |

The third mutant was also run alone against the crash matrix, because the matrix is silent on success: `cargo xtask durability-crash-matrix --sequence retained-catalog` stopped at its first case with "`KEEP-CRASH-074` before: post-crash verification failed while attempting to refuse the recorded generation-one catalog: generation two admitted a generation-one expectation". So the recorded-catalog check in that sequence runs and can fail.

The second row answers the first round's third row. With the lock replacement in place, the portability law does fail when the mechanism that makes a replaced `writer.lock` harmless is removed. It is also the only law in the suite that failed for a non-exclusive root lock: the in-crate writer-lock laws passed.

## What this does not establish

- The first round's first mutant still survives. The commit laws substitute the root synchronizer, so they witness that the protocol requests the synchronization, in order, and what a failed request leaves. That `filesystem_catalog_artifact::synchronize_directory` reaches stable storage is established by reading it. Process death cannot expose a missing synchronization, because the page cache survives it, and no power-loss test exists.
- The crash matrix was run against one mutant only, as described above.
- The retained-closure check on the gate's second entry has no law of its own. It shares one function with the candidate check that the killing law exercises.
- Only default APFS was used. The mutated code is not platform-specific, but that is an argument, not a run.
- Six hand-picked mutants say nothing about the rest of the suite.

## Replay

Apply one mutant to a clean tree, run the command for its round under `scripts/native/check.sh`'s bounds, and restore the file. The laws live in `src/adapters/store_initialization/initialization_commit_tests.rs`, `tests/single_format/portability_laws.rs`, `tests/single_format/expectation_laws.rs` and `src/adapters/filesystem_recovery_next_head_finalization_tests/refusal_laws.rs`.
