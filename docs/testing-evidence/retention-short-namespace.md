# Interrupted root namespace size

Change kind: bug fix. Owner: `@flyingrobots`. Subject: preservation of root prefixes with impossible complete namespace-length fields (#99).

Regression `1d739fa` is observed RED on unfixed production `6dde780`: a 42-byte root prefix declaring an empty namespace is discarded successfully.

Prefix admission now calls the same allocation-free domain length admission used by namespace construction as soon as the complete length field is available.

The helper becomes crate-visible for boundary admission, without extending the public API or introducing an adapter dependency into the domain.

The filesystem law covers zero, 256, and the maximum encoded unsigned length at prefixes ending immediately after the namespace length and after all framing fields; declared record lengths remain consistent with the namespace size.

Each case requires the exact root corruption and namespace cause, and an unchanged retained-file witness.

Disposable production mutations replace the namespace cause with an unrelated overflow error and delete the stage while returning the correct cause; each fails the corresponding runtime assertion.

Namespace-filtered library suites and canonical-prefix integration tests pass in copied Docker debug and release; the retention process-death matrix also passes.

Both workspace Clippy configurations with `-D warnings`, formatting, and source-structure checks pass.

Replay uses `cargo test --lib namespace --all-features` and `cargo test --test retention_stage_prefix --all-features`, adding `--release` for optimized execution; calibrations use separate copied trees and fresh targets.

Namespace bytes are opaque, so no text, path, or Unicode validation is added; other root semantic fields, partial fields, and incomplete-stage pinning remain under audit.
