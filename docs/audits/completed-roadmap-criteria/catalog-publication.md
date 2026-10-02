# Catalog publication admission audit

This page owns the current T-12.2 verdict from the originally checked roadmap
at `1a586d83d5750083172d440f90e7b786d540ff0e`, lines 546–548. Inspected main
is `b50dbd4cb4cee286aea1aa0352152a232197dda1`. T-12.1 and T-12.3 remain under
review; the checks below do not establish their complete definitions of done.

## T-12.2 — Writer exclusion and platform-admitted publication

**Acceptance/definition of done: not met across supported features.** The
ordinary publisher constructor consumes a private-field
`FilesystemPlatformAdmission`. Writer locking and the publication state
machine have executable exclusion, ordering, refusal and recovery evidence.
However, `repository-tasks` exposes a public alternate constructor accepting
only `FilesystemWriterLock`. Its admission producer performs lenient root
identity observation without enforcing the production platform profile.

The path is
`src/adapters/filesystem_catalog_publisher.rs::open_unchecked_for_repository_tasks`
through
`src/adapters/filesystem_platform_admission.rs::unchecked_for_repository_tasks`
to the lenient identity probe in `filesystem_platform_profile.rs`. The
existing architecture law checks the ordinary constructor and admission type
text, so it does not cover this alternate public route. The production crash
harness consumes that route after unchecked repository initialization.

A temporary public API probe on an actually unsupported Linux Docker
filesystem first verified production initialization's exact
`AdmitPlatform/Unsupported` refusal. It then created the canonical namespace,
acquired the writer lock, and called the public repository-task constructor.
The constructor returned a publisher. The refusal law failed with
`refused platform acquired public publisher authority` on the exact inspected
main source, using a dedicated build directory. No admission flags were
forged, no bad artifact was published, and no host test was run.

This demonstrates a platform-admission capability bypass, not corruption or
physical power-loss failure. The original platform-admitted publication
criterion and the feature-invariant rule remain binding. Executable
[issue #150](https://github.com/flyingrobots/keep/issues/150) owns the public
boundary regression, correction and affected crash-harness adaptation as one
independently mergeable outcome. Shared harness files alone do not establish
a prerequisite on the separate sealed-stage escape in #146.

## Executed evidence and remaining limits

Eleven catalog integration targets passed 59 laws in Docker debug and 59 in
release on `8d902516e682361882bc5c9902de296ce5c9de85`. The two ordering laws
and sixteen filesystem publisher fixture laws also passed in each mode.
These fixtures use an unchecked test publisher and do not prove production
platform admission. The failure probe ran separately on the inspected newer
main source; passing existing fixture laws does not negate that failure.

The catalog ledger's absent ordering and filesystem publication evidence
owners are being corrected by
[PR #149](https://github.com/flyingrobots/keep/pull/149) for #148. Those
reference corrections do not resolve T-12.2 or close #150. No fresh crash
campaign or host-power-loss evidence is claimed here.
