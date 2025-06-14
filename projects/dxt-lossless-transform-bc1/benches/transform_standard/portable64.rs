use criterion::{black_box, BenchmarkId};
use dxt_lossless_transform_bc1::transforms::standard::transform::bench::*;
use safe_allocator_api::RawAlloc;

fn bench_portable64_shift(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        shift(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_portable64_shift_with_count(
    b: &mut criterion::Bencher,
    input: &RawAlloc,
    output: &mut RawAlloc,
) {
    b.iter(|| unsafe {
        shift_with_count(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_portable64_shift_unroll_2(
    b: &mut criterion::Bencher,
    input: &RawAlloc,
    output: &mut RawAlloc,
) {
    b.iter(|| unsafe {
        shift_unroll_2(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_portable64_shift_with_count_unroll_2(
    b: &mut criterion::Bencher,
    input: &RawAlloc,
    output: &mut RawAlloc,
) {
    b.iter(|| unsafe {
        shift_with_count_unroll_2(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_portable64_shift_unroll_4(
    b: &mut criterion::Bencher,
    input: &RawAlloc,
    output: &mut RawAlloc,
) {
    b.iter(|| unsafe {
        shift_unroll_4(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_portable64_shift_with_count_unroll_4(
    b: &mut criterion::Bencher,
    input: &RawAlloc,
    output: &mut RawAlloc,
) {
    b.iter(|| unsafe {
        shift_with_count_unroll_4(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_portable64_shift_unroll_8(
    b: &mut criterion::Bencher,
    input: &RawAlloc,
    output: &mut RawAlloc,
) {
    b.iter(|| unsafe {
        shift_unroll_8(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_portable64_shift_with_count_unroll_8(
    b: &mut criterion::Bencher,
    input: &RawAlloc,
    output: &mut RawAlloc,
) {
    b.iter(|| unsafe {
        shift_with_count_unroll_8(
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
        BenchmarkId::new("portable64 shift no-unroll", size),
        &size,
        |b, _| bench_portable64_shift(b, input, output),
    );

    group.bench_with_input(
        BenchmarkId::new("portable64 shift_with_count no-unroll", size),
        &size,
        |b, _| bench_portable64_shift_with_count(b, input, output),
    );

    group.bench_with_input(
        BenchmarkId::new("portable64 shift unroll-8", size),
        &size,
        |b, _| bench_portable64_shift_unroll_8(b, input, output),
    );

    group.bench_with_input(
        BenchmarkId::new("portable64 shift_with_count unroll-8", size),
        &size,
        |b, _| bench_portable64_shift_with_count_unroll_8(b, input, output),
    );

    if !important_benches_only {
        group.bench_with_input(
            BenchmarkId::new("portable64 shift unroll-2", size),
            &size,
            |b, _| bench_portable64_shift_unroll_2(b, input, output),
        );

        group.bench_with_input(
            BenchmarkId::new("portable64 shift_with_count unroll-2", size),
            &size,
            |b, _| bench_portable64_shift_with_count_unroll_2(b, input, output),
        );

        group.bench_with_input(
            BenchmarkId::new("portable64 shift unroll-4", size),
            &size,
            |b, _| bench_portable64_shift_unroll_4(b, input, output),
        );

        group.bench_with_input(
            BenchmarkId::new("portable64 shift_with_count unroll-4", size),
            &size,
            |b, _| bench_portable64_shift_with_count_unroll_4(b, input, output),
        );
    }
}
