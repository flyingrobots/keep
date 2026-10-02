# Partial initial predecessor admission

Change kind: bug fix. Owner: `@flyingrobots`. Subject: Keep runtime refusal and evidence preservation for impossible initial-record predecessor prefixes (#99).

Regression `a0a8133` was observed RED against unfixed production `b941d58`: an initial root with a nonzero byte in its incomplete predecessor field returned clean recovery after `DiscardRootStage`. Successor-prefix controls passed on that same unfixed code.

The medium filesystem law sweeps every nonempty strict predecessor prefix for roots, manifests and heads. A nonzero final available byte must produce the exact stage-specific corruption refusal, byte offset, expected zero and observed value, while preserving every retained path and byte. It invokes the real filesystem recovery authority after driving the required earlier publication phases.

The small public-assessment law sweeps the same prefix endpoints for successor generations two and the maximum generation, with both zero and nonzero available predecessor bytes. Each remains an interrupted record with a possible nonzero completion. The specified oracle is the protocol's initial-versus-successor history rule, independently of the admission helper. Inputs and endpoints are deterministic and permanently represented in the test; no probabilistic seed or shrinking step is involved.

Admission compares only available predecessor bytes when the complete generation is initial. It neither pads missing bytes into observed state nor changes complete-field semantic errors. Root and manifest history admission reuse the existing boundary module; head admission calls it only while the predecessor is incomplete. No wire-format or public API change is introduced.

Replay uses `cargo test --lib history --all-features` and `cargo test --test retention_stage_prefix --all-features`, adding `--release` for optimized execution. Runs use copied Docker source, Rust 1.96.0, Linux aarch64 and the owned ext4 sandbox. The retention process-death replay is `cargo xtask durability-crash-matrix --sequence retention`.

This evidence does not establish partial numeric-header admission, feasibility of every future declared entry, incomplete-stage pinning, post-removal failure handling, physical power-loss survival or out-of-band replacement safety. Per-test resource ceilings and suite latency admission remain unapproved enforcement gaps. No performance claim is made.

Delete these laws only when stronger public runtime evidence subsumes the same initial refusal, exact diagnostics, evidence preservation and valid-successor completion promises. No existing expectation is weakened.

Disposable copied product mutations with fresh build targets change the reported expected byte, name the wrong stage, delete the root on refusal and apply the initial-record zero rule to successors. Each fails its corresponding exact-cause, stage, retained-evidence or possible-successor assertion. The first diagnostic mutation command matched no formatted source and changed nothing; that run is excluded. The corrected mutation was inspected and failed the named assertion in a fresh target.

The focused history laws and canonical-prefix controls pass in debug and release. The retention process-death matrix, both workspace Clippy configurations with `-D warnings`, formatting and source-structure checks pass.
