# Exact retention model refusals

Change kind: test-oracle bug fix. Owner: `@flyingrobots`. Subject: the runtime diagnostics returned by rejected retention operations, alongside the existing persisted-state model.

At base `5242b05`, the model accepted any error for a refused operation, including unrelated preparation failures and operating-system errors; unchanged stored state could therefore conceal an incorrect diagnostic.

The model now distinguishes repeated initial roots from stale publications: the former must return `ManifestSuccessorMismatch` with the selected namespace, root generation and digest, an initial candidate generation and no predecessor; the latter must return `CurrentVerification` containing `Superseded` with the current liveness generation and manifest digest.

The model determines acceptance from its namespace map and liveness generation; diagnostic coordinates come from the admitted pre-operation manifest, never from the returned error. That coordinate oracle shares the production decoder and therefore does not independently establish codec correctness; independent canonical-format tests remain necessary.

Calibration changed actual production refusal paths in isolated copied trees. Replacing the preparation mismatch with `ManifestEntryIndex` passed the old model and failed the revised mismatch assertion. Replacing the superseded cause with `PermissionDenied` passed the old model and failed the revised exact-superseding-manifest assertion. These observed runtime RED results establish the oracle repair; the unmutated product passes the tightened checks.

An additional mutation replaces the publication error wrapper with `MissingPublicationArtifacts` to calibrate the current-verification boundary separately.

The sequence-count assertion was deleted because it froze harness structure without asserting Keep behavior. No runtime risk moves to another count: the suite retains publication outcomes, exact refusals, manifest entries, liveness generation, root generation and anchor-set checks.

These are medium filesystem tests with deterministic enumeration of the existing short operation histories. Replay uses `cargo test --lib retention_model_tests --all-features`, adding `--release` for optimized execution, inside the copied Docker runner. They do not claim arbitrary-length history exploration, automatic shrinking, independent decoder validation or newly enforced per-test resource ceilings.

Debug and release model runs and both workspace Clippy feature configurations passed, with formatting and source-structure checks. Final metadata-only edits received a further focused debug run. Markdown lint validates this receipt separately from product behavior.

The initial preparation mutation failed compilation because its replacement left unused arguments; that setup result was excluded and the corrected mutant was rerun. An additional wrapper-calibration copy initially exhausted the disposable filesystem's file capacity; completed artifacts were preserved outside it before replay. Raw setup failures, mutant runs and successful unmutated runs remain separate evidence.
