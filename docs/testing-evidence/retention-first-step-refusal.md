# First-step recovery refusal

Change kind: test coverage repair. Owner: `@flyingrobots`. Subject: the public recovery executor's first-refusal contract (#99).

At parent `9712be6`, the refusal law exercised a failure after successful finalization, leaving refusal of finalization itself uncovered.

The added law passes a finalization plan to `execute_retention_recovery` and injects refusal at the public storage port's first capability.

The returned error must name `FinalizeHead`, report an empty completed prefix, and leave later cleanup capabilities unexecuted.

The recording adapter observes successful effects across the storage port; this is a test of the product executor's contract, not a test of fixture cardinality or an internal call sequence.

Three disposable production mutations independently misname the refused step, include that refused step in the completed prefix, and invoke root-stage cleanup after refusal.

Each mutation fails its corresponding named assertion with a fresh build target and the same unchanged adapter.

The executor suite passes in copied Docker debug and release with `cargo test --lib recovery_execution_tests --all-features`, adding `--release` for optimized execution.

Both workspace Clippy configurations with `-D warnings`, formatting, and source-structure checks pass.

Production already satisfies the contract; no failing-parent bug reproduction is claimed.

This small test establishes port-level execution behavior only; the fake does not model filesystem durability or partial effects within a failed storage capability, which remain the responsibility of filesystem and fault-injection evidence.
