# Reader collector digest and I/O evidence

Change kind: test coverage and oracle repair. Subject: returned view contents and source-preserving errors at Keep's public `RetentionViewSource` contract. Owner: `@flyingrobots`.

At parent `e0d0893`, collector tests covered generation changes but omitted same-generation digest changes and the initial-coordinate, view-load and final-coordinate I/O failures.

The revised tests return distinct superseded and stable payloads and require the stable payload after a coordinate change. Separate deterministic sweeps change every nonzero first digest byte for catalog and retention heads while preserving generation and other coordinates; this is a bounded digest domain, not exhaustive hash-space coverage.

Each failing read must return `RetentionViewError::Io` preserving the original raw error code. The synthetic code 123 is a controlled opaque source value, not a claim about an operating system's interpretation or an actual kernel fault.

Independent copied-source mutations ignore catalog digests, ignore retention digests, reconstruct I/O errors from their kinds, or reclassify them as catalog absence. The revised tests fail at the intended returned-payload, original-cause or error-variant checks, respectively. These are assertions on the running collector's outputs through its public port; they do not claim filesystem integration coverage.

The parent collector tests were then run against each same production mutation with fresh build targets and passed, demonstrating the original coverage gaps separately from the revised tests' observed RED results.

The old load-count assertions are removed because they read script internals. The review suggestion to assert an empty script queue is intentionally declined for the same reason. Bounded refusal remains asserted through `AttemptsExhausted { attempts: 2 }`; absence remains asserted through `CatalogAbsent`; successful collection and retry remain asserted through returned payloads.

Debug and release collector laws, both workspace Clippy feature configurations, formatting and source-structure checks passed in copied Docker. Initial Clippy rejected an unnecessary result type on the now-infallible absence test; that signature was corrected before the final checks. Markdown validation is separate static evidence.

Replay uses `cargo test --lib retention_view_collector --all-features`, adding `--release` for optimized execution. The sweeps are deterministic, consult no ambient randomness and report their mismatched coordinates on failure. Per-test resource-ceiling enforcement and broader physical-filesystem race coverage remain outside this change.
