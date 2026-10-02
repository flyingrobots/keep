# Identity layers and chunking acceptance audit

This page owns the next five remaining-task verdicts for issue #131.
It uses the source and delivery rules in [foundations.md](foundations.md).
Binding task text is the original roadmap at
`1a586d83d5750083172d440f90e7b786d540ff0e`, lines 246–274;
inspected main is `f49cff732cf7a6e1b472decba9e4c4130990559e`.
These entries have no separate task-specific definition-of-done block;
delivery means the named decision or implementation and evidence are on main.

## T-03.1 — Decide identity layers and transition laws

**Acceptance: pass. Delivery: pass. Leave checked.**

| Obligation | Inspected evidence | Verdict |
| --- | --- | --- |
| Distinguish the five concepts | ADR-0002 Decision separately owns BlobId, LayoutId, RepresentationId, physical location, and retention reference | Pass |
| Specify permitted identity changes | ADR-0002 Transition laws table covers rechunking, repacking, key changes, tier copy, both compaction postures, catalog rebuild, and logical-byte changes | Pass |
| Preserve refusal and evidence boundaries | ADR-0002 Required refusal behavior and adversarial examples distinguish stale coordinates, representation substitution, and publication authority | Pass |
| Deliver accepted decision | ADR-0002 is Accepted and present on main | Pass |

The feature explicitly says Done **as a model**. This verdict does not
claim an implemented representation codec, encryption, or compaction.
Originally unchecked T-03.3 remains excluded. ADR-0002's unassigned-codec
statement describes what that decision assigned; the later layout format
makes its own assignment.

## T-03.2 — Assign layout codec 1

**Acceptance: pass. Delivery: pass. Leave checked.**

| Obligation | Inspected evidence | Verdict |
| --- | --- | --- |
| Assign codec 1 to keep.flat-chunks/v1 | `docs/formats/flat-chunk-layout-v1/README.md`: header and Canonical LayoutId define coordinate and assignment | Pass |
| Preserve ADR-0002's typed envelope | Same specification: KEEP:LAYOUT:ID domain, envelope version, codec, exact plan bytes and trailing checked length | Pass |
| Bound canonical plan and admission | Same specification supplies grammar, bounds, unsupported-coordinate refusals and fixtures; `src/adapters/layout_id_binary.rs` admits codec 1 only | Pass |
| Deliver exact canonical coordinate evidence | `conformance/layout/v1/` and `tests/layout_id.rs` exist on main; golden text/binary and malformed-coordinate laws executed | Pass |

Assignment does not certify all F-05 behavior. Those four completed tasks
receive separate verdicts later in the original task order.

## T-04.1 — Decide algorithm and profile record

**Acceptance: pass. Delivery: pass. Leave checked.**

| Obligation | Inspected evidence | Verdict |
| --- | --- | --- |
| Accepted deterministic boundary decision | ADR-0003 freezes Gear64/FastCDC, masks, probe-byte handling, NC2 normalization and seed zero | Pass |
| Exact first registered profile | ADR-0003: minimum 16,384, target 65,536, maximum 262,144 bytes; canonical 96-byte big-endian record | Pass |
| StorageProfileId and admission policy | ADR-0003 hashes exact profile bytes; `RegisteredStorageProfile` admits only the registered identity; `tests/storage_profile_identity.rs` verifies exact coordinate and typed unsupported-profile refusal | Pass |
| Deliver decision and independent record evidence | Accepted ADR, `conformance/cdc-profile/v1/profile-record.bin`, and independent conformance checker are on main | Pass |

The independent checker reconstructs the 96-byte field encoding and hashes
it using external b3sum; this is stronger than a codec round trip.

## T-04.2 — Implement FastCdc and ChunkId

**Acceptance: pass. Delivery: pass. Leave checked.**

| Obligation | Inspected evidence | Verdict |
| --- | --- | --- |
| Deterministic registered detector | `src/chunk/detector.rs` and `detector_feed.rs`; frozen minimum/target/maximum, masks, checked offsets, and explicit EOF | Pass |
| Exact nonempty chunk identity | `src/chunk/id.rs` and `hasher.rs`; golden identity checks and exact empty-input refusal law | Pass |
| Source-partition invariant boundaries and identities | `tests/streaming_cdc/suite.rs`: golden sources, boundary-adjacent feeds, one-byte carry, and generated partition/reconstruction/bounds laws | Pass |
| At most 4 KiB retained state and no detector heap allocation | Inline-size law and isolated allocation measurement at 16 KiB, 1 MiB, and 4 MiB; both profiles pass | Pass |
| Supply fuzz and benchmark surfaces | `fuzz/fuzz_targets/fast_cdc.rs` compares whole, bytewise and input-derived schedules plus exact coverage; `benches/streaming_cdc.rs` benchmarks whole/bytewise feeds; benchmark builds in Docker | Pass |
| Deliver implementation and evidence on main | All named files exist at inspected main | Pass |

The allocation measurement excludes caller-owned input creation and sink
allocation, matching the documented detector boundary. It measures total,
current, and peak allocation through `AllocationInfo::default()`.
The benchmark build proves the benchmark is executable, not a throughput
result. No performance change or speed claim is made. The fuzz target was
inspected; this local run did not execute a fuzz campaign.

## T-04.3 — Language-neutral corpora

**Acceptance: pass. Delivery: pass. Leave checked.**

| Obligation | Inspected evidence | Verdict |
| --- | --- | --- |
| Freeze reproducible CDC corpus | `conformance/cdc-profile/v1/`: language-neutral profile, sources, mutations, boundaries, binary Gear table and profile record, with origin and grammar | Pass |
| Freeze reproducible ChunkId corpus | `conformance/chunk-id/v1/`: exact identity manifest, domain/preimage recipe, and origin | Pass |
| Executable independent checking | `cargo xtask conformance-check` regenerates Gear table/profile and validates sources, mutations, boundaries and ChunkId digests using external b3sum; executed successfully | Pass |
| Compare production detector with frozen evidence | `every_golden_source_has_exact_boundaries_and_chunk_identities` compares the case sets and exact boundaries, then verifies each span's bytes and identity | Pass |
| Deliver both corpora and checker | Both directories and `xtask/src/protocol_conformance/` exist on main | Pass |

Language-neutral describes the fixture grammar. It does not claim a second
language implementation or certify arbitrary third-party readers.

## Executed evidence

On 2026-10-01, copy-isolated Docker used the same pinned Rust 1.96.0
main-equivalent source clone identified in [foundations.md](foundations.md).

- `streaming_cdc`: seven laws passed in debug and release.
- `streaming_cdc_memory`: one allocation law passed in debug and release.
- `storage_profile_identity`: five laws passed in debug and release.
- `layout_id`: four laws passed in debug and release.
- `cargo xtask conformance-check`: passed with b3sum 1.8.5.
- `cargo bench -p keep --bench streaming_cdc --no-run`: passed.

The next four verdicts are in [flat layout](flat-layout.md). Fourteen of
the 64 remaining checked entries now have criterion-level verdicts. The
other 50, plus full accounting of the 19 reopened entries, remain open.
