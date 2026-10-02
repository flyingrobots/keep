# Interrupted root and manifest history

Change kind: bug fix. Owner: `@flyingrobots`. Subject: preservation of interrupted root and manifest records with contradictory complete history fields (#99).

Regression `db7e398` is observed RED on unfixed production `f475211`: an interrupted generation-one root naming a predecessor is discarded as clean recovery.

The filesystem law covers both root and manifest initial-with-predecessor and successor-without-predecessor contradictions, requiring exact semantic causes and unchanged retained-file witnesses.

Domain predecessor admission is now crate-visible through each record type and shared by complete construction and prefix admission; it neither requires nor allocates unavailable body entries.

Complete decoding retains its existing integrity checks before semantic construction.

Three disposable product mutations replace the history cause with overflow, misname the corrupt stage, and delete the root stage while returning the correct error; each fails the corresponding runtime assertion.

History-filtered library suites and canonical-prefix integration tests pass in copied Docker debug and release, and the retention process-death matrix passes.

Both workspace Clippy configurations with `-D warnings`, formatting, and source-structure checks pass.

Replay uses `cargo test --lib history --all-features` and `cargo test --test retention_stage_prefix --all-features`, adding `--release` for optimized execution; calibration uses separate copied trees and fresh targets.

This closes complete root and manifest history fields only; root profile/limit fields, partial fields, record bodies and integrity prefixes, and incomplete-stage pinning remain under audit.
