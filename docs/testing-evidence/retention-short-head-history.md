# Interrupted head history admission

Change kind: bug fix. Owner: `@flyingrobots`. Subject: preserving short head stages whose complete semantic fields already contradict retention history (#99).

Regression commit `de271d8` is observed RED against unfixed production `785275a`: a 104-byte generation-one head prefix with a nonzero predecessor is successfully discarded rather than refused.

The filesystem law constructs that contradiction and the complementary successor-without-predecessor case, requiring `Plan/StageCorrupt(Head)/Semantic` with each exact typed cause and an unchanged retained-file witness.

Full head decoding and interrupted-head admission now share semantic field admission; the complete decoder still checks exact length, fixed fields, and checksum first.

Interrupted recovery calls that shared admission only once all semantic fields are available, and never publishes or exposes the resulting unchecksummed head as admitted state.

The head-filtered library suites pass in copied Docker debug and release, and the retention process-death matrix passes with valid interrupted prefixes.

A disposable production mutation deleting `head.next` on the planning-error path fails the retained-evidence assertion while still reporting the correct semantic refusal.

Both workspace Clippy configurations with `-D warnings`, formatting, and source-structure checks pass.

Replay uses `cargo test --lib contradictory_history_in_short_heads --all-features`, adding `--release` for optimized execution; the calibration uses a fresh build target.

This closes complete semantic head fields only; partially available fields, root and manifest variable fields, and incomplete-stage pinning remain in the broader recovery audit.
