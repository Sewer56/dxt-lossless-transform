#![allow(dead_code)]
#![allow(unused_variables)]
use criterion::{black_box, BenchmarkId};
use dxt_lossless_transform_common::transforms::split_color_endpoints::avx2_shuf_impl;
use safe_allocator_api::RawAlloc;

fn bench_avx2_shuf(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        avx2_shuf_impl(
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
    group.bench_with_input(BenchmarkId::new("avx2 shuffle", size), &size, |b, _| {
        bench_avx2_shuf(b, input, output)
    });

    if !important_benches_only {}
}
