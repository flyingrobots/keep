# Interrupted closure-limit admission

Change kind: bug fix. Owner: `@flyingrobots`. Subject: Keep runtime refusal and preservation of interrupted roots whose available closure limits cannot admit a legal completion (#99).

Regression `25f4be2` was observed RED with unchanged admission behavior from `3f3b7ea`: a node-limit prefix containing one `0xff` byte was discarded with `Clean`. The RED commit adds the diagnostic variant and formatting/source support so the regression compiles, but does not add prefix admission.

The medium filesystem law exercises every nonempty strict prefix of each limit with a leading `0xff`, plus each complete field containing zero or the specified ceiling plus one. It requires the exact corrupt stage, typed domain error or impossible-prefix coordinates, and unchanged retained paths and bytes. Its specified oracle is the format's positive bounded big-endian integer contract; test ceilings are independent protocol literals.

The small public-assessment law deterministically generates admitted powers of two through each ceiling and the ceiling itself, then checks every nonempty prefix. This protects the existence of a legal completion, including all-zero partial prefixes. Canonical strict-prefix controls and complete-policy filesystem laws retain earlier promises. There is no random seed: replay inputs are field offset, generated value and prefix length; the checked-in tests and fixture are the permanent corpus, and ascending enumeration provides the first failing boundary.

The domain now owns independent resource admission shared with complete `RetentionClosureLimits` construction. The boundary computes a partial big-endian field's smallest completion by zero-filling only for that calculation. It reports a new `ClosureLimitPrefixAboveMaximum` diagnostic when this minimum exceeds the domain ceiling; complete fields retain the existing `ClosureLimit` cause. No missing byte is presented as an observation, no candidate policy is synthesized, and no wire-format or public constructor behavior changes. The public decode-error enum gains the named variant, which exhaustive consumers must account for.

Replay uses `cargo test --lib closure_prefix --all-features`, `cargo test --lib short_policy --all-features`, `cargo test --test retention_stage_prefix --all-features` and `cargo test --test retention_root_encoding --all-features`, with `--release` for optimized execution, in copied Docker source. The environment is Linux aarch64, Rust 1.96.0 and an owned ext4 sandbox; the process-death replay is `cargo xtask durability-crash-matrix --sequence retention`.

The fault model is interrupted limit bytes and real filesystem recovery, not physical power loss or concurrent namespace replacement. Other partial header fields, body admission, integrity prefixes, incomplete-stage pinning and post-removal failure semantics remain open. Per-test resource ceilings and suite latency admission remain unapproved enforcement gaps. No performance claim is made.

Deletion requires stronger public recovery evidence that subsumes both impossible-prefix refusal/preservation and valid-completion controls; no existing behavioral expectation was weakened.

Four disposable product mutations use copied trees and fresh targets: subtracting one from the reported minimum fails the exact-cause assertion; reporting `Head` fails the stage assertion; deleting `root.next` on refusal fails retained-evidence equality; and rejecting zero-valued partial prefixes fails the generated valid-completion law. All reach the intended assertions rather than failing setup or compilation.

The focused runtime laws, canonical-prefix controls and root-encoding contract suite pass in debug and release. The retention process-death matrix, both workspace Clippy configurations with `-D warnings`, formatting and source-structure checks pass.
