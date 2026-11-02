# Filesystem platform adapter evidence

This is an implementation record, not platform certification. The design is
in [filesystem platforms](../architecture/filesystem-platforms.md).

## Claims and current status

- Product: native production storage on ordinary and case-sensitive APFS.
  **Pending implementation and native runtime evidence.**
- Product: Linux ext4 admission, authority, and durability laws survive the
  extraction of the native mechanism boundary. **Runtime validation pending.**
- Static/API: focused protocol ports and public constructors remain available.
  Source inspection establishes the intended preservation; compilation pending.
- Tool: bounded native compilation without writes outside admitted outputs.
  Runner implemented; execution and guard calibration pending.
- Documentation: architecture and decision record delivered unchanged to Reader.
  Receipt `f8a83855-f07e-4622-ac91-25b5bcf72b34`, `awaiting_filing`,
  `integrity: intact`, for design commit `ba99881`.

The author owns the evidence; repository owner `@flyingrobots` owns policy.
No missing gate is treated as an approved waiver.

## Native mechanism observations

On the workstation's existing writable APFS data volume, read-only descriptor
observations established that `README.md` and `readme.md` resolve to the same
device/inode. `F_GETPATH` returned the actual stored spelling in both cases.
A bounded one-byte fixture with two hard links returned each opened link's
actual spelling, including when one was opened using an uppercase alias.
The fixture was removed. `F_FULLFSYNC` on an open repository directory returned
success. These observations motivate the adapter; they are not permanent
production tests, a concurrency proof, or a power-loss experiment.

The pinned Rust 1.96 standard library's Apple implementation of
`File::sync_all` calls `F_FULLFSYNC`. A new adapter must not weaken that call.

## Native compilation launch contract

The initial compiler-only runner is `scripts/native/check.sh`. It does not
launch test executables or crash campaigns. Its admitted child tree is Cargo,
the pinned compiler, linker, and admitted dependency build scripts; workloads
that detach process groups are excluded from this runner.

- Reused Linux worker: `keep-validation`, image `rust:1.96.0`; idle at inspection.
  The stopped historical documentation worker is `keep114-closure-docs`.
- Workstation lock store: `/Users/Shared/git-locks/workstation.git`.
  Hold `host/cache/keep-native-validation/`, `host/docker/keep-validation/`, and
  `host/docker/keep114-closure-docs/` together for project-budget admission.
- Measured Docker writable layers: 19,150,114,816 and 271,781,888 bytes.
  Counting their entire layers conservatively gives 19,421,896,704 bytes.
- Native build allocation: one reusable 1.5 GiB APFS sparse image, mounted under
  `$HOME/Library/Caches/keep-native-validation/build`. Target and Cargo-local
  metadata are the only compiler-writable build paths. Existing registry
  contents and toolchains are read-only inputs. No second target is permitted.
- The conservative existing-layer total plus the full native build capacity
  is 21,032,509,440 bytes, below the 21,474,836,480-byte project build budget.
- Native scratch: a separate 256 MiB APFS image at the adjacent `data` mount;
  `TMPDIR` points to its `tmp` directory. It is scratch, not durability evidence.
- Logs: adjacent owned `logs/`; each logger has a 4 MiB file-size ceiling.
  Launch refuses above 20 MiB already retained, bounding native logs at 24 MiB.
  Existing worker log rotation permits at most 100 MiB; combined ceilings stay
  below 128 MiB. Evidence must be exported before owned logs are recycled.
- Disk admission observed about 557 GiB host and 530 GiB Docker backing free.
  The native run monitors host free space and stops below 50 GiB. It does not
  generate Docker output while holding both worker locks.
- Hard filesystem capacities bound build and temporary output. The compiler
  sandbox denies network and all filesystem writes except target, Cargo-local
  metadata, scratch, and `/dev/null`; its log writer is separately size-limited.
- Two Cargo jobs; inherited 900-second per-process CPU ceiling and 4 GiB
  per-process virtual-address-space ceiling. The parent monitors aggregate
  process-group RSS and refuses above 6 GiB. Whole-command deadline: 1,200 seconds.
- The compiler tree has a fresh process group. Limit breach, monitor failure,
  interruption, or exit kills the owned group and waits for its leader.
  The reservation must exceed the deadline and cleanup time.

The native images are retained reusable caches, not per-revision artifacts.
Their measured usage and execution receipts must be added after validation.
This contract does not authorize another cache when one fills.

## Oracles and remaining acceptance

Shared mount-admission laws compare exact typed device/mount contradictions
and permit a child inode distinct from its parent on the same mount. Existing
Linux profile laws retain writable ext4 and casefold rejection. The old combined
child-policy unit law is separated into native profile laws and common mount
laws because those responsibilities now have different owners. Production
directory traversal and child-admission coverage still require execution.

Native product acceptance must exercise public production admission, exact
content reads, publication, retention, migration/reopen, writer exclusion,
case aliases, malformed namespaces, substitutions, and interrupted recovery.
Both APFS naming variants require actual runs. Debug/release, strict Clippy,
dependency checks, Linux regressions, CI configuration, and replayable crash
evidence remain pending. No native support or regression-free claim is made.
