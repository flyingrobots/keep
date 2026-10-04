# Partial anchor ordering

Change kind: bug fix. Owner: `@flyingrobots`. Subject: Keep runtime refusal and preservation when no canonical completion of a partial anchor can follow its predecessor (#99).

Regression `575e742` was observed RED on unfixed production `aec721f`: a partial blob length already below its predecessor was assessed as `Truncated` and discarded by filesystem recovery. Ordered-prefix controls passed on the unfixed revision.

Medium filesystem laws cover decisive and near-complete prefixes at each ordering level: blob length, blob digest, layout length and layout digest. They also cover a predecessor at the greatest canonical anchor, where no successor exists. Refusal must identify the root stage and second anchor exactly and preserve every retained path and byte. Small public-assessment sweeps check every decisive malformed prefix and every partial prefix of the reversed, strictly ordered pairs.

The specified oracle is lexicographic ordering of typed blob length/digest followed by typed layout length/digest. Test pairs differ at one chosen ordering field and use independent protocol limits; reversing each strict descending pair supplies a known valid completion. Inputs are deterministic field choices and prefix lengths, permanently retained in the test; there is no random seed or probabilistic reduction.

The boundary builds a greatest candidate consistent with observed bytes, using the domain's greatest canonical layout length under a bound to respect both the ceiling and entry alignment. It confirms that canonicalization preserved every available byte, then invokes the existing typed anchor parser and ordering check. The candidate is only a feasibility witness and is never returned as observed state. The process uses a fixed-size buffer and does not allocate the unavailable suffix. No wire or public API change is introduced.

Replay uses `cargo test --lib anchor_order_prefix --all-features`, `cargo test --lib partial_anchor --all-features` and `cargo test --test retention_stage_prefix --test retention_root_decoding --all-features`, adding `--release` for optimized execution. Runs use copied Docker source, Rust 1.96.0, Linux aarch64 and the owned ext4 sandbox. The retention process-death replay is `cargo xtask durability-crash-matrix --sequence retention`.

This covers the current partial anchor, not whether all future entries declared by a header remain possible. Partial header constraints, remaining-entry feasibility, incomplete-stage pinning and post-removal failures remain open; physical power loss and out-of-band replacement are outside this evidence. Per-test resource ceilings and suite latency admission remain unapproved enforcement gaps. No performance claim is made.

Delete these laws only when stronger public runtime evidence subsumes the exact refusal, preservation and valid-completion promises; no existing expectation was weakened.

Disposable copied product mutations with fresh targets alter the offending anchor index, name the wrong stage, and delete the root on refusal. The runtime laws fail the corresponding assessment/diagnostic, stage and retained-evidence checks. Replacing unknown maxima with zero makes the positive ordered-prefix law fail, calibrating the absence of false refusals.

The focused ordering and partial-anchor laws, canonical-prefix controls and complete root-decoding controls pass in debug and release. The retention process-death matrix, both workspace Clippy configurations with `-D warnings`, formatting and source-structure checks pass.
