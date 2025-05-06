#![allow(dead_code)]
#![allow(unused_variables)]
use criterion::{black_box, BenchmarkId};
use dxt_lossless_transform_common::transforms::split_565_color_endpoints::{
    avx512_impl, avx512_impl_unroll2,
};
use safe_allocator_api::RawAlloc;

fn bench_avx512_unroll2(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        avx512_impl_unroll2(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_avx512(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        avx512_impl(
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
    group.bench_with_input(BenchmarkId::new("avx512", size), &size, |b, _| {
        bench_avx512(b, input, output)
    });

    group.bench_with_input(BenchmarkId::new("avx512 unroll2", size), &size, |b, _| {
        bench_avx512_unroll2(b, input, output)
    });

    if !important_benches_only {}
}
