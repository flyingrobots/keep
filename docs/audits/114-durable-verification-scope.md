# Durable verification landing scope

Status: implementation work in progress for [#114](https://github.com/flyingrobots/keep/issues/114), under verification parent [#20](https://github.com/flyingrobots/keep/issues/20).

This ledger reconciles the requested verification outcome with the code available at the branch baseline; it does not establish a runtime guarantee or mark an acceptance criterion complete.

## Source of authority

The branch starts at `origin/main` commit `6051abb25a9fd33ae7ee0de5614514b709a4d82a`.

The original T-21.1 task fields in `ROADMAP.md` at prepared-branch commit `66c0e4653424cd36e55a868c24d94898a40aca59` remain authoritative, including per-subject achieved depth, typed diagnostic coordinates, unsupported-depth refusal, immutable report construction, bounded costs, and no repair.

The prepared branch's `docs/invariants/verification/requirements.md` marks `KEEP-VERIFY-006` Planned; its other Implemented entries describe that branch and must not be copied into main as evidence of delivery.

## Dependency finding

Main has durable catalog admission, fenced version-two retention snapshots, and retention-closure verification, but no `src/verification/` report domain.

The issue's reference to an existing policy/report domain therefore describes an unmerged prerequisite, not an available mainline API.

The implementation must explicitly supply the required domain within this coherent change or wait for its separately reviewed mainline integration; it must not import the unrelated prepared feature branch wholesale.

PR #164's authenticated read conveniences are not an established prerequisite: the relevant catalog and retention evidence boundaries already exist on this baseline.

No new tracker dependency is recorded by this document.

## Existing evidence boundaries

| Boundary | What the inspected code establishes | What it does not establish |
| --- | --- | --- |
| `AdmittedSegment::decode` through `segment_reader` | Header/seal admission, bounded record admission, physical segment digest, and logical record identities. | Full reconstruction of every blob described by a layout. |
| `FilesystemCatalogSnapshot::load` and `snapshot` | Exact head-selected catalog coordinates, selected segment admission, and catalog-to-record bindings; owned segment bytes are bounded by caller policy. | Retention authority, every layout's chunk closure, or a general shallow-depth diagnostic pass. |
| `FilesystemRetentionSnapshot::load` | Version-two admission, shared reader fence, and bounded double collection of catalog and retention coordinates. | Closure verification of all roots selected by the manifest. |
| `FilesystemRetentionSnapshot::retained_root` | Manifest-selected root bytes, canonical decoding, and selected generation/digest agreement. | Complete root closure; some failures currently lose structured coordinates in message-only I/O errors. |
| `verify_retention_closure` | Bounded root traversal, required catalog members, anchor-to-layout binding, profile replay, and complete blob identity for each anchor. | A whole-store completeness claim for unretained records or a reusable general verification report. |

These are inspected implementation boundaries, not new execution receipts.

## Closure ledger

Catalog, segment and logical-record reporting now have [runtime and static/API evidence](../testing-evidence/durable-verification.md); every full-issue obligation below remains open until its entire exit condition is met.

| Obligation | Required implementation boundary | Concrete exit condition |
| --- | --- | --- |
| Required subjects and depths | Domain vocabulary and adapters over segment, catalog, layout/blob, and retention evidence. | Every original durable subject/depth has a documented supported operation or precise unsupported result; v1/v2 corpus and empty-store outcomes establish the advertised matrix. |
| Requested versus achieved evidence | Private report construction with a verified entry per subject; adapters construct entries only after the corresponding checks finish. | A shallow request cannot certify an unchecked deeper claim; unsupported requests refuse; external callers cannot construct or deepen evidence. Runtime laws and static/API laws are identified separately. |
| Missing, corrupt, ambiguous, operational | Semantic error admission at durable boundaries, preserving original typed causes. | Absence, demonstrated contradiction, conflicting evidence, and failed observation produce distinct public outcomes with the available expected/observed coordinates and bounded conflict evidence. No classification depends on parsing a message. |
| Consistent snapshot | Existing immutable catalog ownership and fenced retention collection. | Reports bind the exact observed coordinates; moving views cannot combine evidence from different attempts; retained-root closure is checked against that same catalog. |
| Bounded cost and report contents | Caller-bounded catalog/segment loading, root-at-a-time work where applicable, and bounded report data. | Public documentation states I/O, allocations, memory, blocking, and complexity per supported operation; catalog-ceiling evidence checks the stated bound; reports contain no plaintext, keys, or unbounded paths. |
| Read-only behavior | Verification uses observation and admission capabilities. | Refusals and success leave persistent bytes unchanged; no repair, recovery, retention publication, or GC is triggered. |
| Honest delivery claim | Normative verification page, rationale, requirement ledger, public rustdoc, and consolidated execution evidence. | `KEEP-VERIFY-006` becomes Implemented only when the durable contract is implemented and verified; mainline delivery is recorded only after integration. |

## Design constraints for implementation

Depths describe checks on a particular subject; the ordinal position of `CatalogReachability` must not be used as evidence that every catalogued layout reconstructs a complete blob.

An already admitted snapshot may contain evidence beyond a shallow request, but its construction cost and refusal behavior must be disclosed; it cannot be presented as a framing-only scan that succeeds despite a deeper checksum failure.

A report must distinguish requested policy from established evidence without treating failed or unattempted work as verified.

Snapshot binding from the original future F-19 obligation must not be conflated with the existing catalog and retention coordinates.

The prepared reference report only names blob/layout subjects and reserves ambiguity without candidate coordinates; copying it unchanged would not satisfy the original durable acceptance criteria.

The current root reader's message-only errors require a focused boundary decision before they can feed truthful typed verification refusals; this does not authorize a repository-wide filesystem audit.

## Evidence and exclusions

This is documentation-only scope reconciliation, based on source inspection at the two commits above; no runtime RED/GREEN claim is made for this ledger.

New runtime assertions must be calibrated against the behavior they protect; absence of a new API on the parent is a compile failure, not a behavioral RED receipt.

Bug fixes discovered at existing boundaries require a runtime regression observed on the unfixed revision, preserving the original failure artifact.

Durable report serialization, a new report decoder, repair, GC execution, remote attestation, application trust policy, and unrelated prepared-branch features remain outside this issue.
