# Reader fence visibility

Status: accepted.

Change kind: deliberate API surface reduction, with no runtime behavior change.

`FilesystemRetentionSnapshot` owns the reader fence for the snapshot's lifetime; no public operation accepts, returns, or constructs the fence itself.

The previously exported `ReaderFence` exposed an unusable implementation detail and invited an unnecessary compatibility obligation, so its declaration is now visible only within retention, its parent import is private, and its crate-root export is removed.

This changes the unreleased source API without changing locking, snapshot lifetime, durable encoding, or recovery behavior.

Static before/after evidence is the public declaration and two export sites at parent `429e3f7`; compiler validation and the existing reader-fence and snapshot runtime laws check that internal consumers continue to work.

No new runtime assertion is added for visibility: source/API evidence establishes this change, while the existing behavioral laws retain their independent oracles and calibration evidence.
