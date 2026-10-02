# Canonical benchmark-report admission

This directory owns the ingress and publication boundary for optimized
streaming-CAS baseline evidence. The report is a protocol, not arbitrary
stdout: the named profile is `keep.streaming-cas-baseline/v1`, with UTF-8,
LF framing, tab-separated fields and a final LF.

The parser and subprocess capture share a one-MiB input ceiling. The parser
checks byte length before UTF-8 decoding or allocation-heavy metadata admission.
The exact limit reaches decoding; a larger input refuses with maximum and
observed lengths, even if its encoding is invalid. UTF-8 failures retain their
original source through the typed error chain.

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

Byte and count metrics use a portable unsigned 64-bit ceiling, including
profile chunk counts whose producer representation is `usize`. Ratio counters
share the width of the byte counters they repeat. Timing and throughput retain
unsigned 128-bit precision. The maximum counter is admitted; a larger value
refuses with the metric name, maximum and observed value before relationship
checks. The ingress makes no narrowing or saturating conversion.

Admission produces an `AdmittedReport` whose fields are private to the
admission module. It borrows the exact immutable input bytes without copying
them. Publication requires a reference to this validated type; raw bytes cannot
reach the persister directly. The borrow prevents mutation of the source bytes
between admission and publication. Admission runs before artifact publication. A grammar refusal performs no
filesystem mutation. Existing publication and recovery ordering are unchanged;
these checks introduce no durable format version or public API change.

Issue #142 remains open until final acceptance verification is complete. A refusal law verifies that a prior artifact, an
interrupted stage and the absence of a publication lock remain unchanged. Structural admission alone does not prove those semantics.

The committed report supplies an independent row-order fixture: each of its
38 adjacent-row transpositions refuses at the first exact order violation, and
each of its 18 catalog-row deletions refuses the named incomplete catalog. A
temporary metadata-order mutation causes the transposition law to fail; the
restored production parser passes both laws in debug and release. This bounded
model evidence does not replace the remaining parser fuzz campaign.

The I/O-free benchmark-report fuzz facade reuses these production parser files
and captured-coordinate types. Its dependency-free feature admits the fixed
historical seed and refuses deterministic corruptions. Seed preparation
materializes the canonical report for the registered libFuzzer target. On the
reviewed nightly and cargo-fuzz versions, a final-source bounded campaign ran
10,000 executions with seed 142, a one-MiB input cap, five-second input timeout
and one-GiB RSS limit without a failure. This is exploration evidence, not a
claim that green coverage establishes absence of malformed states.
