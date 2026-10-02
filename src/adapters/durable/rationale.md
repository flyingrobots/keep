# Fenced durable authenticated reads

The read adapter composes the existing immutable catalog snapshot, shared reader fence, retention root admission and authenticated reconstruction cores. It performs no publication, deletion or repair. An opened store handle is a locator; only snapshot admission establishes a readable view.

A durable snapshot keeps the fence and selected catalog alive for every borrowed read. Snapshot admission verifies every manifest-selected retained closure against that catalog. Blob lookup chooses the lowest canonical retained layout identity; exact-layout reads may name any layout in the admitted catalog, including an unretained one, while the fence protects the view.

The existing catalog loader materializes selected segment bytes under the caller's aggregate byte policy. This cost is explicit in the API documentation; it is not a lazy segment reader. Reconstruction adds no whole-blob buffer. Selected roots are loaded one at a time rather than accumulating a second index of all anchors. The format bounds each root and manifest, and each root supplies traversal counters. This bounds memory without inventing another on-disk limit.

The shared crate-private chunk source must return immutable bytes throughout verification and emission. The reference map and durable catalog satisfy this requirement through owned immutable storage. Both cores preserve the verify-before-output contract and single hash pass; no public mutable or callback-provided source is admitted through this internal boundary.

Refusal and operational failure remain distinct typed sources. Missing logical content is evidenced against an admitted view; inability to open a physical segment is a catalog I/O failure. Output failures preserve the exact accepted prefix through the existing reconstruction/range errors. A receipt is constructed only after successful emission and includes the complete admitted retention head and catalog coordinates.

The writer lock and reader fence coordinate cooperating Keep operations in a managed namespace. They do not isolate arbitrary concurrent raw filesystem mutation. Existing exact-byte, identity, namespace and corruption checks remain in force, including selected-root re-admission during lookup.

Selected-root admission binds the canonical root's namespace to the selecting manifest entry in addition to its digest and generation. A canonical root with a valid complete closure still refuses if another namespace selects it; the existing root error retains typed expected and observed namespace digests. This check belongs to the shared root-read boundary so initial admission and subsequent anchor lookup enforce the same rule.

The delivery checklist and remaining evidence obligations are recorded in [the #109 evidence ledger](../../../docs/testing-evidence/durable-authenticated-reads.md). This rationale does not assert that the issue's acceptance checks are already complete.
