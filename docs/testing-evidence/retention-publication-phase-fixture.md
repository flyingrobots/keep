# Publication prefix fixture binding

Change kind: refactoring. Owner: `@flyingrobots`. Subject: the test fixture that creates interrupted filesystem publication prefixes (#99).

At parent `627782d`, the fixture maintained an independent positional array of storage closures beside the public `RetentionPublicationPhase::ALL` sequence.

The fixture now verifies current state once when requested, then iterates the requested prefix of `ALL` and exhaustively dispatches each phase to its matching storage capability.

The prefix bound derives from the public sequence plus current-state verification; it is an input-generation bound, not an asserted harness count.

A zero prefix still performs no storage operation, and every later prefix retains the prior verification-before-publication order.

Existing runtime assertions remain unchanged: the initial and successor prefix laws independently require their documented recovery results, and the successor law reads the recovered generation and exact root bytes through the public reader.

The expected disposition table is not derived from `ALL`, so changing the generated phase order does not automatically bless changed recovery outcomes.

Validation uses the filesystem retention library suites in copied Docker debug and release, including the complete ordered-prefix sweeps, with both workspace Clippy configurations, formatting, and source-structure checks.

No test of the fixture itself or production behavior change is introduced; the finite prefix evidence does not claim arbitrary filesystem interleaving coverage.
