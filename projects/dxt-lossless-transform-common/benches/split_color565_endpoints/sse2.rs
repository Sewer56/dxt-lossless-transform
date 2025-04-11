#![allow(dead_code)]
#![allow(unused_variables)]
use criterion::{black_box, BenchmarkId};
use dxt_lossless_transform_common::transforms::split_565_color_endpoints::{
    sse2_shift_impl, sse2_shuf_impl, sse2_shuf_unroll2_impl,
};
use safe_allocator_api::RawAlloc;

// Placeholder for future SSE2 implementation
fn bench_sse2_shift(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        sse2_shift_impl(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_sse2_shuf(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        sse2_shuf_impl(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_sse2_shuf_unroll_2(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        sse2_shuf_unroll2_impl(
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
    // Uncomment and adjust when SSE2 implementations are available
    group.bench_with_input(BenchmarkId::new("sse2 no-unroll", size), &size, |b, _| {
        bench_sse2_shift(b, input, output)
    });

    group.bench_with_input(BenchmarkId::new("sse2 shuf", size), &size, |b, _| {
        bench_sse2_shuf(b, input, output)
    });

    group.bench_with_input(
        BenchmarkId::new("sse2 shuf unroll 2", size),
        &size,
        |b, _| bench_sse2_shuf_unroll_2(b, input, output),
    );

    if !important_benches_only {}
}
