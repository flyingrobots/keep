# One unchecked version-two reopen path

Change kind: refactoring. Owner: `@flyingrobots`. Subject: duplicated repository-only version-two admission constructors (#99).

At parent `3f2d7d8`, the test-only and repository-task constructors both opened an ambient directory, mapped the same platform error, and called `Self::admit` with identical code.

The test-only duplicate is removed; its callers use `reopen_unchecked_for_repository_tasks` under `cfg(any(test, feature = "repository-tasks"))`.

Normal library builds without the repository-task feature still expose only the production `reopen` entry point, which performs platform admission before the shared namespace, lock, record, and identity checks.

The misplaced capability-release comment now documents `into_parts`, rather than the unchecked constructor.

Existing test expectations are unchanged; only constructor calls move to the retained implementation, so no new assertion or fabricated bug reproduction is introduced.

Validation runs the complete library suites with all features in debug and release, the version-two admission suite without default features, both workspace Clippy configurations with `-D warnings`, formatting, and source-structure checks in copied Docker.

The generated model sequences and existing filesystem admission, recovery, and refusal laws continue exercising the same shared admission implementation through the consolidated entry point; these finite checks do not prove equivalence for every possible filesystem state.
