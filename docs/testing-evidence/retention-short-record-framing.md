# Interrupted root and manifest framing

Change kind: bug fix. Owner: `@flyingrobots`. Subject: preserving short stage headers whose declared size contradicts complete framing fields (#99).

Regression `c817fa0` is observed RED on unfixed production `0868675`: a 48-byte root prefix declaring zero length is successfully discarded as a clean recovery.

Once all size fields are available, prefix admission now reuses each decoder's checked canonical-size calculation and declared-length comparison.

The filesystem law covers root and manifest stages with zero, actual-length-plus-one, and maximum unsigned declared lengths, requiring the exact stage and `DeclaredLengthMismatch` coordinates plus unchanged retained evidence.

Expected sizes come from the original golden record bytes, independently of the framing functions used by admission.

Three disposable production mutations mislabel the root corruption as a head, swap expected and observed size coordinates, and delete the root stage while returning the correct error; each fails its corresponding named runtime assertion.

Recovery suites, the focused framing law, and canonical-prefix integration tests pass in copied Docker debug and release; the retention process-death matrix also passes.

Both workspace Clippy configurations with `-D warnings`, formatting, and source-structure checks pass.

Replay uses `cargo test --lib contradictory_lengths_in_short_stage_headers --all-features` and `cargo test --test retention_stage_prefix --all-features`, adding `--release` for optimized execution; calibration uses separate copied trees and fresh targets.

This closes consistency of complete framing-size fields only; bounds and semantics of other root/manifest fields, partially available numeric fields, and incomplete-stage pinning remain under audit.
