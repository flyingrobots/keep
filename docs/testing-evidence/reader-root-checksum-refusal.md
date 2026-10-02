# Reader root checksum refusal

Change kind: test-oracle repair. Owner: `@flyingrobots`. Subject: the exact public reader diagnostic for a corrupted selected root.

At parent `68df384`, the filesystem test flipped the final checksum byte but accepted any `FilesystemRetentionSnapshotError::Root`, so an unrelated root failure satisfied the test.

The renamed law now requires `Root` containing `InvalidData` with `RetentionRootDecodeError::ChecksumMismatch`. Its expected checksum comes from the independently checked-in root fixture's final checksum bytes; its observed checksum comes from those bytes after the specified bit flip. It checks both fields without calling the production checksum helper to compute its expectation.

The mutation setup now refuses an empty fixture instead of silently skipping the byte flip. The test still publishes through real filesystem authority and reads the selected root through the public fenced snapshot API.

Copied-source calibration substitutes `RootDigestMismatch` for the checksum variant and, separately, swaps the expected and observed checksum fields. These exercise the named runtime diagnostic rather than the fixture or test harness.

Both mutations passed the parent test and failed the revised exact-checksum assertion. Each revision used a fresh build target, and neither runtime RED result was a setup or compilation failure.

The filesystem snapshot suite passes in copied Linux Docker in debug and release; both workspace Clippy feature configurations, formatting and source-structure checks pass. Replay uses `cargo test --lib root_checksum_damage --all-features`, adding `--release` for optimized execution. Markdown lint is separate static validation.

This evidence covers a single selected-root checksum mutation and its diagnostic fields. It does not establish arbitrary corruption coverage, physical power-loss behavior, or kernel fault handling.
