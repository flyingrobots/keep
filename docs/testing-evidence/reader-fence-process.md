# Reader-fence process evidence

This change addresses #113 and the process-death/exclusion portion of original T-19.1 and KEEP-RETENTION-008. Change kind: missing verification, with no production behavior change. Owner: `@flyingrobots`. The branch starts from main `6051abb25a9fd33ae7ee0de5614514b709a4d82a`, including #99. This record does not claim mainline delivery before its PR merges.

## Contract and observations

`tests/reader_fence_process.rs` loads a real `FilesystemRetentionSnapshot` in a separate process over a fresh migrated golden store. The child sends readiness only after public snapshot admission completes and keeps the snapshot alive until an explicit release message or SIGKILL. The parent takes actual writer authority before acting as a collector and uses the protocol's exclusive kernel lock on the persistent `reader.lock` file.

`reader_death_releases_collection_without_replacing_the_fence` requires exclusive acquisition to return exactly `EWOULDBLOCK` while that snapshot lives. After killing and reaping the reader with a verified SIGKILL result, the same collector descriptor must acquire the exclusive lock. The persistent pathname must still name the original device/inode and have zero length. This is kernel process-death evidence, not a Rust destructor simulation.

`collection_excludes_a_new_snapshot_until_release` holds the exclusive fence before spawning a reader. It requires Linux `/proc/locks` to report a queued shared flock for that exact child PID and the owned fence inode. A child that announces snapshot readiness before appearing in that queue fails immediately. The parent then releases collection authority and requires successful snapshot admission and normal child completion. The kernel queue is the blocking witness; elapsed time without a message is never accepted as proof of exclusion.

The fixed schedules are live-reader → exclusive refusal → SIGKILL/reap → exclusive admission, and exclusive collector → queued reader → collector release → admitted reader. Unix-domain channels establish readiness. Polling yields to the scheduler without sleeps. Twenty-second watchdogs only fail stuck executions; they cannot produce a passing exclusion result. The tests do not explore arbitrary interleavings, physical power loss, or a complete garbage collector implementation.

## Calibration and replay

The unmodified product passes both laws. In a separately copied source/build tree, replacing shared-lock acquisition with an unlock makes both laws fail: the live-reader check observes `Ok(())` instead of `EWOULDBLOCK`, and the collector-exclusion check reports `snapshot escaped the exclusive collector fence`. This is an observed runtime calibration of missing evidence, not a claim that main contains a production locking bug. The mutation is not included in the branch.

Run `cargo test --all-features --locked --test reader_fence_process` and its `--release` variant. The tests are Linux-only and require `repository-tasks` and a readable `/proc/locks` for their PID namespace; unsupported platforms do not supply this evidence. Their subprocess inherits the same executable and exact test name, keeping process creation outside `src`. Immutable candidate SHA, toolchain and validation results are recorded in the PR.

These are medium-size, single-machine tests with owned filesystem scratch and Unix-domain sockets, no network service, ambient identity selection or random input. The fixture uses repository-only initialization/migration admission to isolate fencing; this is not proof of production platform eligibility. Kernel observations are restricted to the owned child and lock. Child cleanup kills/reaps on failure. Existing ordinary-test memory/resource-ceiling gaps remain as documented in the enforcement profile; watchdogs are not a memory sandbox or suite latency SLO.

Retire these laws only if the reader-fence contract disappears or stronger process/schedule evidence subsumes the same guarantees. Existing same-process sharing, fence substitution, corruption and snapshot-consistency tests remain. No format, API, identity, recovery protocol or performance behavior changes.
