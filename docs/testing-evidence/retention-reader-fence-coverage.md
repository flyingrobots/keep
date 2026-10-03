# Reader fence acquisition coverage

Change kind: test correction with a behavior-preserving acquisition refactor. Subject: Keep runtime reader exclusion and fence identity. Owner: `@flyingrobots`. Oracle: the acquired shared lock must name the same empty regular-file inode as the directory entry, and an exclusive nonblocking collector must receive `Errno::WOULDBLOCK` while readers hold it.

At base `e6edb4357d89819506729ecf3016fe33750199c4`, acquisition already verified identity both before and after flock, but the test named for replacement only wrote nonempty bytes to the existing inode. That case is renamed and now requires the precise public `Fence` refusal with `InvalidData`.

The new medium filesystem law replaces the zero-length lock after the initial verified open and before flock, while the original inode remains pinned by its opened handle. Acquisition must refuse with `InvalidData` when the post-lock entry no longer names that handle.

A private scheduling callback shares the actual acquisition implementation with the ordinary no-op production path. There are no global hooks, sleeps, arbitrary scheduler races, or simulated filesystem results. The law enters the narrow reader-fence contract that owns identity verification; the existing nonempty-file law separately exercises the public snapshot's fence-error mapping. This intentionally avoids adding a second injection interface to the outer snapshot loader solely to duplicate that mapping check.

The existing shared-reader law now asserts the exact kernel contention errno rather than accepting every possible I/O error. Its final exclusive acquisition after dropping the reader fences remains the positive release control.

Fresh-target copied-source calibration removed the post-flock verification: the prior nonempty-file law still passed, while the new replacement law failed at its named assertion because acquisition returned a fence. Replacing shared acquisition with an unlock failed the exact contention assertion when the collector incorrectly acquired its exclusive lock. Because the identity protection already existed, this is not a claim that unfixed production lacked the check: these mutations expose the prior coverage gap.

Changing the identity/length refusal from `InvalidData` to `Other` failed both precise refusal assertions while preserving generic failure, demonstrating why a bare error predicate is insufficient.

The unchanged acquisition protocol passed complete workspace all-feature debug and release suites, default-feature and all-feature Clippy with warnings denied, formatting, and staged source-structure checks in copied Docker isolation. Final metadata and diagnostic-message changes passed focused debug/release fence laws, all-feature Clippy, formatting and source checks. Raw calibration and validation logs remain outside tracked source.

All affected laws are medium and use owned migrated stores on the copied Docker ext4 audit filesystem with pinned Rust 1.96.0 and Linux aarch64. Replay uses `cargo test --lib fence --all-features`; optimized replay adds `--release`. Ordinary per-test resource ceilings remain an unresolved enforcement gap.

This schedule covers replacement during acquisition, not arbitrary replacement after the final verification, hostile deletion outside the cooperative locking protocol, process-death durability, or other platforms. The change does not establish collector integration, repair other reader admission gaps, or change a durable format.
