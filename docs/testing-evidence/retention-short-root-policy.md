# Interrupted root policy admission

Change kind: bug fix. Owner: `@flyingrobots`. Subject: Keep runtime preservation of invalid complete policy groups in interrupted retention roots (#99).

Regression commit `452e229` was observed RED on unfixed production `79e0530`: recovery discarded roots with an unsupported profile identity or a zero node limit and returned `Clean`.

An initial test attempt shared a sandbox name across concurrent laws and produced an `AlreadyExists` setup failure; that failure is excluded from the RED evidence. The committed regression isolates each field and prefix case, and both laws then fail at the intended runtime refusal check.

The medium filesystem laws invoke real authority recovery, require the exact corrupt-root stage and profile or closure-limit cause, and compare retained paths and bytes before and after refusal. The specified oracle is the version-2 registered profile and positive bounded closure-resource contract; profile digest expectations use the registered public profile constant, so an independently corrupted registry definition is outside this oracle.

The cases cover unsupported identity and version, a wrong profile digest, and zero and above-maximum values for nodes, depth, encoded bytes and physical bytes. They use deterministic literal byte mutations of the checked-in root fixture, with no random seed or reducer needed; the permanent reproducer is `filesystem_retention_short_policy_tests`.

Production now reuses `RegisteredRetentionProfile::admit` when the complete group is available through byte 88 and `RetentionClosureLimits::new` through byte 116. It does not allocate unavailable body entries or change the integrity-before-semantics order of complete record decoding.

Replay uses `cargo test --lib short_policy --all-features` and `cargo test --test retention_stage_prefix --all-features`, with `--release` for optimized execution, in copied Docker source on Linux aarch64 with Rust 1.96.0 and the owned ext4 sandbox. The retention process-death replay is `cargo xtask durability-crash-matrix --sequence retention`.

These laws establish refusal and byte preservation for complete policy groups, not partial fields, body ordering, root/manifest integrity prefixes, stage pinning against concurrent replacement, or physical power loss. Per-test resource ceilings and suite latency admission remain enforcement-profile gaps, not approved waivers. No performance claim or format/API change is made.

Delete these tests only when stronger public recovery coverage subsumes their malformed-policy cases and exact preservation promises; no existing test expectations were relaxed.

Calibration used disposable copied product trees and fresh build targets: wrong profile/limit causes fail `report exact root policy violation`, naming `Head` for root corruption fails `identify the corrupt policy stage`, and deleting `root.next` while returning the correct refusal fails `invalid policy must preserve retained evidence`. Both filesystem laws fail each injected defect at the corresponding assertion.

Debug and release regressions and canonical strict-prefix controls pass, as does the retention process-death matrix. Both workspace Clippy configurations pass with `-D warnings`, alongside formatting and source-structure checks. An initial Clippy failure for a needless by-value test parameter was corrected to a borrow and the focused debug/release tests and checks rerun; the original failure remains recorded.
