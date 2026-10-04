# Complete head without manifest refusal

Change kind: test-oracle repair. Owner: `@flyingrobots`. Subject: the precise refusal exposed by publication when restart finds a complete head stage without its manifest stage.

At parent `88eaa98`, the direct storage-attempt law and the public publication-executor law accepted any `RecoveryRefused` cause despite constructing the specific missing-manifest state.

Both laws now require `RetentionRecoveryRefusal::HeadStageWithoutManifestStage` inside the existing recovery wrapper. Their real migrated-store setup and existing no-publication or retained-evidence assertions remain unchanged.

Calibration changes the actual recovery planner to return `HeadStageNamesOtherManifest` at that branch. The observation distinguishes an absent manifest from an existing but disagreeing manifest; the wrong variant must not satisfy either public boundary's diagnostic contract.

Both parent tests passed that wrong production variant, while both revised assertions failed at their specific missing-manifest checks. The final diagnostic-message additions received a further focused debug run and all-feature Clippy check.

The attempt and publication-storage suites pass in copied Docker debug and release, with both workspace Clippy feature configurations, formatting and source-structure checks. Replay uses `cargo test --lib refused_verification_admits_no_later_phase --all-features` and `cargo test --lib retained_stage_refuses_publication_before_recovery --all-features`, with `--release` for optimized execution.

The first calibration copy exhausted the disposable filesystem's inode capacity and was excluded as a setup failure. Completed calibration trees were preserved outside that filesystem, then a complete copy and fresh build targets were used for the runtime calibration. This change claims neither new crash-schedule coverage nor a production behavior change.
