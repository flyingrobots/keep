# Streaming CAS Baseline v1

## Status

This document defines `keep.streaming-cas-baseline/v1`, the reproducible
performance-evidence protocol for Keep's non-durable streaming reference CAS.
It is implemented by the non-published `keep-benchmark` workspace crate and
the repository-owned `cargo xtask benchmark-baseline` command.

The reference evidence artifact measures source commit
`c529c07f385b5bcd76a4e57c1987001d496f9135` on
`aarch64-apple-darwin` with Rust 1.96.0:

- [c529c07-aarch64-apple-darwin.tsv](../../../benchmark/baselines/c529c07-aarch64-apple-darwin.tsv)

This single-host result is a methodology and baseline witness, not a marketing
claim, portability claim, optimization mandate, or correctness proof.

## Run the baseline

From the repository root:

```console
cargo xtask benchmark-baseline
```

The command:

1. refuses ambient code-generation settings and Cargo configuration outside
   the source-bound repository;
2. captures `HEAD`, complete tracked and untracked worktree state, `rustc`
   version, target triple, operating system and kernel, CPU model, and logical
   CPU count, then refuses a dirty source tree;
3. runs the benchmark executable through locked Cargo in release mode for the
   captured host target;
4. collects exactly 100 timed samples after five untimed warmups;
5. bounds standard output to 1 MiB and diagnostics to 256 KiB while draining
   both pipes concurrently;
6. recaptures every source, compiler, and host coordinate and refuses any drift
   during compilation or measurement;
7. validates the schema, captured coordinates, optimized build marker,
   scenario and profile cardinalities, and threshold policy;
8. acquires a single-writer lock, recovers an interrupted stage, and
   atomically publishes `target/benchmark/streaming-cas-baseline-v1.tsv`.

Tracked-source admission reconstructs a temporary index directly from `HEAD`
and compares it with the working tree. It does not trust mutable index hints
such as `assume-unchanged` or `skip-worktree`.

The `target` artifact is intentionally ignored. Promoting evidence into
`benchmark/baselines` is a separate reviewed action so environment-sensitive
numbers never change a regression gate merely because someone ran a command.

Direct debug execution refuses before measuring. Library tests may collect
diagnostic measurements, but those rows are labeled `debug-diagnostics` and
cannot pass the optimized publication gate.

## Corpus law

Every input byte is generated from fixed repository-owned rules. No source
file, third-party project, downloaded archive, or host file enters the corpus.
One generated corpus retains at most 16 MiB.

| Member | Purpose |
| --- | --- |
| Large source-like text | Stable-boundary source workload |
| Large deterministic binary | Opaque binary workload |
| Two MiB edit base | Reuse coordinate |
| Early insertion | Shift-resilience witness |
| Early deletion | Shift-resilience witness |
| Near-neighbor substitution | Local-change reuse witness |
| Independent deterministic binary | Zero-intentional-reuse witness |
| 256 tiny blobs | Metadata and per-operation overhead witness |

Logical identities for all named members are frozen in tests. Insertion,
deletion, and substitution coordinates are exact. Allocation and aggregate
length arithmetic is checked before admission.

## Workload catalog

The report always contains these 13 scenarios in stable order:

1. cold ingest;
2. warm ingest;
3. repeated near-neighbor edits;
4. early insertion;
5. early deletion;
6. many tiny blobs;
7. large binary ingest;
8. high deduplication;
9. zero intentional deduplication;
10. sequential range reads;
11. deterministic random range reads;
12. authenticated whole-blob verification;
13. varied input read partitioning.

Prepared stores and deterministic range coordinates are outside timed regions.
Every timed sample must return exactly the same semantic counters. Any change
is a typed nondeterminism failure.

Verification has no disabled state. Ingest authenticates chunk and complete
blob identity. Range reads authenticate every selected complete chunk before
and during output. Whole reconstruction authenticates chunks, profile
boundaries, and the complete named blob.

## Chunking-profile comparison

The same generated source-like bytes compare:

- benchmark-only Keep FastCDC at 4/16/64 KiB;
- registered Keep FastCDC at 16/64/256 KiB;
- benchmark-only Keep FastCDC at 64/256/1,024 KiB;
- fixed 64 KiB chunking;
- git-cas default Buzhash at 64/256/1,024 KiB, pinned to
  `432c5d9effb12c9f66536f1386791bb4421f3cea`.

Each profile row times complete partition and chunk-identity calculation and
records exact base chunk count, materialized bytes, and reused identities for
the insertion, deletion, and neighbor variants. Benchmark-only profiles never
enter Keep's production profile registry.

## Metric semantics

Scenario semantic counters describe one sample because all samples must be
identical. Timing and allocation totals cover every timed sample.

| Field family | Meaning |
| --- | --- |
| Logical bytes | Caller-requested content processed |
| Physical bytes read | Complete stored chunk bytes authenticated |
| Physical bytes written | New exact chunk bytes materialized |
| Source bytes read | Bytes crossing the caller-owned `Read` boundary |
| Output bytes written | Bytes crossing the caller-owned `Write` boundary |
| Throughput | All logical sample bytes divided by total wall nanoseconds |
| p50/p95/p99 | Exact nearest-rank per-sample latency |
| CPU time | Fallible process CPU-clock duration |
| Allocation totals | Count and bytes summed over timed samples |
| Peak live heap | Maximum incremental allocator-live bytes in one sample |
| Chunk reuse | Distinct identities already visible to the operation |

Peak live heap is allocator-local incremental evidence. It is not resident set
size and excludes state prepared before the timed region.

Read amplification is
`physical-bytes-read / logical-bytes`. Write amplification is
`physical-bytes-written / logical-bytes`. The deduplication ratio is
`logical-bytes / physical-bytes-written`. Reports store the numerator and
denominator separately and never use floating-point serialization. A zero
denominator means the operation materialized no bytes; consumers must retain
that exact state instead of inventing infinity, zero, or a substitute value.

Whole-blob verification reads each complete chunk twice: once before output
and once while emitting authenticated bytes. Its expected read amplification
is therefore exactly `2 / 1`. Range-read amplification includes every complete
selected chunk in both passes, not only returned slices.

## Regression threshold policy

All version-1 performance thresholds are deliberately `unconfigured`.
The first baseline provides evidence but cannot justify a tolerance.

A future threshold proposal must first collect at least five clean,
optimized baselines on the designated runner class with the same:

- schema and corpus identities;
- scenario and profile catalogs;
- sample and warmup counts;
- target triple and measurement semantics;
- operating-system, kernel, CPU model, and logical CPU count;
- verification posture;
- compiler policy.

The proposal must publish the observed distribution, environmental variance,
selected statistic, failure threshold, and false-positive tradeoff for every
gated metric. Thresholds require ordinary code review and a separate commit.
Comparisons must refuse incompatible metadata rather than normalize unlike
runs.

No measurement may justify weakening identity calculation, validation,
verification, output ordering, checked arithmetic, recovery, or any other
correctness contract. Performance findings create optimization work; they do
not rewrite Keep's laws.
