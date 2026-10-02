# Completed-roadmap audit scope

This directory continues issue #131. The criterion-by-criterion audit is in
progress; the scope manifest is not a completion verdict.

The original roadmap at commit
`1a586d83d5750083172d440f90e7b786d540ff0e` has 83 checked task entries.
The first-pass roadmap at commit
`4b9c38930f988911ab020b7c42e9221b721933af` leaves 64 checked and reopens 19.
`scope.tsv` records every original task, its original line, and its first-pass
state. Originally unchecked tasks and feature-level checkboxes are excluded.

The [foundation verdicts](foundations.md),
[identity-layer and chunking verdicts](identity-layers-and-chunking.md), and
[flat-layout verdicts](flat-layout.md), and
[reference-store and read verdicts](reference-store-and-reads.md), and
[conformance-oracle verdicts](conformance-oracles.md), and
[benchmark-baseline verdict](benchmark-baseline.md), and
[architecture verdicts](architecture.md) cover the
first 29 remaining checked tasks. Each separates acceptance and mainline
delivery and names inspected evidence and limits. T-06.3 has unresolved
acceptance scope; T-06.3 and T-06.4 are not delivered on main. T-09.1's
canonical report-admission gap is owned by issue #142. T-10.2
misses forbidden-filename enforcement; issue #144 owns that correction.

Validation now uses separate Docker build directories per source clone.
A shared target directory reused a stale test binary across clones; those
earlier runs do not establish source-specific validation. Fresh isolated
debug/release keep suites, formatting, workspace Clippy, and source policy
passed for main and the inspected PR #134/#135 implementations.

Task identifiers can have an alphabetic suffix: `T-22.1a` is a distinct
originally completed task. Counting only numeric identifiers incorrectly
produces 82 original and 63 remaining entries.

Original acceptance criteria and definitions of done remain binding. The
reviewed implementation and its integration into `main` require separate
verdicts. Existing follow-ups for reopened entries remain owned by the
[tracking container](https://github.com/flyingrobots/keep/issues/132).
