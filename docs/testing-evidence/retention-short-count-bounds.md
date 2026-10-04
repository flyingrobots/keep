# Interrupted stage count bounds

Change kind: bug fix. Owner: `@flyingrobots`. Subject: preserving short root and manifest stages with excessive complete count fields (#99).

Regression `323578c` is observed RED against unfixed production `85d7fe9`: recovery discards a 48-byte root header declaring 65,537 anchors with a self-consistent record length.

Interrupted framing admission now calls the same count-admission functions as complete semantic header admission, rejecting excessive counts before calculating the rest of the interrupted record's framing.

Full-record decoding retains its existing checksum-before-semantic-admission order.

The filesystem law covers root and manifest counts immediately above their specified ceilings and at the largest unsigned count, while keeping the declared length consistent so a length mismatch cannot mask the count defect.

Each case requires the exact stage, maximum and observed count, and byte-identical retained evidence after refusal.

Three disposable production mutations misname the corrupt stage, replace the observed count with zero, and delete the root stage while returning the correct refusal; each fails its corresponding runtime assertion.

Recovery suites, the focused count law, and canonical-prefix integration tests pass in copied Docker debug and release; the retention process-death matrix also passes.

Clippy initially required the extracted count-admission functions to be `const`; after that declaration-only correction, both workspace Clippy configurations, formatting, and source-structure checks pass.

Replay uses `cargo test --lib excessive_counts_in_short_headers --all-features`, adding `--release` for optimized execution; calibrations use separate copied trees and fresh targets.

This closes complete count bounds only; other semantic fields, partially available fields, and incomplete-stage pinning remain in the broader audit.
