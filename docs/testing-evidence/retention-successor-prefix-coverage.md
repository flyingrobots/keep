# Successor publication prefix recovery

Change kind: test coverage and oracle repair. Owner: `@flyingrobots`. Subject: the runtime state visible after recovery of an interrupted successor publication (#99).

At parent `d353252`, the successor law exercised only prefixes 2, 9, 13, and 15, while the requirement ledger suggested exhaustive prefix evidence.

The revised law exercises every ordered phase prefix from 0 through 18 in a fresh migrated store and loads a public `FilesystemRetentionSnapshot` after recovery.

The observable oracle requires generation 1 and the original root bytes before completion of the head-stage write at prefix 12, and generation 2 and the prepared successor's exact bytes thereafter.

The expected root payload is the publication input, independent of recovery receipts; its decoder supplies only the namespace used to request the selected root.

Calibration deletes the committed immutable root immediately after recovery removes its root stage, while preserving the successful recovery receipt: the parent successor test passes, but the revised test fails when the public reader reports the missing root.

A separate calibration changes the public reader to return `wrong` after its normal verification: the revised test fails at `successor prefix 0: exact selected root bytes`.

Both calibrations use disposable copied Docker trees and fresh build targets; the production code in the final change is unchanged.

The expanded law passes in copied Docker debug and release with `cargo test --lib successor_prefixes_recover --all-features`, adding `--release` for optimized execution.

Both workspace Clippy feature configurations with `-D warnings`, formatting, and `cargo xtask source-structure-check` pass.

This evidence covers ordered operation boundaries, not arbitrary byte offsets, power loss, or every possible syscall interruption.

The separate mid-write law samples one 100-byte interruption in each of the root, manifest, and head stage writes; the ledger now states that scope explicitly.

Independent reader assertions after process death in the durability crash runner remain a separate open review finding; this in-process law does not close it.
