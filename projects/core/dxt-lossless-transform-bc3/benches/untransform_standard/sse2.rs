use criterion::BenchmarkId;
use dxt_lossless_transform_bc3::bench::untransform::standard::{
    u32_untransform_sse2, u64_untransform_sse2,
};
use safe_allocator_api::RawAlloc;
use std::hint::black_box;

fn bench_u64_sse(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        u64_untransform_sse2(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_u32_sse(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        u32_untransform_sse2(
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
        bench_u64_sse(b, input, output)
    });

    group.bench_with_input(BenchmarkId::new("u32 sse2", size), &size, |b, _| {
        bench_u32_sse(b, input, output)
    });

    if !important_benches_only {}
}
