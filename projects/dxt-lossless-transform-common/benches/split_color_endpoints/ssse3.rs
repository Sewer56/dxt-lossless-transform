#![allow(dead_code)]
#![allow(unused_variables)]
use criterion::{black_box, BenchmarkId};
use dxt_lossless_transform_common::transforms::split_color_endpoints::ssse3::{
    ssse3_pshufb_unroll2_impl, ssse3_pshufb_unroll4_impl,
};
use safe_allocator_api::RawAlloc;

// Benchmark for SSSE3 implementation with unroll factor of 2 (32 bytes at once)
fn bench_ssse3_unroll_2(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        ssse3_pshufb_unroll2_impl(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

// Benchmark for SSSE3 implementation with unroll factor of 4 (64 bytes at once)
fn bench_ssse3_unroll_4(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        ssse3_pshufb_unroll4_impl(
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
    group.bench_with_input(BenchmarkId::new("ssse3 unroll-2", size), &size, |b, _| {
        bench_ssse3_unroll_2(b, input, output)
    });

    group.bench_with_input(BenchmarkId::new("ssse3 unroll-4", size), &size, |b, _| {
        bench_ssse3_unroll_4(b, input, output)
    });
}
