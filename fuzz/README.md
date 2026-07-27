# Fuzzing Keep

The `keep-fuzz` workspace holds runtime fuzz targets outside the production
crate. Compile and lint them on stable Rust; run coverage-guided campaigns with
the pinned nightly, `cargo-fuzz` version, and resource bounds in
[`campaign.env`](campaign.env).

## Campaign policy

`campaign.env` is the single reviewed source for smoke and scheduled campaign
versions and resource limits. `campaign_policy.py` parses that file without
shell substitution and refuses missing, duplicate, unknown, malformed, or
out-of-bound values.

`run_campaign.py` compares `cargo fuzz list` with the checked-in target files,
sorts the exact target names, and exercises every target even if an earlier
target fails. Run:

```bash
python3 fuzz/run_campaign.py describe --profile smoke
python3 fuzz/run_campaign.py describe --profile scheduled
```

See [the contributor workflow](../CONTRIBUTING.md) for exact local
installation and execution commands.

## Deterministic seeds

Run:

```bash
python3 fuzz/prepare_corpus.py
```

The script materializes bounded, ignored seed files under `fuzz/corpus/`.
Parser seeds come from the canonical Golden File Worldline identity table. CDC
seeds reproduce registered minimum, natural-boundary, probe-carry,
forced-maximum, and multi-chunk witnesses.

The `golden_protocol` target directly exercises the Rust `xtask` framing,
field-count, lowercase-hexadecimal, and platform-neutral path admission
primitives under the campaign's one-mebibyte input bound.

The generated corpus is derived test state, not protocol authority. The
canonical identities, source recipes, and expected boundaries remain under
`conformance/`. Fuzzing may add or minimize files beneath `fuzz/corpus/`
without changing those authoritative fixtures.

## Retained derived state

The scheduled workflow restores an evolving `fuzz/corpus/` through GitHub's
cache service, reapplies deterministic seeds, runs every target, and minimizes
each successful corpus. Every run attempt receives a unique immutable cache
key and may restore the newest compatible prefix.

The cache is disposable acceleration, not evidence or protocol authority. A
missing cache is normal absence. A failed restore is discarded before seed
preparation. GitHub may evict the cache at any time without affecting the
lawfulness of the campaign.

Before use and retention, `check_corpus.py` refuses links, nonregular files,
unknown target directories, oversized inputs, excess files, and excess total
bytes under the limits in `campaign.env`. Refused restored state is discarded;
refused post-campaign state is neither cached nor uploaded.

The workflow retains minimized corpus evidence and runtime failure artifacts
for the finite periods declared in `campaign.env`. A confirmed crash, timeout,
or out-of-memory input must be minimized and promoted into a committed
deterministic regression test; an artifact or cache alone never closes the
defect.
