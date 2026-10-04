# Interrupted head manifest-length admission

Change kind: bug fix. Owner: `@flyingrobots`. Subject: preservation of corrupt evidence when an interrupted head contains a complete invalid manifest-length field (#99).

The regression in `f45a748` runs against unfixed production from `02f710b`, creating real linked root and manifest stages and a 40-byte head prefix whose complete length field is invalid.

The zero-length case fails RED because recovery returns a successful receipt executing `DiscardHeadStage`, instead of the expected typed corruption refusal.

Prefix admission now calls the existing semantic manifest-length constructor once all eight length bytes are available, returning `RetentionHeadDecodeError::ManifestLength` on refusal.

The filesystem law covers zero, just below the minimum, a within-bounds misaligned length, just above the maximum, and the largest unsigned value; each must return `Plan/StageCorrupt(Head)` with exact error coordinates and preserve the retained-file witness.

The expected bounds and alignment cases are specified directly from the version-two format rather than computed with the admission constructor under test.

A disposable production mutation deletes `head.next` while returning the correct planning error; the preservation assertion fails at `invalid short head length 0 must preserve retained evidence`.

Recovery suites and the focused short-head law pass in copied Docker debug and release; the retention process-death sequence also passes, retaining valid interrupted-write recovery behavior.

Both workspace Clippy configurations with `-D warnings`, formatting, and source-structure checks pass.

Replay uses `cargo test --lib invalid_manifest_lengths_in_short_heads --all-features`, adding `--release` for optimized execution; the process-death replay is `cargo xtask durability-crash-matrix --sequence retention`.

This closes the complete head manifest-length field case only; other variable fields, partially available numeric fields, and incomplete-stage pinning remain under the broader open recovery audit.
