# PR 94 hosted benchmark evidence

This is measurement evidence for the BLAKE3 upgrade, not a new performance threshold or a claim of universal equivalence. The [successful hosted run](https://github.com/flyingrobots/keep/actions/runs/37168090008) executes the existing source-bound benchmark in Docker. The separate [validation workflow](https://github.com/flyingrobots/keep/blob/0a194bfd0b6e27acfa02ccdb007564804947fc2a/.github/workflows/pr94-benchmark-evidence.yml) is evidence orchestration; it is not part of the product candidate.

## Exact coordinates and controls

Baseline: `11bc72c41cc379944766adbdef60f046e786b820`, tree `01c712f41fac391e14bde02cd84be3150596fd85`. Candidate: `9a7a46ec6d5043f61222c2f75fc7c49a16002cd3`, tree `9de5045171b4a779456a66ebc9b7cc96125c372b`. Each Docker copy checks out the original commit from bundled history and verifies cleanliness before and after measurement; no synthetic commit or candidate source change supplies the coordinates.

The digest-pinned image is `rust@sha256:58fe97504a0e4cbba5d85599619a589923d3e779472a6fb0840d58d1c4ba99d7`. Both copies use Rust 1.96.0 and x86_64-unknown-linux-gnu on the same AMD EPYC 7763 runner, with a two-CPU quota and 6 GiB container memory limit. Build directories are separate. Preparation fetches dependencies and builds the optimized benchmark before disconnecting the Docker network; the retained container inspection reports no attached networks during measurement.

Order is baseline-1, candidate-1, candidate-2, baseline-2, bounding linear host drift without pretending that a shared hosted runner is a dedicated performance laboratory. Each report contains 100 timed samples after five warmups for each of the existing 13 scenarios and five profiles. The existing protocol reports p50/p95/p99 wall time, CPU measures, allocation measures and mandatory behavioral verification. Its performance-threshold row remains explicitly unconfigured.

## Raw results and comparison

The four files below are copied byte-for-byte from the hosted artifact. The job artifact additionally retains preparation logs, actual CPU metadata, clean-tree coordinates, image identity and full Docker resource/network inspection.

| Report | SHA-256 |
| --- | --- |
| [baseline-1.tsv](baseline-1.tsv) | `7ea622ee433cd029dea14d2aef6b9b2574bd3de165efeb3ebc76754168fc9908` |
| [candidate-1.tsv](candidate-1.tsv) | `f178cab9e8e258f9f15e57ab8af3aab51fbe8283acd2d2161de0ae8da2b25eca` |
| [candidate-2.tsv](candidate-2.tsv) | `07982178b974968e85b53c5f092cfd96f1111eeb9cc17fe806db3056e5181fca` |
| [baseline-2.tsv](baseline-2.tsv) | `1e6018d2db7419b2e966f406ad84e65199374ea57107c9dab8f23b62d6fb9c83` |

[comparison.tsv](comparison.tsv) reports `(candidate-1 statistic + candidate-2 statistic) / (baseline-1 statistic + baseline-2 statistic)` separately for each row's p50 and p99 wall time. These are ratios of averages of per-run quantiles, not pooled quantiles, confidence intervals or significance tests. All row names are included; no unfavorable result is omitted.

Across the 13 scenarios, p50 ratios range from 0.982336 to 1.030927 and p99 ratios from 0.764819 to 1.065063. Many-tiny-blobs has the largest p50 increase, about 3.1%; early-deletion has the largest scenario p99 increase, about 6.5%. Among profiles, keep-fastcdc-64-256-1024 has a p99 ratio of 1.083662. These observed increases remain visible; the run does not justify a blanket “no performance regression” statement.

Total allocation count, total allocated bytes and peak live heap agree across all four runs for every scenario and profile. This is the benchmark's incremental heap accounting, not process RSS or a whole-system memory guarantee.

## Limits and prior failures

The original local attempt refused an inherited CARGO_TARGET_DIR; after clearing it, the local ARM Docker profile refused missing CPU-model metadata. Neither attempt yielded measurements. Hosted execution uses actual supported metadata and does not relabel either refusal as a BLAKE3 failure or fabricate a local success.

This is one same-host experiment with order control and existing protocol verification. It does not supply a long-term variance baseline, p99 confidence interval, all-platform performance result, new harness calibration, physical power-loss evidence or ordinary-test resource-policy compliance. It supplies the previously missing benchmark execution/comparison evidence for this dependency upgrade. The source reviewer and existing final-head CI gates remain separate; #179's unapproved policy disposition is not waived here.
