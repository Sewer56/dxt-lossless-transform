#![allow(dead_code)]
#![allow(unused_variables)]
use criterion::{black_box, BenchmarkId};
use dxt_lossless_transform_common::transforms::split_color_endpoints::portable64::*;
use safe_allocator_api::RawAlloc;

fn bench_portable64(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        u64(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_portable64_unroll_2(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        u64_unroll_2(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_portable64_unroll_4(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        u64_unroll_4(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_portable64_unroll_8(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        u64_unroll_8(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_portable64_mix(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        u64_mix(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_portable64_mix_unroll_2(
    b: &mut criterion::Bencher,
    input: &RawAlloc,
    output: &mut RawAlloc,
) {
    b.iter(|| unsafe {
        u64_mix_unroll_2(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_portable64_mix_unroll_4(
    b: &mut criterion::Bencher,
    input: &RawAlloc,
    output: &mut RawAlloc,
) {
    b.iter(|| unsafe {
        u64_mix_unroll_4(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_portable64_mix_unroll_8(
    b: &mut criterion::Bencher,
    input: &RawAlloc,
    output: &mut RawAlloc,
) {
    b.iter(|| unsafe {
        u64_mix_unroll_8(
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
        BenchmarkId::new("portable64 unroll-8", size),
        &size,
        |b, _| bench_portable64_unroll_8(b, input, output),
    );

    group.bench_with_input(
        BenchmarkId::new("portable64 mix unroll-2", size),
        &size,
        |b, _| bench_portable64_mix_unroll_2(b, input, output),
    );

    if !important_benches_only {
        group.bench_with_input(BenchmarkId::new("portable64", size), &size, |b, _| {
            bench_portable64(b, input, output)
        });

        group.bench_with_input(
            BenchmarkId::new("portable64 unroll-2", size),
            &size,
            |b, _| bench_portable64_unroll_2(b, input, output),
        );

        group.bench_with_input(
            BenchmarkId::new("portable64 unroll-4", size),
            &size,
            |b, _| bench_portable64_unroll_4(b, input, output),
        );

        group.bench_with_input(BenchmarkId::new("portable64 mix", size), &size, |b, _| {
            bench_portable64_mix(b, input, output)
        });

        group.bench_with_input(
            BenchmarkId::new("portable64 mix unroll-4", size),
            &size,
            |b, _| bench_portable64_mix_unroll_4(b, input, output),
        );

        group.bench_with_input(
            BenchmarkId::new("portable64 mix unroll-8", size),
            &size,
            |b, _| bench_portable64_mix_unroll_8(b, input, output),
        );
    }
}
