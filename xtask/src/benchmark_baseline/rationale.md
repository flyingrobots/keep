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

The optimized subprocess has a fixed policy of 100 measured samples and five
warmups, matching `benchmark/src/main.rs`. Metadata records that exact policy;
every scenario and profile row repeats the same sample count. Policy changes
require a coordinated update to the runner and admission contract. Process CPU
clock, incremental live-heap memory, mandatory verification, nanoseconds, bytes
and exact numerator/denominator ratios are admitted as fixed semantic values.

Metric relationships are admitted with typed expected/observed failures. Ratio
numerators and denominators must equal their named counters; reused chunks may
not exceed their base or observed chunk counts. Percentiles must be ordered
and may not exceed the aggregate total. Throughput is the exact integer quotient
of logical bytes times samples times one billion divided by wall duration;
overflow and zero duration refuse. The frozen profile timed input is one MiB.
The decoder preserves its original integer parse error through the error chain.

Admission runs before artifact publication. A grammar refusal performs no
filesystem mutation. Existing publication and recovery ordering are unchanged;
these checks introduce no durable format version or public API change.

Issue #142 remains open until numeric-width bounds, bounded
parser fuzzing, a validated publication input and prior-artifact preservation
laws are complete. Structural admission alone does not prove those semantics.
