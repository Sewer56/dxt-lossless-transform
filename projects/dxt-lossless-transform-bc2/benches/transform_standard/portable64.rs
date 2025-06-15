use safe_allocator_api::RawAlloc;

pub(crate) fn run_benchmarks(
    _group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    _input: &RawAlloc,
    _output: &mut RawAlloc,
    _size: usize,
    important_benches_only: bool,
) {
    if !important_benches_only {}
}
