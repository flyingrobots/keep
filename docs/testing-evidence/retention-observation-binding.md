# Shared retention observation binding

Change kind: refactoring with test-fixture admission repair. Owner: `@flyingrobots`. Subject: the validated relationship between an observed retention head and its selected manifest.

At parent `d118f27`, production observation checked the selected manifest's digest, generation and predecessor after reading exactly the length named by the head, while `ObservedRetentionState::for_tests` decoded its supplied records separately and could construct contradictory state.

Both paths now call the same private binding validator before constructing `ObservedRetentionState`. The validator requires the named length and the same digest, generation and predecessor checks; the production read retains its existing earlier exact-length check and error precedence.

The repair is established by static before/after constructor evidence and the single shared validation path. No test of a test fixture or source-text assertion is added. The existing planner scenarios continue to enter through their narrow semantic boundary, and existing filesystem observation tests exercise the real authority boundary.

Recovery planner and filesystem current-state laws pass unchanged in Docker debug and release, with both workspace Clippy feature configurations, formatting and source-structure checks. A copied-source calibration removes the shared predecessor check and runs the existing real-filesystem predecessor-disagreement law to verify that the production path still depends on that check.

Replay uses `cargo test --lib recovery_planner_tests --all-features` and `cargo test --lib filesystem_retention_current_tests --all-features`, adding `--release` for optimized runs. This is fixture-integrity and existing runtime regression evidence, not a claim of a newly reproduced production bug, new power-loss coverage or exhaustive input equivalence.
