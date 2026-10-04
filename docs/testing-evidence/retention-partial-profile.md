# Interrupted registered-profile prefixes

Change kind: bug fix. Owner: `@flyingrobots`. Subject: Keep runtime refusal and evidence preservation when an interrupted root already contradicts its registered profile (#99).

Regression `f4875fe` was observed RED on unfixed production `fc29870`: the first available identity byte contradicted the registered encoding, but real filesystem recovery discarded the root stage and returned `Clean`.

The medium filesystem law enumerates every interrupted profile end position from 49 through 87, flips the last available profile byte, and requires `StageCorrupt(Root)` with exact `PrefixByteMismatch` offset, expected byte and observed byte. It independently compares retained paths and bytes before and after refusal. The oracle is the checked-in canonical root fixture and the format's closed single-profile registry; no unavailable byte is asserted as observed.

The fixed-field comparison implementation is shared with existing stage-prefix admission. For complete profile groups the existing typed domain coordinate/digest admission remains in force. The deterministic canonical-prefix integration sweep protects every valid truncated fixture prefix; complete-policy filesystem laws protect the prior diagnostic contract. Registration of another valid profile must update this single-profile prefix rule and its conformance evidence together.

Replay uses `cargo test --lib partial_profile --all-features`, `cargo test --lib short_policy --all-features` and `cargo test --test retention_stage_prefix --all-features`, adding `--release` for optimized execution. These run in copied Docker source with Rust 1.96.0 on Linux aarch64 and an owned ext4 sandbox. There is no random seed: the ordered end position and fixture identify each minimized one-byte counterexample. The permanent corpus is the checked-in law and fixture.

Fault scope is deterministic interrupted profile bytes and real filesystem recovery, not physical power loss or concurrent out-of-band replacement. Closure fields, record bodies, root/manifest integrity prefixes, incomplete-stage pinning and post-removal failure semantics remain open. Per-test resource ceilings and suite latency admission remain unapproved enforcement gaps; container execution alone does not establish those controls.

No public API or wire encoding changes. No performance claim is made. Delete the law only when stronger public recovery evidence subsumes these exact contradiction and preservation promises; no test expectation was weakened.

Disposable copied product mutations with fresh build targets prove the load-bearing checks: a wrong byte offset fails `name the exact available profile contradiction`, a wrong stage fails `identify the corrupt profile stage`, and deletion on refusal fails `must preserve retained evidence`. The first coordinate mutation did not compile because of an unused variable and is excluded from calibration; its corrected build reaches and fails the intended assertion.

The focused filesystem laws, complete-policy controls and canonical strict-prefix integration sweep pass in debug and release. The retention process-death matrix, both workspace Clippy configurations with `-D warnings`, formatting and source-structure checks pass.
