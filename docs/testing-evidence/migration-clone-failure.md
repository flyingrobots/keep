# Migration capability-clone failure

Change kind: bug fix. Owner: `@flyingrobots`. Oracle: a failed root capability duplication reports `FilesystemMigrationAuthorityError::Namespace` with the original operating-system cause; root identity observation has its separate `RootIdentity` boundary.

At base `84f5861`, the repository-task migration constructor used the unchecked-admission helper and labeled either failure `Platform`, despite performing no platform admission. The original review referred to an older location and a `RootIdentity` wrapper; the current defect was verified at the moved repository-task constructor.

RED commit `50d0d0b` adds a Linux child-process regression that creates a valid version-one store, reacquires its real writer lock, fills a child-owned descriptor table, then invokes the public repository-task migration constructor. The kernel returns `EMFILE` during root capability cloning; the unfixed code reports `Platform` and fails the named namespace assertion.

The child runs with a 64-descriptor limit and a 20-second deadline through the Docker image's shell and timeout utility. The parent retains its descriptor limits. Descriptors are released before inspecting the returned error, and no test mutates global process limits in the shared test runner.

The fix performs capability cloning and lenient identity observation separately, preserving each source at its proper boundary, and releases the temporary cloned root before opening inventory capabilities as the previous helper did.

The physical clone failure is exercised directly; identity-probe failure mapping is verified by code inspection, not claimed as a separately injected kernel fault. This evidence does not broaden the repository-task API into production platform admission.

A copied-source mutation retained the `Namespace` variant but reconstructed the I/O error from its kind, losing `EMFILE`; the same assertion failed, independently calibrating source preservation in addition to the original wrong-boundary RED.

The migration suite passed in debug and release. After explicitly restoring the temporary root handle's original lifetime, the focused regression passed again in both modes, with both workspace Clippy feature configurations, formatting and source-structure checks. Markdown validation passed separately.

Replay uses `cargo test --lib a_root_clone_failure --all-features`, adding `--release` for optimized execution, inside copied Linux Docker. The fault schedule is deterministic descriptor exhaustion after writer-lock acquisition. Other targets do not run this Linux-specific regression.
