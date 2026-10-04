# Canonical head-length inputs

Change kind: test-maintenance refactoring. Owner: `@flyingrobots`. Subject: generated inputs for recovery's exact head-to-manifest length contract (#99).

At parent `5cabf82`, the finite length sweep repeated the manifest entry width as a literal `72`.

The sweep now derives its minimum and positive entry stride from the public encoder's empty and single-entry manifest outputs, then generates lengths through the manifest's admitted entry-count bound.

Only input construction changes: the specified oracle still requires recovery precisely when the head length equals the frozen manifest fixture's actual byte length, and the exact `HeadStageNamesOtherManifest` refusal otherwise.

The encoder supplies inputs, not expected outcomes, so no private codec constant is exposed for testing and the planner cannot validate its own answer through the generator.

This relies on the version-two protocol's fixed-width entry representation; it is not a generator for arbitrary future variable-width formats.

Removing the production planner's head-length comparison in a disposable copied tree makes the revised law fail at the first mismatched length: head length 224 must refuse the 296-byte fixture.

The unchanged matching-length assertion passes on production, while the mismatched-length assertion rejects that mutation; no harness count assertion is introduced.

The focused sweep passes in Docker debug and release, with both workspace Clippy configurations, formatting, and source-structure checks.

Replay uses `cargo test --lib recovery_binds_head_length --all-features`, adding `--release` for optimized execution; calibration uses a fresh build target.
