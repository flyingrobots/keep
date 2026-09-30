# Verification Requirements

This ledger maps the verification vocabulary to stable laws and executable
evidence. A planned case is not evidence.

<!-- markdownlint-disable MD013 -->

| ID | Exact law | Evidence | Status |
| --- | --- | --- | --- |
| `KEEP-VERIFY-001` | Verification depth is one ordered enumeration; a report establishes exactly the requested depth and exposes no way to deepen it | `tests/verification_report.rs`; `VerificationReport` has private fields and crate-only construction | Implemented |
| `KEEP-VERIFY-002` | A view refuses a depth it cannot establish as `Unsupported`, naming its supported range, instead of reporting a shallower depth | `tests/verification_report.rs` | Implemented for `ReferenceStore` |
| `KEEP-VERIFY-003` | Missing, corrupt, and ambiguous evidence are distinct refusals; each carries the exact expected and observed coordinates it can | `tests/verification_report.rs`, `src/reference/verification_tests.rs`; every durable structural field's corruption maps to one exact first refusal and stage (`framing`, `checksum`, `identity`, `binding`) in `conformance/segment-store/{v1,v2}/mutations.tsv` via `tests/segment_store_mutations.rs` | Implemented for `ReferenceStore`; `Ambiguous` has no producer yet |
| `KEEP-VERIFY-004` | A lower-stage refusal is reported before a deeper one, and the single chunk pass hashes every chunk once | `tests/verification_report.rs` (profile-boundary and target contradictions succeed at `ChunkIdentity` and refuse only at `CompleteBlobIdentity`) | Implemented for `ReferenceStore` |
| `KEEP-VERIFY-005` | Verification never repairs, substitutes, quarantines, or rewrites physical state | `ReferenceStore::verify` takes `&self`; `src/reference/verification_tests.rs` observes the tampered chunk unchanged | Implemented for `ReferenceStore` |
| `KEEP-VERIFY-006` | Durable views establish `Framing`, `Checksum`, `CatalogReachability`, and `RetentionClosure` against one fenced snapshot and may report `Ambiguous` for conflicting evidence | durable read surface and snapshot laws | Planned in [#20](https://github.com/flyingrobots/keep/issues/20) |
| `KEEP-VERIFY-007` | A durable, replayable verification receipt binds the subject, view coordinates, depth, and refusal classification | `CanonicalVerificationReceipt` over `keep.verification-receipt/v1` ([format](../../formats/verification-receipt-v1/README.md)); golden oracle, cross-process admission, every reference-store outcome round-tripping, a field-complete corruption matrix, and report/refusal exclusivity in `tests/verification_receipt.rs`; the `verification_receipt` fuzz target | Implemented |

<!-- markdownlint-enable MD013 -->
