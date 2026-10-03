# Filesystem corruption refusal

Change kind: test coverage repair. Owner: `@flyingrobots`. Subject: refusal and evidence preservation when a complete retained root stage has a damaged checksum (#99).

At parent `c3fca1c`, short corrupt prefixes were covered through filesystem recovery, but the complete checksum-corrupt root stage had no corresponding filesystem recovery law.

The new law writes the golden root as `root.next` with one checksum bit flipped, then invokes the public filesystem authority's recovery method.

Its specified oracle requires `FilesystemRetentionRecoveryError::Plan` containing `StageCorrupt` for the root stage, with the exact `RetentionRootDecodeError::ChecksumMismatch` expected and observed checksum bytes taken from the original and damaged fixtures.

The complete retained-file witness must remain byte-identical after refusal, including the damaged stage.

No assertion reaches into observation or classification internals: the typed public refusal and retained bytes establish the behavior through the real filesystem observation, decoding, planning, and refusal path.

Production already satisfies this behavior; the change adds missing runtime evidence rather than claiming a production bug fix or a failing parent reproduction.

Disposable production mutations independently mislabel the corrupt root as a head, swap the checksum coordinates, and delete `root.next` on the planning-error path.

The diagnostic assertion rejects both incorrect error mutations; the retained-byte assertion rejects deletion while the correct refusal is still returned.

Each mutation runs in a separate copied Docker tree with a fresh build target.

The filesystem recovery suite passes in copied Docker debug and release using `cargo test --lib filesystem_retention_recovery_tests --all-features`, adding `--release` for optimized execution.

Both workspace Clippy configurations with `-D warnings`, formatting, and source-structure checks pass.

The calibration replay filter is `checksum_corrupt_root_stage_is_refused_without_changing_evidence`; this law covers complete root-stage checksum damage, not every corruption class or mid-recovery filesystem fault.
