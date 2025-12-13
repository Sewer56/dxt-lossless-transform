use criterion::BenchmarkId;
use dxt_lossless_transform_bc2::bench::untransform::standard::sse2_shuffle;
use safe_allocator_api::RawAlloc;
use std::hint::black_box;

fn bench_shuffle(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        sse2_shuffle(
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
    group.bench_with_input(BenchmarkId::new("sse2 shuffle", size), &size, |b, _| {
        bench_shuffle(b, input, output)
    });

    if !important_benches_only {}
}
