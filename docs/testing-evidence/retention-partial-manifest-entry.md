# Partial manifest entry admission

Change kind: bug fix. Owner: `@flyingrobots`. Subject: Keep runtime preservation of partial manifest entries whose available bytes already violate generation or namespace-order contracts (#99).

Regression `574619d` was observed RED on unfixed production `e52b4fd`: both an impossible namespace prefix and a zero root-generation field caused `DiscardManifestStage`, leaving only the root protected. The positive ordered-prefix control already passed on that revision.

The medium filesystem laws cover descending namespace prefixes, fully available duplicate namespaces, and equal prefixes whose unknown suffix cannot exceed an all-maximum predecessor. A separate sweep places generation zero in every partial-entry length after that field is complete. Each invokes real recovery, requires the exact manifest-stage refusal, entry index and cause, and independently compares retained paths and bytes before and after refusal.

The small public-assessment control generates every nonempty partial-entry length from two known ordered namespace pairs: one differs in its first byte, the other only in its last. Each full ordered value witnesses a possible completion of its prefix; unknown zero-leading generation bytes remain permitted. These are deterministic prefix sweeps, with field values and lengths as replay coordinates; the checked-in test is the permanent reproducer and no random seed is needed.

The specified oracle is strict unsigned lexicographic namespace order and positive root generation. Production shares generation and ordering admission with complete entries. For an incomplete namespace, it computes the greatest possible completion by filling only unknown suffix bytes with `0xff`; refusal follows only if even that bound cannot be greater than the predecessor. The bound is never returned as an observed identity.

Replay uses `cargo test --lib partial_entry --all-features`, `cargo test --lib body_prefix --all-features` and `cargo test --test retention_stage_prefix --test retention_manifest_codec --all-features`, adding `--release` for optimized execution. Runs use copied Docker source, Rust 1.96.0, Linux aarch64 and the owned ext4 sandbox. The retention process-death replay is `cargo xtask durability-crash-matrix --sequence retention`.

This establishes constraints decidable from the current partial manifest entry, not feasibility of all declared future entries, partial anchor admission or every partial header constraint. Incomplete-stage pinning, post-removal failures and physical power-loss guarantees remain outside this evidence. Per-test resource ceilings and suite latency admission remain unapproved enforcement gaps. There is no wire/API change or performance claim.

Delete the laws only when stronger public runtime evidence subsumes these refusal, exact diagnostic, preservation and valid-completion promises; no existing expectation was weakened.

Disposable copied product mutations with fresh targets change the reported entry index, name the wrong stage, and delete the manifest on refusal; both refusal laws fail the corresponding diagnostic, stage and retained-evidence assertions. Filling unknown namespace bytes with zero instead of computing the greatest completion makes the positive law fail at a valid prefix, calibrating the absence of false refusals.

Focused laws and complete-entry controls pass in debug and release, as do the canonical-prefix and manifest-codec suites. The retention process-death matrix passes. An initial Clippy request to express a boolean pattern match with `matches!` was corrected, followed by fresh focused debug/release runs and passing all-feature/no-default-feature workspace Clippy with `-D warnings`, formatting and source-structure checks; earlier failed attempts remain recorded.
