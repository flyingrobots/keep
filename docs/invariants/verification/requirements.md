# Verification requirements

The [original task](../../audits/114-durable-verification-scope.md) remains authoritative; partial subject reporting does not satisfy the full durable requirement.

| ID | Requirement | Status | Evidence and remaining work |
| --- | --- | --- | --- |
| `KEEP-VERIFY-006` | Durable verification at explicit subject-specific achieved depths, with precise refusals, immutable reports and bounded costs. | Implemented on main through #165 | The implementation provides subject-specific catalog, segment, record, blob and retained-namespace reports, typed durable ingress outcomes and bounded conflict evidence; focused runtime/calibration and catalog-ceiling evidence are recorded. Exact-head validation, independent acceptance and mainline integration are recorded on [PR #165](https://github.com/flyingrobots/keep/pull/165); the signed merge preserves the independently reviewed candidate tree. See the [closure ledger](../../audits/114-durable-verification-scope.md) and [execution evidence](../../testing-evidence/durable-verification.md). |

## Reference and receipt integration

The following additions are implemented in the #107 integration candidate; final acceptance still requires the complete integration review and validation.

| ID | Requirement | Evidence and scope |
| --- | --- | --- |
| `KEEP-VERIFY-001` | Reports establish exactly the requested, subject-supported depth without a public upgrade capability or global depth ordering. | `tests/verification_report.rs`; private report construction; `VerificationDepth` compile-fail ordering law. |
| `KEEP-VERIFY-002` | Unsupported requests name their exact supported set rather than silently downgrading. | `tests/verification_report.rs`; reference supports chunk, layout and complete-blob identity only. |
| `KEEP-VERIFY-003` | Missing, corrupt and ambiguous evidence remain distinct, with typed causes and observed coordinates. | Reference report/refusal laws, durable ingress laws and the existing #165 evidence above; reference views do not produce ambiguity. |
| `KEEP-VERIFY-004` | The reference verifier authenticates chunks once and preserves chunk-refusal precedence over profile/complete-blob failures. | `tests/verification_report.rs`, `src/reference/verification_tests.rs`; no ordering implication is made across unrelated subjects. |
| `KEEP-VERIFY-005` | Verification does not repair, substitute, quarantine or rewrite evidence. | Immutable reference API and tampered-chunk preservation laws; durable snapshot evidence remains governed by #165. |
| `KEEP-VERIFY-007` | Canonical v1 receipts preserve their frozen subject, view, depth and refusal vocabulary without manufacturing runtime provenance. | `tests/verification_receipt.rs`, frozen fixtures and receipt fuzz target. Checked live projection supports reference-origin evidence only; historical durable-view decoding is a codec capability, not live durable-report projection. |
