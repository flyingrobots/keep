# Partial layout-length admission

Change kind: bug fix. Owner: `@flyingrobots`. Subject: Keep runtime preservation of partial anchor layout lengths whose available bytes force every completion above the protocol maximum (#99).

Regression `18a44e3` was observed RED with admission behavior unchanged from `6c70a28`: a one-byte layout-length prefix beginning `0xff` was discarded with `Clean`. The RED commit adds only the new diagnostic variant and its display/source handling alongside the tests, so the regression compiles without fixing admission. Its valid-prefix control already passed.

The medium filesystem law covers every nonempty strict length-field prefix with an excessive leading byte, plus the nearest impossible seven-byte prefix above the ceiling, in both the first and second anchor. It requires the exact corrupt-root stage, anchor index, minimum possible completion and format maximum, followed by unchanged retained paths and bytes across real recovery.

The small public-assessment control generates canonical layout lengths from zero entries and power-of-two entry counts through the format ceiling, then checks every strict length prefix. Each full canonical value witnesses a legal completion; the zero-leading prefixes must remain admissible. The independent specified oracle is the 176-byte fixed overhead, 44-byte entry stride and 46,137,520-byte maximum. Deterministic values, anchor indices and prefix lengths are replay coordinates; the checked-in laws preserve the corpus.

Production zero-fills unavailable bytes only to compute a lower bound, compares it with the domain maximum and never admits that bound as an observed layout identity. Complete fields retain the existing codec's bounds and congruence checks. The new public `LayoutLengthPrefixAboveMaximum` decode-error variant distinguishes a minimum possible completion from an observed complete value; exhaustive consumers must handle it. No wire-format change is made.

Replay uses `cargo test --lib partial_anchor --all-features` and `cargo test --test retention_stage_prefix --test retention_root_decoding --all-features`, with `--release` for optimized execution. Runs use copied Docker source, Rust 1.96.0, Linux aarch64 and the owned ext4 sandbox. The retention process-death replay is `cargo xtask durability-crash-matrix --sequence retention`.

The failure model is interrupted length fields and real filesystem recovery, not partial-anchor ordering, feasibility of all future entries, other partial header constraints, concurrent replacement or physical power loss. Incomplete-stage pinning and post-removal failures remain open. Per-test resource ceilings and suite latency admission remain unapproved enforcement gaps. No performance claim is made.

Delete the new laws only when stronger public runtime evidence subsumes their exact refusal, preservation and valid-completion promises; no existing expectation was weakened.

Disposable copied product mutations with fresh targets change the reported minimum, name the wrong stage and delete the root on refusal; the new refusal law fails its exact-cause, stage and retained-evidence checks respectively. A mutation rejecting zero lower bounds fails the generated valid-prefix control, calibrating the absence of false refusals.

Partial-anchor laws and canonical-prefix/root-decoding controls pass in debug and release, and the retention process-death matrix passes. Initial Clippy findings for the expanded diagnostic formatter and singleton iterator were corrected by sharing identity-message formatting and using `iter::once`; focused debug/release tests, both workspace Clippy configurations with `-D warnings`, formatting and source-structure checks then pass. Original failed attempts remain recorded.
