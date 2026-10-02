# Forbidden source basenames

The source checker owns literal filename policy for inventoried repository
sources. AGENTS.md forbids nine Rust filenames: `utils.rs`, `helpers.rs`,
`common.rs`, `misc.rs`, `shared.rs`, `manager.rs`, `service.rs`, `types.rs`
and `models.rs`. Completed roadmap task T-10.2 requires this enforcement.

Admission compares the final path component with those exact names after
regular-file and tracked-mode admission. It does not interpret prefixes,
parent directory names or case variants as new prohibitions. This preserves
semantic filenames such as `segment_header.rs` and `storage_models.rs`.
Existing inventory scope, root identity, no-follow opens, Python refusals and
line-size checks retain their behavior.

A forbidden name returns `ForbiddenFilename` with its repository-relative
path. Diagnostics escape control characters. Selection is deterministic;
this check performs no write, changes no durable format or API, and grants no
storage authority. Its additional work is a fixed nine-name comparison per
admitted source path, with allocation only when building a refusal.

The full-checker regression failed before the gate and passes afterward.
Independent policy cases cover every prohibited name at three parent depths;
negative cases protect prefixed names and directory names. A real CLI probe
in copied Docker source refuses `src/utils.rs`, and the clean source passes
again after removing that fixture. All 41 existing/new source laws pass in
Docker debug and release. No native tests or writable host Git mount were
used. No performance or full-workspace test claim is made.
