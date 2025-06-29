use criterion::{black_box, BenchmarkId};
use dxt_lossless_transform_bc3::transforms::standard::untransform::bench::*;
use safe_allocator_api::RawAlloc;

fn bench_portable32(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        u32_untransform(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_portable32_v2(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        u32_untransform_v2(
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
        BenchmarkId::new("portable32 no-unroll", size),
        &size,
        |b, _| bench_portable32(b, input, output),
    );

    group.bench_with_input(BenchmarkId::new("portable32 v2", size), &size, |b, _| {
        bench_portable32_v2(b, input, output)
    });

    if !important_benches_only {}
}
