# Interrupted head checksum admission

Change kind: bug fix. Owner: `@flyingrobots`. Subject: preserving interrupted heads with already contradictory checksum bytes (#99).

Regression `9decf08` is observed RED against unfixed production `c9c27b4`: a head ending after its first checksum byte, with that byte flipped, returns a successful `DiscardHeadStage` receipt.

Recovery now computes the checksum once its entire preimage is available and compares every checksum byte present in the interrupted record.

The filesystem regression flips the last available byte for each strict checksum prefix ending from 113 through 143 bytes, requiring the exact head corruption refusal, mismatch offset and byte values, and an unchanged retained-file witness.

Expected checksum bytes come from the canonical publication input before corruption, while recovery computes its own checksum over the preimage.

A disposable production mutation deletes the stage on the planning-error path; the preservation assertion fails at the first corrupted prefix.

The short-head filesystem laws and the existing canonical-prefix integration laws pass in copied Docker debug and release; the retention process-death matrix also passes.

Both workspace Clippy configurations with `-D warnings`, formatting, and source-structure checks pass.

Replay uses `cargo test --lib short_head --all-features` and `cargo test --test retention_stage_prefix --all-features`, adding `--release` for optimized execution; calibration uses a fresh build target.

This closes available head-checksum bytes only; other partially available semantic fields, root and manifest validation, and incomplete-stage pinning remain under audit.
