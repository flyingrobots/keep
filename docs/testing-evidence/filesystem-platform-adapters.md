# Filesystem platform adapter evidence

This is an implementation record, not platform certification. The design is
in [filesystem platforms](../architecture/filesystem-platforms.md) and the
decision in [filesystem platform adapters](../adr/filesystem-platform-adapters.md).

## Claims and status

- Native production storage on default and case-sensitive APFS: **implemented**.
  Process-death evidence below. No power-loss experiment has been run on APFS.
- Linux ext4 admission, authority, and durability laws survive extraction of the
  native boundary: established by the Linux CI jobs on the merge candidate.
- The platform work changed no format and no golden fixture.
- No unsafe code and no new dependency: `Cargo.toml` and `Cargo.lock` are unchanged.

## Native execution

Darwin 27.0.0, arm64, pinned `rustc 1.96.0`, under `scripts/native/check.sh`.

| Check | Default APFS | Case-sensitive APFS |
| --- | --- | --- |
| `cargo test --all-features` (keep package, debug) | 963 passed, 0 failed | 963 passed, 0 failed |
| `cargo xtask durability-crash-matrix` | exit 0 | exit 0 |

The volumes are sparse images: default APFS `data` and case-sensitive APFS
`data-cs`, 256 MiB each, selected with `KEEP_NATIVE_DATA_IMAGE`. The worker points
`KEEP_TEST_SCRATCH_ROOT` at the selected image and sets
`KEEP_TEST_EXPECT_APFS_VARIANT`, and the spelling laws fail if the volume is the
other variant. Release-profile tests and the optimized crash matrix run in the
`Native macOS storage` CI job. `xtask` tests do not run on macOS.

Two earlier sets of native runs are not counted. One used Homebrew's Rust 1.98
ahead of rustup on `PATH`; the worker now selects the pinned toolchain and asserts
its version. Another claimed both variants, but the test sandbox ignored `TMPDIR`,
so both runs used the default-APFS build image. Independent review found this.

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

## Volume reattachment

Darwin assigns `st_dev` per mount and reuses a freed number. Mounting another
volume in the gap moved a volume from 16777251 to 16777255, with its root inode
unchanged. No record names that number, since
[ADR-0010](../adr/0010-one-storage-format.md) removed the persisted root binding.
`tests/darwin_volume_reattach.rs` still forces the renumbering on a real volume and
requires the store to reopen. Reboot itself has not been exercised.

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

## Known refusal: Finder metadata

Finder can write `.DS_Store` into a folder it displays. The namespace admits no
entry it cannot account for, so the store then refuses until that file is removed.
`a_finder_metadata_file_in_the_root_refuses_until_it_is_removed` pins this. It
fails closed and loses nothing, but an operator browsing a store in Finder will
hit it, and whether to tolerate the name is an owner decision.

## Crash matrix scope

The matrix uses repository entry points that skip the production platform probe,
as it does on Linux. It exercises the protocols and the spelling boundary on APFS,
and separate public laws exercise production admission.

## Parent scan cost

One release-mode run on the default APFS image, 200 opens of one existing file, using
an unrepeated throwaway test. The cost grows with the directory, as expected.

| Directory entries | Plain open | With spelling check |
| --- | --- | --- |
| 10 | 25 µs | 74 µs |
| 1,000 | 9 µs | 282 µs |
| 10,000 | 11 µs | 475 µs |
| 50,000 | 17 µs | 3.4 ms |

This is the per-open overhead only, not a store workload.

## Not measured

- `F_FULLFSYNC` durability. A successful call is not proof that hardware honored
  it.
- A real non-APFS or network volume at the public boundary.

## Runner bounds

Compilation and test output stays on the reusable 1.5 GiB `build` image, 353 MiB
used after a clean run. Each data image stayed under 7 MiB used, and logs total
1.9 MiB. The sandbox denies network except local Unix sockets, which the process
laws need, and denies writes outside the build and selected data images.

The runner watches only its own process group, so a law that starts its own group
is bounded by that law's watchdog, not by the runner.
