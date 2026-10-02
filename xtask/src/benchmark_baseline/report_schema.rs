//! This module owns the frozen streaming-CAS baseline v1 row grammar.
pub(super) const SCENARIO_HEADER: &str = "scenario-header\tname\tverification\tsample-count\tlogical-bytes\tphysical-bytes-read\tphysical-bytes-written\tsource-bytes-read\toutput-bytes-written\tread-amplification-numerator\tread-amplification-denominator\twrite-amplification-numerator\twrite-amplification-denominator\tdeduplication-ratio-numerator\tdeduplication-ratio-denominator\treused-unique-chunks\tchunk-instances\toperation-count\tlogical-bytes-per-second\ttotal-wall-time-ns\tp50-wall-time-ns\tp95-wall-time-ns\tp99-wall-time-ns\ttotal-cpu-time-ns\tp50-cpu-time-ns\tp95-cpu-time-ns\tp99-cpu-time-ns\ttotal-allocation-count\ttotal-allocated-bytes\tpeak-live-allocation-count\tpeak-live-heap-bytes";
pub(super) const PROFILE_HEADER: &str = "profile-header\tname\tprovenance\tminimum-kib\ttarget-kib\tmaximum-kib\ttimed-input\tsample-count\tlogical-bytes-per-second\ttotal-wall-time-ns\tp50-wall-time-ns\tp95-wall-time-ns\tp99-wall-time-ns\ttotal-cpu-time-ns\ttotal-allocation-count\ttotal-allocated-bytes\tpeak-live-heap-bytes\tbase-unique-chunks\tbase-materialized-bytes\tinsertion-reused-chunks\tdeletion-reused-chunks\tneighbor-reused-chunks";
pub(super) const METADATA_KEYS: [&str; 16] = [
    "build-profile",
    "git-commit",
    "git-tree",
    "rustc-version",
    "target-triple",
    "os-description",
    "cpu-model",
    "cpu-clock",
    "peak-memory",
    "verification",
    "timing-unit",
    "byte-unit",
    "ratio-encoding",
    "logical-cpu-count",
    "sample-count",
    "warmup-count",
];
pub(super) const SCENARIO_PREFIXES: [&str; 13] = [
    "scenario\tcold-ingest\tingest-chunk-and-blob-identity",
    "scenario\twarm-ingest\tingest-chunk-and-blob-identity",
    "scenario\trepeated-near-neighbor-edits\tingest-chunk-and-blob-identity",
    "scenario\tearly-insertion\tingest-chunk-and-blob-identity",
    "scenario\tearly-deletion\tingest-chunk-and-blob-identity",
    "scenario\tmany-tiny-blobs\tingest-chunk-and-blob-identity",
    "scenario\tlarge-binary\tingest-chunk-and-blob-identity",
    "scenario\thigh-deduplication\tingest-chunk-and-blob-identity",
    "scenario\tzero-deduplication\tingest-chunk-and-blob-identity",
    "scenario\tsequential-range-reads\tselected-complete-chunks",
    "scenario\trandom-range-reads\tselected-complete-chunks",
    "scenario\twhole-blob-verification\tchunks-profile-and-blob",
    "scenario\tvaried-input-partitioning\tingest-chunk-and-blob-identity",
];
pub(super) const PROFILE_PREFIXES: [&str; 5] = [
    "profile\tkeep-fastcdc-4-16-64\tkeep.fastcdc-gear64/v1\t4\t16\t64\tlarge-text",
    "profile\tkeep-fastcdc-16-64-256\tkeep.fastcdc-gear64/v1\t16\t64\t256\tlarge-text",
    "profile\tkeep-fastcdc-64-256-1024\tkeep.fastcdc-gear64/v1\t64\t256\t1024\tlarge-text",
    "profile\tfixed-64\tbenchmark.fixed-size/v1\t64\t64\t64\tlarge-text",
    "profile\tgit-cas-buzhash-64-256-1024\tgit-cas@432c5d9effb12c9f66536f1386791bb4421f3cea\t64\t256\t1024\tlarge-text",
];
