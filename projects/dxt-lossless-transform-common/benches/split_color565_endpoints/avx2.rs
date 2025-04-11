#![allow(dead_code)]
#![allow(unused_variables)]
use criterion::{black_box, BenchmarkId};
use dxt_lossless_transform_common::transforms::split_565_color_endpoints::{
    avx2_shuf_impl, avx2_shuf_impl_asm, avx2_shuf_impl_unroll_2,
};
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

fn bench_avx2_shuf_asm(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        avx2_shuf_impl_asm(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_avx2_shuf_unrolled(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        avx2_shuf_impl_unroll_2(
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

    group.bench_with_input(BenchmarkId::new("avx2 shuffle asm", size), &size, |b, _| {
        bench_avx2_shuf_asm(b, input, output)
    });

    group.bench_with_input(
        BenchmarkId::new("avx2 shuffle unroll 2", size),
        &size,
        |b, _| bench_avx2_shuf_unrolled(b, input, output),
    );

    if !important_benches_only {}
}
