# Filesystem platform adapter evidence

This is an implementation record, not platform certification. The design is
in [filesystem platforms](../architecture/filesystem-platforms.md) and the
decision in [filesystem platform adapters](../adr/filesystem-platform-adapters.md).

## Claims and status

- Native production storage on default and case-sensitive APFS: **implemented**.
  Process-death evidence below. No power-loss experiment has been run on APFS.
- Linux ext4 admission, authority, and durability laws survive extraction of the
  native boundary: established by the Linux CI jobs on the merge candidate.
- No format change: no golden fixture changed, and Linux coordinates are untouched.
- No unsafe code and no new dependency: `Cargo.toml` and `Cargo.lock` are unchanged.

## Native execution

Darwin 27.0.0, arm64, pinned `rustc 1.96.0`, under `scripts/native/check.sh`.

| Check | Default APFS | Case-sensitive APFS |
| --- | --- | --- |
| `cargo test --all-features` (keep package, debug) | 959 passed, 0 failed | 959 passed, 0 failed |
| `cargo xtask durability-crash-matrix` | exit 0 | exit 0 |

The volumes are sparse images: default APFS `data` and case-sensitive APFS
`data-cs`, 256 MiB each, selected with `KEEP_NATIVE_DATA_IMAGE`. Release-profile
tests, `xtask` tests, and lint run in the `Native macOS storage` CI job.

An earlier set of native runs used Homebrew's Rust 1.98 ahead of rustup on `PATH`
and is not counted. The worker now selects the pinned toolchain and asserts its
version before running.

## Spelling oracle

`F_GETPATH` was rejected. With a descriptor retained on `a/HEAD`, a probe of its
hard link reported `b/head` for it, so the name is vnode state, not bound to the
descriptor, and publication uses hard links. Directory listings return the stored spelling, so the check is
byte equality against a listing of the parent.

Calibration with the check disabled, on default APFS:

- `a_foreign_spelling_never_answers_for_a_protocol_name`,
  `a_non_ascii_fold_never_answers_for_a_protocol_name`, and
  `mutation_through_a_foreign_spelling_leaves_the_entry_unchanged` **fail**.
- `darwin_store_spelling` laws **still pass**. Namespace admission independently
  refuses the foreign entry, so those laws do not witness the per-lookup check.

## Laws that remain Linux-only, and why

- tmpfs refusal (`reader_platform_law_tests`, `a_writer_lock_cannot_mint_...`):
  no tmpfs. Darwin refusal of non-APFS uses devfs and of read-only APFS uses the
  sealed system volume, both at the adapter rather than the public boundary.
- Raw non-UTF-8 names: APFS refuses to create them.
- `filesystem_retention_fifo_tests`: Rustix omits `mknodat` on Apple targets and
  `std`'s `mkfifo` is unstable. `tests/darwin_fifo_refusal.rs` covers a FIFO at
  `HEAD` only.
- O_PATH directory synchronization and ext4 profile laws: Linux mechanisms.

## Weaker on macOS

- The reader-fence law cannot observe a queued `flock`, because Darwin has no
  `/proc/locks`. It requires that no snapshot escapes for two seconds, so a late
  child can weaken it but cannot falsely fail it.
- Isolated laws use `ulimit -t` where Linux uses wall-clock `timeout`.

## Not measured

- The cost of the parent scan per existing-entry lookup. It is linear in the
  directory size and matters most for a large segment pool.
- `F_FULLFSYNC` durability. A successful call is not proof that hardware honored
  it.
- A real non-APFS or network volume at the public boundary.

## Runner bounds

Compilation and test output stays on the reusable 1.5 GiB `build` image, 349 MiB
used after a clean run. Each data image stayed under 4 MiB used. Logs total
1.1 MiB. The sandbox denies network except local Unix sockets, which the process
laws need, and denies writes outside the build and selected data images.

The runner watches only its own process group, so a law that starts its own group
is bounded by that law's watchdog, not by the runner.
