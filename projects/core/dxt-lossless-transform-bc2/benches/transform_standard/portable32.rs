use criterion::{black_box, BenchmarkId};
use dxt_lossless_transform_bc2::bench::transform::standard::{u32, u32_unroll_2, u32_unroll_4};
use safe_allocator_api::RawAlloc;

fn bench_portable32(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        u32(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_portable32_unroll_2(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        u32_unroll_2(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_portable32_unroll_4(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        u32_unroll_4(
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

    if !important_benches_only {
        group.bench_with_input(
            BenchmarkId::new("portable32 unroll-2", size),
            &size,
            |b, _| bench_portable32_unroll_2(b, input, output),
        );

        group.bench_with_input(
            BenchmarkId::new("portable32 unroll-4", size),
            &size,
            |b, _| bench_portable32_unroll_4(b, input, output),
        );
    }
}
