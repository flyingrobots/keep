# Partial anchor coordinate admission

Change kind: bug fix. Owner: `@flyingrobots`. Subject: Keep runtime preservation of short root anchors whose available identity framing or complete layout length already contradicts its embedded codec (#99).

Regression `8b299da` was observed RED on unfixed production `067966c`: a bad first blob-identity byte and a complete zero layout length inside short anchors were discarded with `Clean`.

The medium filesystem laws flip each available fixed byte of blob magic, identity version and hash algorithm, and layout magic, identity version and codec, stopping at that byte. They also supply zero, noncongruent and above-maximum complete layout lengths at the earliest available field boundary and near anchor completion. Both first and second anchors are exercised, requiring absolute offending-byte coordinates or exact anchor index and layout-length cause. Every refusal independently checks that all retained paths and bytes remain unchanged.

The specified/derived oracle is the frozen canonical root's embedded identity bytes and the layout protocol's 176-byte minimum, 46,137,520-byte maximum and 44-byte entry stride. Test expectations use protocol literals rather than invoking production admission. Inputs are deterministic field/anchor positions and malformed lengths; the checked-in laws and fixture preserve the reproducer without a random seed.

Production checks the final incomplete anchor without allocating an unavailable suffix. Fixed fields refer to the existing boundary codecs' constants, and complete layout lengths reuse their existing validator; only internal visibility expands. Full anchor admission retains its original typed parsing and ordering path. No wire format or public API changes.

Replay uses `cargo test --lib partial_anchor --all-features`, `cargo test --lib body_prefix --all-features` and `cargo test --test retention_stage_prefix --test retention_root_decoding --all-features`, with `--release` for optimized execution. Runs use copied Docker source, Rust 1.96.0, Linux aarch64 and the owned ext4 sandbox. The retention process-death replay is `cargo xtask durability-crash-matrix --sequence retention`.

This evidence covers fixed coordinate fields and complete layout lengths, not every partial numeric length, partial-anchor ordering, future-entry feasibility, partial header rule, concurrent replacement or power loss. Incomplete-stage pinning and post-removal failures remain open. Per-test resource ceilings and suite latency admission remain unapproved enforcement gaps. No performance claim is made.

Delete these laws only when stronger public runtime evidence subsumes their exact coordinate/length refusals and retained-evidence guarantees; no existing expectation was weakened.

Disposable copied product mutations with fresh targets alter reported byte offsets and anchor indices, name the wrong stage, and remove the root stage on refusal. Both filesystem laws fail at the corresponding exact-cause, stage and retained-evidence assertions.

Focused and complete-anchor filesystem laws, canonical-prefix controls and complete root-decoding controls pass in debug and release; the retention process-death matrix passes. Initial Clippy findings for an option match and a tuple-shaped fixture return were corrected, followed by passing focused debug/release tests, both workspace Clippy configurations with `-D warnings`, formatting and source-structure checks. Original failed attempts remain recorded.
