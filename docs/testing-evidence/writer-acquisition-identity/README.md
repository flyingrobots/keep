# Acquisition calibration receipts

These captured outputs and production mutation patches support [#169's evidence](../writer-acquisition-identity.md). Absolute container source/target prefixes in output are normalized to `<isolated-build>`, and redundant trailing blank lines are removed; assertion messages, outcomes and diagnostics are otherwise retained. They are execution receipts, not golden expectations or tests of test-count totals.

## Coordinates and environment

The old-oracle survivor uses main `6051abb25a9fd33ae7ee0de5614514b709a4d82a` with the identity-refusal result ignored. The early-order survivor uses the initial strengthened test at `6a5958b9f27b81be70b25e1c680398258818e2f8` with verification moved before locking. The final RED receipts use the after-lock checkpoint and the current strengthened law. The resulting committed head and final hosted check results are recorded in [PR #170](https://github.com/flyingrobots/keep/pull/170), separately from these focused receipts.

Execution used copied Linux Docker source/build trees, Rust `1.96.0 (ac68faa20 2026-05-25)`, Cargo `1.96.0 (30a34c682 2026-05-25)`, host target `aarch64-unknown-linux-gnu`, the committed lockfile and default Cargo features for focused tests. Mutant RED runs use the debug test profile. Restored tests use debug and release; Clippy explicitly enables every feature and target. The final calibration uses ext4 scratch for both library and integration fixtures. Earlier old-oracle/early-order library runs used overlay; the complete profile correction and excluded setup failures are recorded in the parent evidence page.

Per-test resource ceilings and network-denial enforcement were not supplied by this execution profile; the medium test owns its scratch state and does not call the network. The container itself is not claimed as complete hermeticity enforcement. Maintainer `@flyingrobots` owns the contract; the author supplies these receipts.

## Replay inside a copied Docker checkout

Run the following only inside the repository's Docker validation environment, with an isolated copy of the candidate source and a dedicated `CARGO_TARGET_DIR`. Library fixtures use source-local `target/tmp` when `CARGO_TARGET_TMPDIR` is absent; integration fixtures use the target scratch directory. Bind both to ext4 before running the broader production-platform suite. Do not mutate a running validation tree or mount the host repository writable.

For each patch below, start with the unmutated candidate, apply the patch, touch the production file to invalidate timestamp caches, execute the focused command, retain the failing output, reverse only that patch, and touch the restored source before GREEN. Successful compilation followed by the named runtime failure is required; an exit code alone is insufficient.

```sh
git apply docs/testing-evidence/writer-acquisition-identity/early-order.patch
touch src/adapters/filesystem_writer_lock.rs
cargo test --locked --lib filesystem_writer_lock
git apply -R docs/testing-evidence/writer-acquisition-identity/early-order.patch
touch src/adapters/filesystem_writer_lock.rs
```

Substitute each other listed patch for `early-order.patch`. The captured assertions use pre-formatting line coordinates; the named law, assertion expression and literal expected bytes identify the current check.

| Mutation | Patch | Actual RED output | Named failure |
| --- | --- | --- | --- |
| Verify before taking the file lock | [patch](early-order.patch) | [receipt](early-order-red.txt) | `replacement received writer authority` |
| Run checkpoint before taking the file lock | [patch](early-checkpoint.patch) | [receipt](early-checkpoint-red.txt) | `replacement checkpoint must observe the acquired kernel lock` |
| Ignore the identity refusal | [patch](ignored-refusal.patch) | [receipt](ignored-refusal-red.txt) | `replacement received writer authority` |
| Return the wrong phase | [patch](wrong-phase.patch) | [receipt](wrong-phase-red.txt) | `replacement must retain the identity-refusal boundary` |
| Truncate displaced evidence | [patch](lost-original.patch) | [receipt](lost-original-red.txt) | Original bytes differ from empty observed bytes |
| Truncate replacement evidence | [patch](lost-replacement.patch) | [receipt](lost-replacement-red.txt) | Replacement bytes differ from empty observed bytes |

The [old-oracle survivor](old-oracle-survived.txt) executes `cargo test --locked --lib filesystem_writer_lock`, `cargo test --locked --test catalog_writer_lock`, `cargo test --locked --test store_initialization`, and `cargo test --locked --lib filesystem_store_initializer_tests` on the pinned main mutation. The [early-order survivor](early-order-survived.txt) executes only the first command on the initial strengthened test. Their survival establishes the precise gaps; neither is claimed as full-suite mutant survival.

The [restored GREEN receipt](restored-green.txt) runs the following after every production mutation is removed:

```sh
cargo test --locked --lib filesystem_writer_lock
cargo test --locked --release --lib filesystem_writer_lock
cargo test --locked --test catalog_writer_lock
cargo test --locked --release --test catalog_writer_lock
cargo test --locked --test store_initialization
cargo test --locked --release --test store_initialization
cargo test --locked --lib filesystem_store_initializer_tests
cargo test --locked --release --lib filesystem_store_initializer_tests
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
```

The helper-only test is retired because the stronger authority-boundary law subsumes its refusal claim. Complete generated schedule exploration, unrelated recovery paths and physical durability remain outside these receipts.

The [checkpoint-order survivor](early-checkpoint-survived.txt) runs the focused debug law at `1b27e4c6d2e791b8088123793befc219a97937c2` with the checkpoint moved before locking. The [checkpoint-order RED](early-checkpoint-red.txt) adds the independent contention witness to that mutation; the [restored checkpoint GREEN](checkpoint-green.txt) records formatting, the focused law in debug/release and all-feature workspace Clippy after restoring production order. These runs use the same copied Docker toolchain and ext4 scratch profile above. They supplement the earlier receipts rather than changing their historical coordinates.
