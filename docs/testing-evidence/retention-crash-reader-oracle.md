# Reader-visible state after process death

Change kind: test-oracle repair. Owner: `@flyingrobots`. Subject: retention generation and selected-root bytes observed after a real publication child is killed (#99).

At parent `276f0e7`, the retention crash runner checked the recovery receipt and forward retry, without independently reading the recovered snapshot.

The restart verifier now loads `FilesystemRetentionSnapshot` after recovery and before retry, with a bounded catalog read policy.

The crash coordinate determines the expected state: before a complete head-stage write, no retention head or selected root may be visible; from completion of that write onward, recovery must expose generation one and exactly the golden publication input's root bytes.

The expected state does not come from the recovery receipt; the root decoder supplies only the golden input's namespace for the public read.

The reader snapshot is dropped before forward retry, so the new verification does not retain its fence across publication.

Calibration substitutes `wrong` for the public reader's verified returned root bytes and runs `cargo xtask durability-crash-matrix --case KEEP-CRASH-046 after` in a disposable copied Docker tree.

With that same production mutation, the parent verifier passes and the revised verifier fails at `verify recovered selected root bytes`, reporting expected and observed payloads.

A separate production mutation makes `retention_head()` return `None`; the revised verifier fails at `verify recovered retention generation`, reporting `expected Some(1), observed None` at the same crash coordinate.

Each calibration uses a fresh build target, and neither mutation enters the committed product code.

The complete retention process-death sequence passes in copied Docker with `cargo xtask durability-crash-matrix --sequence retention` and `cargo run --quiet --locked --release --package xtask -- durability-crash-matrix --sequence retention`.

Both workspace Clippy configurations with `-D warnings`, formatting, and the source-structure check pass.

This is product runtime evidence delivered by repository tooling: the assertions read Keep's public outputs after killing and reaping a real child, rather than asserting harness case counts or implementation structure.

The crash coordinate is the deterministic replay artifact; the sequence covers the runner's declared before/during/after positions for initial retention publication, not arbitrary schedules, successor publication, stacked faults, or power-loss behavior.
