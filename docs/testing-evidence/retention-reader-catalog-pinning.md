# Retention reader catalog pinning evidence

Change kind: bug fix.

The specified oracle is that replacing an ambient store path after directory pinning cannot change the catalog selected through that pinned directory.

The medium filesystem regression `replacing_the_ambient_root_preserves_the_pinned_catalog` enters the actual `RetentionViewSource` collection contract with a real migrated store and drives path replacement synchronously, without sleeps or scheduler assumptions.

The assertion that ambient replacement must not displace the pinned catalog was observed red on unfixed head `0531fca0aa92f7a43fc1a8e3da3660de5a119e0f` with the test overlay; regression commit `ceefca2` preserves that reproduction. Initial compilation mistakes in fixture and digest accessor names were corrected before this runtime red and are excluded from the evidence.

The remaining assertions compare the returned catalog generation and digest with the original validated HEAD; this scenario calibrates successful selection, while distinct valid replacement catalogs and independent coordinate mutations remain unexercised here.

The fixed implementation passed the complete workspace all-feature test suites in debug and release, Clippy with warnings denied both with default features and all features, formatting, staged source-structure checking, and Markdown lint, using copied Docker isolation.

This evidence does not establish public admission scheduling, complete head-coordinate comparison, total resource ceilings, or absence of other reader defects.
