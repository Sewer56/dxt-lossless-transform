use criterion::{black_box, BenchmarkId};
use dxt_lossless_transform::raw::transform::*;
use safe_allocator_api::RawAlloc;

fn bench_avx2_gather(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        gather(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

#[cfg(target_arch = "x86_64")]
fn bench_avx2_gather_unroll_4(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        gather_unroll_4(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_avx2_permute(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        permute(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_avx2_permute_unroll_2(
    b: &mut criterion::Bencher,
    input: &RawAlloc,
    output: &mut RawAlloc,
) {
    b.iter(|| unsafe {
        permute_unroll_2(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

#[cfg(target_arch = "x86_64")]
fn bench_avx2_permute_unroll_4(
    b: &mut criterion::Bencher,
    input: &RawAlloc,
    output: &mut RawAlloc,
) {
    b.iter(|| unsafe {
        permute_unroll_4(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_avx2_shuffle_permute(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        shuffle_permute(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_avx2_shuffle_permute_unroll_2(
    b: &mut criterion::Bencher,
    input: &RawAlloc,
    output: &mut RawAlloc,
) {
    b.iter(|| unsafe {
        shuffle_permute_unroll_2(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

#[cfg(target_arch = "x86_64")]
fn bench_avx2_shuffle_permute_unroll_4(
    b: &mut criterion::Bencher,
    input: &RawAlloc,
    output: &mut RawAlloc,
) {
    b.iter(|| unsafe {
        shuffle_permute_unroll_4(
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
    // Fastest shuffle_permute
    #[cfg(target_arch = "x86_64")]
    group.bench_with_input(
        BenchmarkId::new("avx2 shuffle_permute unroll 4", size),
        &size,
        |b, _| bench_avx2_shuffle_permute_unroll_4(b, input, output),
    );

    group.bench_with_input(
        BenchmarkId::new("avx2 shuffle_permute unroll 2", size),
        &size,
        |b, _| bench_avx2_shuffle_permute_unroll_2(b, input, output),
    );

    // Gather is slow so I don't care for x86

    #[cfg(target_arch = "x86_64")]
    group.bench_with_input(
        BenchmarkId::new("avx2 gather unroll 4", size),
        &size,
        |b, _| bench_avx2_gather_unroll_4(b, input, output),
    );

    // Fastest permute

    #[cfg(target_arch = "x86_64")]
    group.bench_with_input(
        BenchmarkId::new("avx2 permute unroll 4", size),
        &size,
        |b, _| bench_avx2_permute_unroll_4(b, input, output),
    );

    group.bench_with_input(
        BenchmarkId::new("avx2 permute unroll 2", size),
        &size,
        |b, _| bench_avx2_permute_unroll_2(b, input, output),
    );

    if !important_benches_only {
        group.bench_with_input(BenchmarkId::new("avx2 gather", size), &size, |b, _| {
            bench_avx2_gather(b, input, output)
        });

        group.bench_with_input(
            BenchmarkId::new("avx2 shuffle_permute", size),
            &size,
            |b, _| bench_avx2_shuffle_permute(b, input, output),
        );

        group.bench_with_input(BenchmarkId::new("avx2 permute", size), &size, |b, _| {
            bench_avx2_permute(b, input, output)
        });
    }
}
