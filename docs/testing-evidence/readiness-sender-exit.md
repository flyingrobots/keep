# Subprocess readiness across sender exit

## Claim and subject

Issue #153 owns elimination of a false readiness refusal when a sender exits after queuing the specified signal. This is calibration of test support, not Keep product acceptance. The repository owner owns policy and the author supplies these receipts. The contract is the subprocess readiness socket protocol, whose independent oracle admits only byte `r`; child exit alone cannot erase a previously written signal.

## Counterexample and replay

Hosted run `36991417726`, PR #99 head `8686931`, failed release `cleanup_terminates_the_entire_child_process_group` with `UnexpectedEof: child exited before descendant readiness: exit status: 0`. The original artifact is retained; later successful runs do not replace it or establish that the race vanished.

Regression commit `ff3d239`, branched from main `051d9bb`, extracts the previous observer without changing its decision and supplies a lifecycle observation port. In Docker, `queued_readiness_survives_exit_between_observations` executed its named assertion and failed with observed `Err(UnexpectedEof)` versus required `Ok(())`.

The deterministic schedule is: child connects to the gate and blocks; observer finds no readiness connection; lifecycle callback releases the child; child writes `r` to the readiness socket and exits; callback observes that actual exit; observer must admit the queued signal. The gate and socket are owned by this experiment. There is no random seed; the socket exchange is the recorded replay schedule. This short schedule is already the permanent reduced counterexample.

Replay in a copied Docker checkout of `ff3d239` with pinned Rust `1.96.0`: `cargo test -p xtask --bin xtask bounded_process::process_group::readiness_tests::queued_readiness_survives_exit_between_observations -- --exact --nocapture`. The fixed source is this PR's readiness module; the published GREEN head is recorded in its PR body. Unfixed, fixed, and mutated source copies use independent build directories when source roots differ.

An initial Docker attempt omitted tracked files beneath `xtask/src/fuzz_campaign/target` while excluding build artifacts and failed during formatting. This was setup failure, excluded from RED evidence. Source transfer now uses Git's cached and untracked nonignored inventory, preserving legitimate directories named `target`; the owned container imports the main Git history from a bundle rather than referencing host metadata.

## Execution and limits

The calibration and existing real process-group cleanup, inherited-pipe deadline, and interrupt laws run in debug and release under pinned Rust on Linux aarch64 in an owned Docker source copy on ext4. The test size is medium: filesystem scratch, local Unix sockets, and child processes are required. Controlled subprocesses receive an explicit environment, and storage identity or network services are irrelevant. No writable host repository mount or host Rust test execution is used.

Ordinary Cargo tests do not yet enforce individual size-class time or memory ceilings or a measured suite latency SLO; [the enforcement profile](../testing/enforcement.md) records that repository gap. The container and outer command lifecycle are not claimed as per-test resource admission. No waiver is self-approved. These experiments claim the stated schedule and protocol outcomes, not full resource-policy compliance or physical crash durability.

`an_invalid_readiness_byte_refuses` exhausts the byte domain excluding `r` in ascending order and requires `InvalidData`; its first failure is a minimal byte counterexample. `readiness_refuses_child_exit_without_a_signal` retains `UnexpectedEof`. The valid queued-signal assertion is calibrated by the parent RED above. Separate deliberate signal-admission mutations calibrate the invalid-byte oracle; their actual results and the final GREEN validation are recorded below before publication.

The deliberate receiver mutation admitted `x` instead of `r`. The invalid-domain law failed its named check at byte `120` (`x`), demonstrating that acceptance of a forbidden signal cannot pass this oracle. The source-copy mutation and fixed checkout used separate build directories; neither compilation nor fixture setup was accepted as falsification evidence.

The corrected source passed the focused process-group laws and the complete all-feature workspace suites in debug and release, including descendant cleanup, inherited-pipe deadline, and terminal-interrupt outcomes. Both Clippy configurations with warnings denied, source structure, Rust formatting, and Markdown passed. The source-structure check admitted the new readiness module through the owned checkout's index; hosted validation of the published head remains a separate gate.

## Deletion and blind spots

The `readiness_wait_does_not_depend_on_wall_clock` source-substring assertion was removed under Rule 18: it checked a spelling rather than a runtime contract and did not prove a deadline. No required runtime assertion was weakened. [The readiness rationale](../../xtask/src/bounded_process/process_group/rationale.md) records the displaced claim and remaining process-group evidence.

Only the controlled observation race and signal classes above are claimed. Arbitrary socket peers, a connected peer that never sends a byte, all operating-system schedules, and non-Linux platforms remain unverified. The child fixture itself carries no storage assertion; the parent calibration owns the verdict. No benchmark, format, identity, public API, migration, or production process-group policy changes.
