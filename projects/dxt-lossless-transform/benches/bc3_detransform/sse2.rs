use criterion::{black_box, BenchmarkId};
use dxt_lossless_transform::raw::bc3::detransform::u64_detransform_sse2;
use safe_allocator_api::RawAlloc;

fn bench_sse(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        u64_detransform_sse2(
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
    group.bench_with_input(BenchmarkId::new("u64 sse2", size), &size, |b, _| {
        bench_sse(b, input, output)
    });

    if !important_benches_only {}
}
