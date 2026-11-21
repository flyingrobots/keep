# Benchmark-baseline acceptance audit

This page owns the originally completed T-09.1 verdict for issue #131.
Binding text is the original roadmap at
`1a586d83d5750083172d440f90e7b786d540ff0e`, lines 440–450.
Inspected main is `f49cff732cf7a6e1b472decba9e4c4130990559e`.
Originally unchecked T-09.2 and T-09.3 are excluded.

## T-09.1 — Corpus, scenarios, metrics and one baseline

**Named artifacts: present on main. Definition of done: not met.**

The roadmap delegates evidence to the benchmark README and supplies no
separate task-specific DoD block. The repository's parse/validate/admit and
canonical-format standards remain binding at the report publication boundary.
Correction owner: [issue #142](https://github.com/flyingrobots/keep/issues/142).

| Obligation | Inspected and executed evidence | Verdict |
| --- | --- | --- |
| Generated bounded corpus | `benchmark/src/corpus.rs`, generators and corpus tests: deterministic members, exact edit coordinates and identities, total byte limit 16,777,216 | Pass |
| Thirteen reproducible scenarios | Scenario catalog and scenario tests: frozen order, every scenario executes, deterministic semantic counters | Pass |
| Five profile comparisons | Profile tests: exact parameters and pinned provenance, exact partition coverage, edit reuse and production registered-profile identities | Pass |
| Required reference metrics | Measurement/report modules: wall/process-CPU time, nearest-rank percentiles, allocation/live-heap counters, bytes, operations and exact ratio fields | Pass for this reference scope |
| Verification mandatory; diagnostics distinguished from optimized evidence | Verification posture has no disabled state; build-profile law refuses debug publication; report metadata records posture | Pass |
| One source-bound committed baseline | `benchmark/baselines/c529c07-aarch64-apple-darwin.tsv`: existing Git commit c529c07, clean source, Rust 1.96.0, Darwin 25.3.0/M1 Pro, 100 samples/five warmups, 13 scenario and five profile rows | Artifact present; fields verified |
| Threshold policy explicit | Artifact and README mark all performance thresholds unconfigured; controlled-history work is originally unfinished T-09.2 | Pass |
| Bound captured source/compiler/host without ambiguity | Artifact validator accepts expected metadata plus a conflicting second git-commit coordinate | Fail |
| Admit complete canonical metric rows before publication | Existing accepted unit fixture contains bare scenario/index and profile/index rows, without headers or metric fields; validator checks counts only | Fail |
| Delivery | Existing implementation and historical artifact are on main; strict admission correction has not landed | Correction required |

The original artifact is a historical measurement witness, not proof that
current code has identical throughput. PR #134 has separately source-bound
single-pass evidence; cross-host timings cannot establish a speedup. This
audit neither repeats the historical timings nor invents hardware coordinates
for a new optimized baseline. Durable scenarios remain outside T-09.1.

## Reproduced failure

The production ingress is
`xtask/src/benchmark_baseline/artifact.rs::validate`. It checks that expected
metadata lines occur and that 13 scenario/five profile rows occur. It does
not exclude a conflicting metadata line or validate the complete row grammar.

In the accepted existing fixture, append:

```text
metadata<TAB>git-commit<TAB>ffffffffffffffffffffffffffffffffffffffff<LF>
```

The expected captured commit remains present as well. A Docker probe asserting
refusal failed at `conflicting source coordinates were admitted`: validation
returned success. No bad artifact was published. The probe was restored and
the source clone is clean. The first probe had a function-qualification compile
error and is not RED evidence; only the corrected running assertion is counted.

Issue #142 owns one coherent complete-report admission boundary, including
permanent typed mutation/fuzz evidence and protection of prior artifact state.
Existing positive tests and green builds do not discharge this missing law.

## Executed evidence and accounting

Copy-isolated Docker, pinned Rust 1.96.0, main-equivalent source clone
`7981988`, dedicated target directory:

- keep-benchmark: 19 unit laws and one public integration law passed in
  debug and release.
- Two actual benchmark report-admission laws passed in debug and release.
- Conflicting-source probe: one executed law failed at the intended assertion.
- An earlier `benchmark_report` filter selected zero tests and is not evidence.

No native tests, fresh optimized baseline, full-workspace validation or
performance threshold claim is made. Twenty-seven checked tasks now have
verdicts; 37 other checked tasks and 19 reopened entries still need accounting.
T-06.3, T-06.4 and T-09.1 retain acceptance/DoD/delivery gaps. No checkbox changes.
