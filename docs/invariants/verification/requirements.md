# Verification requirements

The [original task](../../audits/114-durable-verification-scope.md) remains authoritative; partial subject reporting does not satisfy the full durable requirement.

| ID | Requirement | Status | Evidence and remaining work |
| --- | --- | --- | --- |
| `KEEP-VERIFY-006` | Durable verification at explicit subject-specific achieved depths, with precise refusals, immutable reports and bounded costs. | Implemented on the PR branch | The candidate implements subject-specific catalog, segment, record, blob and retained-namespace reports, typed durable ingress outcomes and bounded conflict evidence; focused runtime/calibration and catalog-ceiling evidence are recorded. Exact-head validation, independent acceptance and mainline integration are recorded on [PR #165](https://github.com/flyingrobots/keep/pull/165); implementation does not imply merged delivery. See the [closure ledger](../../audits/114-durable-verification-scope.md) and [execution evidence](../../testing-evidence/durable-verification.md). |
