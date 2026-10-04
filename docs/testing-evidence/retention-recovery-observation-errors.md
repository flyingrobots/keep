# Recovery observation error evidence

Change kind: bug fix with an intentional diagnostic API change. Owner: `@flyingrobots`. Oracle: publication identifies failure during recovery observation while preserving its original typed or operating-system cause.

On base `1c6bab2`, real filesystem regressions for an unknown retention entry and a symbolic-link root stage both failed because publication returned the observation cause directly inside `CurrentVerification`, omitting a recovery-observation boundary. RED commit `27f7259` records the new error vocabulary and regressions before production wiring.

The fix preserves `RecoveryObservationRefused` inside the existing publication error, with the original I/O error as its standard `Error::source`. The tests require that boundary, then read the original cause through the standard source chain: the exact `UnknownRetentionEntry` variant or the kernel's no-follow `ELOOP` code.

Existing current-state tests still assert their underlying specific refusals. Their shared extraction helper explicitly traverses the new observation layer; the new boundary tests inspect that layer directly, so the compatibility adaptation does not stand in for proof of the wrapper itself.

Copied-source mutations stringify the original observation cause or hide it from `Error::source`, calibrating cause fidelity and source-chain access separately from the original missing-boundary RED.

The first workspace run exhausted inode capacity during fixture setup; that run is not reported green. Completed audit copies were preserved on the separate build volume before revalidation, leaving the ext4 audit filesystem available for the capacity scenarios.

The next workspace run exposed an earlier descriptor regression test spawning a child inside the library test binary, violating the existing source-layout guard. Commit `3fe8ed3` moves that law into its own integration binary and retains the guard; restoring the wrong clone-error mapping still makes the moved runtime test fail. The prior hosted Rust failure had the same source-layout cause.

After those corrections, complete workspace all-feature debug and release suites passed. Both workspace Clippy feature configurations, formatting and source-structure checks passed. Both cause-erasure mutations failed the intended runtime checks; pinned Markdown lint passed separately.

Replay uses `cargo test --lib filesystem_retention_observation_error_tests --all-features`, adding `--release` for optimized execution. The symlink law is Unix-specific; this run uses copied Linux Docker on the ext4 audit filesystem. These tests establish diagnostic behavior, not new crash or power-loss coverage.
