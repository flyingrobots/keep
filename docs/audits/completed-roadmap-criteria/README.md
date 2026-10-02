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
[architecture verdicts](architecture.md), and
[immutable-segment verdicts](immutable-segments.md) cover the
first 33 remaining checked tasks. Each separates acceptance and mainline
delivery and names inspected evidence and limits. The individual verdicts
retain their inspected historical coordinates. T-06.3 has unresolved
acceptance scope. T-09.1's canonical report-admission gap is owned by issue
#142. The historical T-10.2 forbidden-filename gap was owned by issue #144.
T-11.2 exposes writable stage authority after sealing; issue #146 owns that
correction. T-11.3 still lacks its originally named integration-test artifact;
the living-documentation correction in #69 does not fulfill that requirement.

## Mainline delivery after the inspected snapshot

PR #135 delivered the T-06.4 capacity-bounded reference-store memory contract
as `07bf0b8f4315305e248e1628b339243e5e59ee0b`. The
[issue #74 receipt](https://github.com/flyingrobots/keep/issues/74#issuecomment-5945001183)
records exact-head validation and debug/release mainline allocation laws.
This does not claim constant total memory or a successful four-GiB ingestion.

PR #145 delivered forbidden Rust source-filename enforcement as
`88f35c417abeb62ef72c65b3a1904151cc2e3ee9`. The
[issue #144 receipt](https://github.com/flyingrobots/keep/issues/144#issuecomment-5944930436)
records mainline debug/release policy laws and exact-head validation.

PR #134 delivered single authentication per selected layout occurrence as
`8d902516e682361882bc5c9902de296ce5c9de85`. The
[issue #71 receipt](https://github.com/flyingrobots/keep/issues/71#issuecomment-5945833591)
records debug/release mainline authentication and accounting laws. T-06.3's
immediate-output criterion remains unmet; issue #71 stays open. No original
criterion or checkbox has been changed.

PR #136 delivered current v1 format documentation as
`99551ece786d47ef62ecff785a2e24261a911e85`. Its corrected ledger names the
actual filesystem unit laws. T-11.3's original named
`tests/segment_filesystem_stage.rs` integration target remains absent, so
closing #69 does not close this remaining audit gap. Issue #131 retains the
gap pending an executable correction owner.

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
