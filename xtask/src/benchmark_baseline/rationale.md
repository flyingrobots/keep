# Canonical benchmark-report admission

This directory owns the ingress and publication boundary for optimized
streaming-CAS baseline evidence. The report is a protocol, not arbitrary
stdout: the named profile is `keep.streaming-cas-baseline/v1`, with UTF-8,
LF framing, tab-separated fields and a final LF.

A source coordinate may occur exactly once. Identical duplicates also refuse:
accepting them would admit multiple representations and leave ambiguous
interpretations available to downstream tools. Refusals retain the coordinate
and escape controls in diagnostics.

The frozen row grammar requires all metadata keys in writer order, exact
scenario and profile headers, complete ordered catalogs, fixed row widths,
unsigned canonical decimal metrics and the explicit unconfigured threshold
policy. Decimal fields reject signs, leading zeros, empty values and u128
overflow. Scenario identity includes its verification posture; profile identity
includes provenance, chunk bounds and timed input. Historical measurements are
not imported as expected metric values. The committed baseline is a positive
compatibility fixture rather than a performance expectation.

Admission runs before artifact publication. A grammar refusal performs no
filesystem mutation. Existing publication and recovery ordering are unchanged;
these checks introduce no durable format version or public API change.

Issue #142 remains open until metadata policies, metric relationships, bounded
parser fuzzing, a validated publication input and prior-artifact preservation
laws are complete. Structural admission alone does not prove those semantics.
