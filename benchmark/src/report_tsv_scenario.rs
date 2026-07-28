//! Stable TSV rows for integrated scenario metrics.

use std::io::{self, Write};

use crate::ScenarioMetrics;

pub(super) fn write(
    writer: &mut impl Write,
    scenarios: &[ScenarioMetrics],
) -> Result<(), io::Error> {
    writeln!(
        writer,
        "scenario-header\tname\tverification\tsample-count\tlogical-bytes\t\
         physical-bytes-read\tphysical-bytes-written\tsource-bytes-read\t\
         output-bytes-written\tread-amplification-numerator\t\
         read-amplification-denominator\twrite-amplification-numerator\t\
         write-amplification-denominator\tdeduplication-ratio-numerator\t\
         deduplication-ratio-denominator\treused-unique-chunks\t\
         chunk-instances\toperation-count\tlogical-bytes-per-second\t\
         total-wall-time-ns\tp50-wall-time-ns\tp95-wall-time-ns\t\
         p99-wall-time-ns\ttotal-cpu-time-ns\tp50-cpu-time-ns\t\
         p95-cpu-time-ns\tp99-cpu-time-ns\ttotal-allocation-count\t\
         total-allocated-bytes\tpeak-live-allocation-count\t\
         peak-live-heap-bytes"
    )?;
    for scenario in scenarios {
        write_row(writer, scenario)?;
    }
    Ok(())
}

fn write_row(writer: &mut impl Write, metrics: &ScenarioMetrics) -> Result<(), io::Error> {
    let observed = metrics.observation();
    writeln!(
        writer,
        "scenario\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t\
         {}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t\
         {}\t{}",
        metrics.scenario().name(),
        observed.verification().name(),
        metrics.sample_count(),
        observed.logical_bytes(),
        observed.authenticated_chunk_bytes_read(),
        observed.materialized_bytes_written(),
        observed.source_bytes_read(),
        observed.output_bytes_written(),
        observed.authenticated_chunk_bytes_read(),
        observed.logical_bytes(),
        observed.materialized_bytes_written(),
        observed.logical_bytes(),
        observed.logical_bytes(),
        observed.materialized_bytes_written(),
        observed.reused_unique_chunks(),
        observed.chunk_instances(),
        observed.operation_count(),
        metrics.logical_bytes_per_second(),
        metrics.total_wall_time_ns(),
        metrics.p50_wall_time_ns(),
        metrics.p95_wall_time_ns(),
        metrics.p99_wall_time_ns(),
        metrics.total_cpu_time_ns(),
        metrics.p50_cpu_time_ns(),
        metrics.p95_cpu_time_ns(),
        metrics.p99_cpu_time_ns(),
        metrics.total_allocation_count(),
        metrics.total_allocated_bytes(),
        metrics.peak_live_allocation_count(),
        metrics.peak_live_heap_bytes(),
    )
}
