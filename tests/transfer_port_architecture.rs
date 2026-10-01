//! Transfer capabilities point inward from both storage adapters.

#[test]
fn reference_chunk_reads_do_not_depend_on_transfer_adapters() {
    for source in [
        include_str!("../src/reference/chunk_reader.rs"),
        include_str!("../src/reference/transfer_source.rs"),
    ] {
        assert!(!source.contains("crate::adapters"));
    }
}
