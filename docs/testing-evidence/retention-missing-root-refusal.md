# Missing root stage diagnostic

Change kind: bug fix. Owner: `@flyingrobots`. Subject: exact filesystem recovery refusal when complete head and manifest stages remain without their root stage (#99).

The regression in `5eb3428` runs against unfixed production code from `6661736`: it creates a real migrated store, drives publication through head-stage synchronization, removes only `root.next`, then calls public filesystem recovery.

The expected `ManifestStageWithoutRootStage` assertion fails RED with the observed `Plan { source: HeadStageWithoutManifestStage }`, proving that recovery named an artifact that was present.

The planner now distinguishes a missing manifest from a complete manifest with a missing root, using the existing typed refusal variants.

The regression also compares the complete retained-file witness before and after refusal, so a precise diagnostic cannot hide deletion or modification of evidence.

A disposable production mutation deletes `head.next` on the planning-error path while returning the corrected refusal; the preservation assertion fails at `missing-root refusal must preserve all retained bytes`.

The recovery-filtered library suites pass in copied Docker debug and release with `cargo test --lib recovery --all-features`, adding `--release` for optimized execution.

Both workspace Clippy configurations with `-D warnings`, formatting, and source-structure checks pass.

This changes the diagnostic variant for one ambiguous state; it does not permit publication, change durable encoding, or weaken recovery's refusal-before-effects contract.
