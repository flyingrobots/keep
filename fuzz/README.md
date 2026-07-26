# Fuzzing Keep

The `keep-fuzz` workspace holds runtime fuzz targets outside the production
crate. Compile and lint them on stable Rust; run coverage-guided campaigns with
the pinned nightly and `cargo-fuzz` versions documented in
[the contributor workflow](../CONTRIBUTING.md).

## Deterministic seeds

Run:

```bash
python3 fuzz/prepare_corpus.py
```

The script materializes bounded, ignored seed files under `fuzz/corpus/`.
Parser seeds come from the canonical Golden File Worldline identity table. CDC
seeds reproduce registered minimum, natural-boundary, probe-carry,
forced-maximum, and multi-chunk witnesses.

The generated corpus is derived test state, not protocol authority. The
canonical identities, source recipes, and expected boundaries remain under
`conformance/`. Fuzzing may add or minimize files beneath `fuzz/corpus/`
without changing those authoritative fixtures.
