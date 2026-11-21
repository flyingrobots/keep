# Bounded local Docker validation

Change kind: operational behavior change for #107 validation; Keep's runtime and formats are unchanged.

This runner reuses the existing `keep-validation` container and its installed pinned Rust toolchain. It never builds an image, creates a container, or downloads dependencies. `KEEP_DOCKER_CONTAINER` can select an existing equivalent worker. The worker must have Bash, util-linux, e2fsprogs, Cargo and permission to mount loop devices; the current privileged review worker already supplies these. Do not use this privileged worker for untrusted code.

```sh
scripts/docker/cargo.sh /path/to/checkout test --all-features
scripts/docker/cargo.sh /path/to/checkout test --all-features --release
scripts/docker/cargo.sh /path/to/checkout clippy --all-targets --all-features -- -D warnings
```

The runner copies the checkout without `.git`, `node_modules`, or the Cargo output paths `target`, `fuzz/target`, and `xtask/target` into one disposable source slot. Source directories such as `xtask/src/fuzz_campaign/target` remain included. Its source slot and shared Cargo target live on a sparse ext4 filesystem capped at 20 GiB. Incremental compilation is disabled. Dependencies already installed in the worker are reused offline. An unchanged source archive reuses the existing copy; a changed archive is extracted with fresh modification times so older parent sources do not reuse newer candidate binaries. A changed checkout does not require a new target directory. Red-on-parent validation must still record the tested source revision and observe the intended assertion fail.

An exclusive lock serializes copying and execution; a second invocation refuses immediately. The host runner also refuses to start below 50 GiB free on the checkout filesystem. The filesystem limit bounds this runner's source, target and test files written there even during a command; when full, the command fails with insufficient space. The free-space check is a start-time guard, not a reservation against other projects.

Cargo compilation uses two jobs. The existing `keep-validation` worker is configured with four CPUs and 8 GiB of memory, with no additional swap allowance. These container limits must also be set on any replacement worker; selecting another existing container does not alter its Docker resource limits.

Use this entry point for subsequent Keep Cargo validation. Do not create per-PR images, containers or target directories. Cargo configuration and target-directory command-line overrides are refused. A direct `docker exec`, an absolute-path test output, the shared Cargo registry, `/tmp`, or another project can still write outside this filesystem; this is a build-cache limit, not a quota on the entire Docker VM. Keep existing controlled recovery-test mounts when a test specifically needs them, and account for their storage separately. Do not bypass a full cache by inventing another output path: preserve receipts, then run `scripts/docker/cargo.sh /path/to/checkout clean` to clear this shared cache.

Export required logs, counterexamples and receipts before the next changed snapshot, which replaces the disposable source slot. Authoritative source stays in the host checkout. This worker is for serial validation; source under active editing should be frozen before copying. Removal of the worker also removes its cache; stop it with `docker stop keep-validation` when idle and reuse it with `docker start keep-validation`.

On 2026-10-03 the previous worker, `keep-lawyer-138`, was renamed to `keep-validation` without recreating it. Its writable layer had reached 428 GB. `/build` occupied 394 GiB and contained 420 Cargo profile directories. A file-size scan attributed 98.3% of apparent `/build` bytes to `deps`, `incremental`, and `build` subdirectories. Removing those generated directories and their `.fingerprint` metadata reduced `/build` to about 8 GiB while preserving source copies, receipts, profile-level executables, and mounted recovery-test data. These are different measurements: apparent file bytes, allocated directory blocks, and Docker's writable-layer size are not interchangeable.

Validation of this operational change used Bash syntax checks, Markdown lint, and a dependency-free Rust probe in the actual worker. An unchanged snapshot reported `Fresh`; changed source exited with its new status; restoring source with an older modification time rebuilt and restored the original status. A target-directory override was refused before creating that directory. The mounted filesystem reported a 20 GiB capacity. No full Keep validation campaign was repeated for this runner change.
