use criterion::{black_box, BenchmarkId};
use dxt_lossless_transform_bc3::transforms::standard::untransform::bench::u64_detransform;
use safe_allocator_api::RawAlloc;

fn bench_portable64(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        u64_detransform(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

pub(crate) fn run_benchmarks(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    input: &RawAlloc,
    output: &mut RawAlloc,
    size: usize,
    important_benches_only: bool,
) {
    group.bench_with_input(
        BenchmarkId::new("portable64 no-unroll", size),
        &size,
        |b, _| bench_portable64(b, input, output),
    );

    if !important_benches_only {}
}
