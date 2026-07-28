//! Stable TSV rows for timed chunk-profile comparisons.

use std::io::{self, Write};

use crate::report_profile::ProfileMetrics;

pub(super) fn write(writer: &mut impl Write, profiles: &[ProfileMetrics]) -> Result<(), io::Error> {
    writeln!(
        writer,
        "profile-header\tname\tprovenance\tminimum-kib\ttarget-kib\t\
         maximum-kib\ttimed-input\tsample-count\tlogical-bytes-per-second\t\
         total-wall-time-ns\tp50-wall-time-ns\tp95-wall-time-ns\t\
         p99-wall-time-ns\ttotal-cpu-time-ns\ttotal-allocation-count\t\
         total-allocated-bytes\tpeak-live-heap-bytes\tbase-unique-chunks\t\
         base-materialized-bytes\tinsertion-reused-chunks\t\
         deletion-reused-chunks\tneighbor-reused-chunks"
    )?;
    for metrics in profiles {
        let (minimum, target, maximum) = metrics.profile.bounds_kib();
        writeln!(
            writer,
            "profile\t{}\t{}\t{}\t{}\t{}\tlarge-text\t{}\t{}\t{}\t{}\t{}\t\
             {}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            metrics.profile.name(),
            metrics.profile.provenance(),
            minimum,
            target,
            maximum,
            metrics.sample_count,
            metrics.logical_bytes_per_second,
            metrics.total_wall_time_ns,
            metrics.p50_wall_time_ns,
            metrics.p95_wall_time_ns,
            metrics.p99_wall_time_ns,
            metrics.total_cpu_time_ns,
            metrics.total_allocation_count,
            metrics.total_allocated_bytes,
            metrics.peak_live_heap_bytes,
            metrics.base_unique_chunks,
            metrics.base_materialized_bytes,
            metrics.insertion_reused_chunks,
            metrics.deletion_reused_chunks,
            metrics.neighbor_reused_chunks,
        )?;
    }
    Ok(())
}
