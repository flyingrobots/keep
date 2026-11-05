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
| `cargo test --all-features` (keep package, debug) | 937 passed, 0 failed | 937 passed, 0 failed |
| `cargo xtask durability-crash-matrix` | exit 0 | exit 0 |

Both rows are at `fb355d66`, and CI run 41953913643 reported the same 937 at that
head. Later commits add laws, so a later head has a different count; this table
does not state it.

The volumes are sparse images: default APFS `data` and case-sensitive APFS
`data-cs`, 256 MiB each, selected with `KEEP_NATIVE_DATA_IMAGE`. The worker points
`KEEP_TEST_SCRATCH_ROOT` at the selected image and sets
`KEEP_TEST_EXPECT_APFS_VARIANT`, and the spelling laws fail if the volume is the
other variant. Release-profile tests and the optimized crash matrix run in the
`Native macOS storage` CI job. CI does not run `xtask` tests on macOS;
[local `xtask` tests on macOS](#local-xtask-tests-on-macos) records what a local
run shows.

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

## Single-format store paths

The initialization, marker, admission, and namespace code from ADR-0010 uses the
same exact-name directory as every other path. `darwin_filesystem_tests` pins the
per-lookup check below namespace admission for `FORMAT`, `HEAD`, and a protocol
directory. Those three unit laws are the per-lookup witnesses for these paths.

`tests/darwin_store_spelling.rs` respells `HEAD`, `FORMAT`, and `segments` and
requires the public reader and writer to refuse, and requires initialization to
refuse a root that holds `Format`. As the calibration above shows, namespace
admission refuses those entries first, so these laws witness the public refusal
and not the per-lookup check. No public-route law respells a pool entry, such as
a segment or catalog file name.

## Volume reattachment

Darwin assigns `st_dev` per mount and reuses a freed number. Mounting another
volume in the gap moved a volume from 16777251 to 16777255, with its root inode
unchanged. No record names that number, since
[ADR-0010](../adr/0010-one-storage-format.md) removed the persisted root binding.
`tests/darwin_volume_reattach.rs` still forces the renumbering on a real volume and
requires the store to reopen. Reboot itself has not been exercised.

## Laws that remain Linux-only, and why

- tmpfs refusal (`reader_platform_law_tests`,
  `initialization_refuses_an_unsupported_platform_before_any_mutation`): no tmpfs.
  On Darwin, non-APFS refusal uses devfs, at the adapter
  (`filesystems_other_than_apfs_are_refused`) and at the public boundary
  (`initialization_refuses_devfs_at_platform_admission` calls
  `FilesystemStoreAdmission::initialize` on `/dev`). Read-only APFS refusal uses
  the sealed system volume, at the adapter only.
- Raw non-UTF-8 names: APFS refuses to create them.
- `filesystem_retention_fifo_tests`: Rustix omits `mknodat` on Apple targets and
  `std`'s `mkfifo` is unstable. `tests/darwin_fifo_refusal.rs` covers a FIFO at
  `HEAD` only.
- O_PATH directory synchronization and ext4 profile laws: Linux mechanisms.

Two laws left this list:

- `a_writer_lock_cannot_mint_...` was deleted with
  `tests/catalog_platform_admission.rs`. Its successor is the `compile_fail`
  doctest on `FilesystemCatalogPublisher::open`, which shows on every platform
  that a bare writer lock cannot open a publisher.
- `filesystem_catalog_recovery_tests` was compiled only on Linux. The gate is
  removed, and its three laws pass natively on default and case-sensitive APFS.

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
- A network volume, or a non-APFS volume other than devfs, at the public
  boundary.
- A read-only APFS volume at the public boundary.

## Local `xtask` tests on macOS

CI runs `xtask` tests on Linux only, and Linux passing clears no macOS-only
cause. A local run under `scripts/native/check.sh`'s environment at `fb355d66`
failed nine `xtask` laws:

- **Two, diagnosed by experiment and fixed on this branch.**
  `terminal_interrupt_terminates_the_isolated_descendant_group` and
  `cleanup_terminates_the_entire_child_process_group`, in
  `bounded_process::process_group`. The runner's scratch directory is a long
  path, and the laws' Unix socket path exceeded the 104-byte `sun_path` bound on
  macOS, so the child exited before it was ready. With short names both pass
  natively. `ProcessGroup` was not at fault. This matters because the crash
  matrix terminates its children through the same `ProcessGroup`.
- **One, a missing tool.** `external_oracle_agrees_on_the_identity_preimage`
  needs the `b3sum` binary, which is not installed on this machine. It fails
  the same way on unchanged `main`.
- **Six, caused by the runner, read from their messages.** Five `cli_contract`
  laws and `successful_verification_runs_every_documentation_tool_silently`.
  The runner sets ambient `CARGO_*` build settings that the tool refuses, and
  its sandbox denies writes outside its images. One law's message names the
  refused setting, one reports `PermissionDenied`, and four report only a
  failed command. None has been re-run without those two conditions.

## Runner bounds

Compilation and test output stays on the reusable 1.5 GiB `build` image, 353 MiB
used after a clean run. Each data image stayed under 7 MiB used, and logs total
1.9 MiB. The sandbox denies network except local Unix sockets, which the process
laws need, and denies writes outside the build and selected data images.

The runner watches only its own process group, so a law that starts its own group
is bounded by that law's watchdog, not by the runner.
