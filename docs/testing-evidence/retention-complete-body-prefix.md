# Complete entries in interrupted retention bodies

Change kind: bug fix. Owner: `@flyingrobots`. Subject: Keep runtime refusal and preservation when an interrupted root or manifest already contains invalid complete body entries (#99).

Regression `10ebd6f` was observed RED on unfixed production `1c46c99`: a malformed blob identity and duplicate anchors in an interrupted root body were discarded with `Clean`.

The medium filesystem laws also exercise malformed layout identity, manifest generation zero and duplicate manifest namespaces. Fixtures declare one further unavailable entry with self-consistent framing so body completion and its set-digest check cannot mask the semantic finding. Each law requires the precise corrupt stage, entry index and typed admission failure, then compares retained paths and bytes across real recovery.

The specified oracle is canonical identity framing, positive root generations and strictly ordered unique anchors/namespaces. Expected magic bytes, indices and zero-generation errors are stated independently of production parsing; the checked-in conformance records supply otherwise valid context. Inputs are deterministic minimal malformed/duplicate entries, identified by stage and fault; no random seed or reducer is needed.

Full decoding and interrupted-body admission share entry parsing and ordering logic. The prefix path walks only available complete chunks and retains only the preceding semantic coordinate; it neither allocates the declared unavailable suffix nor treats an incomplete entry as admitted. Existing integrity-before-semantics ordering at complete-record decoding is retained.

Replay uses `cargo test --lib body_prefix --all-features` and `cargo test --test retention_stage_prefix --test retention_root_decoding --test retention_manifest_codec --all-features`, adding `--release` for optimized execution. Runs use copied Docker source, Rust 1.96.0, Linux aarch64 and an owned ext4 sandbox. The retention process-death replay is `cargo xtask durability-crash-matrix --sequence retention`.

The failure model covers complete bad entries inside incomplete bodies, not every incomplete entry field, physical power loss or out-of-band replacement. Partial header and partial-entry constraints, incomplete-stage pinning and post-removal failures remain open. Per-test resource ceilings and suite latency admission remain unapproved enforcement gaps. There is no wire/API change or performance claim.

Delete these laws only when stronger public recovery evidence subsumes malformed identity, generation and ordering refusals with the same evidence-preservation guarantee; no existing expectation was weakened.

Disposable copied product mutations with fresh targets alter the reported entry index, name the wrong corrupt stage, and delete the root stage while returning the correct refusal. Both filesystem laws fail at the corresponding exact-cause, stage and retained-evidence assertions.

Debug/release filesystem laws, canonical-prefix controls and complete root/manifest codec suites pass, as does the retention process-death matrix. An initial Clippy semicolon finding in the fixture was corrected; the focused runtime laws and both workspace Clippy configurations with `-D warnings`, formatting and source-structure checks then pass. The original static failure remains recorded.
