# Durable verification landing scope

Status: implementation candidate for [#114](https://github.com/flyingrobots/keep/issues/114), under verification parent [#20](https://github.com/flyingrobots/keep/issues/20).

This ledger reconciles the requested verification outcome with the code available at the branch baseline; current closure dispositions and their evidence are recorded below.

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

The following table defines the unchanged acceptance exits; the disposition table below links their implementation and evidence.

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

The initial ledger was documentation-only scope reconciliation at the two commits above; subsequent runtime evidence is retained in the consolidated execution document.

New runtime assertions must be calibrated against the behavior they protect; absence of a new API on the parent is a compile failure, not a behavioral RED receipt.

Bug fixes discovered at existing boundaries require a runtime regression observed on the unfixed revision, preserving the original failure artifact.

Durable report serialization, a new report decoder, repair, GC execution, remote attestation, application trust policy, and unrelated prepared-branch features remain outside this issue.

## Interface reconciliation and current candidate

The original T-21.1 acceptance requires one `VerifiedSubject` per verified subject and explicitly names `verify_blob`, `verify_catalog`, and `verify_retention`; these operations select one subject each, so a truthful singleton subject list satisfies that contract.

The earlier normative-page references to aggregate reporting were an implementation-plan inference, now corrected; this candidate does not expose or claim a whole-store enumeration API, CLI, MCP tool or durable report serialization.

The independent bounded preflight agreed that the original named interfaces do not establish a mandatory aggregate enumerator; final exact-head review must still verify this reconciliation and the concrete subject/depth matrix.

Raw segment/catalog classification, owned filesystem catalog reports, selected-namespace retention reports, precise moving-view candidates, typed selected-root diagnostics, and the scoped catalog-ceiling allocation law are implemented in the current candidate.

The [consolidated evidence](../testing-evidence/durable-verification.md) records their runtime checks and falsification; final full validation and independent exact-head review remain acceptance gates, not assumptions inferred from earlier green commits.

The independent exact-head review of `6504c86` found one acceptance gap in existing corruption-law mapping; the follow-up retains those laws' exact assertions while exercising production verification classification, with focused debug/release and mutation evidence. Final delta review and pushed-head checks remain pending.

## Implementation disposition

| Obligation | Disposition and evidence |
| --- | --- |
| Subjects and depths | Implemented: explicit per-subject supported/refused matrix, v1/v2 corpus, empty evidence, absent members and complete blob/root closure laws. |
| Requested versus achieved | Implemented: immutable private report construction, exact subject/request/depth runtime assertions and compile-fail API laws. |
| Four failure classes | Implemented: raw and filesystem ingress, typed causes, original corruption-law mapping, exact bounded moving-view candidates; decoder/resource/classification mutations observed RED. |
| Consistent snapshot | Implemented: existing double collection and fence, exact catalog/retention provenance, rejected moving views and namespace substitution; independent mutations observed RED. |
| Bounded cost and contents | Implemented: documented ingress/admission costs and report authority, caller limits and precisely scoped catalog-ceiling allocation evidence. No total-process memory claim. |
| Read-only behavior | Implemented: unchanged filesystem evidence on success/refusal; an injected production write fails the persistent-evidence assertion. |
| Honest delivery | Normative contract, public rustdoc, rationale, requirement status and consolidated evidence reconciled. Mainline integration is not claimed before merge. |

The independent-review finding on `6504c86` is implemented and calibrated in its follow-up; approval of that delta and required checks must be recorded against the resulting exact head in [PR #165](https://github.com/flyingrobots/keep/pull/165) before it leaves draft.

This table closes implementation obligations, not the independent review or human merge gate; the PR is the live authority for those exact-head decisions.
