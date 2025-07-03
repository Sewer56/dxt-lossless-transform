use criterion::{black_box, BenchmarkId};
use dxt_lossless_transform_bc2::bench::transform::standard::{
    shuffle_v1, shuffle_v1_unroll_2, sse2_shuffle_v2, sse2_shuffle_v3,
};
use safe_allocator_api::RawAlloc;

fn bench_shuffle_v1(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        shuffle_v1(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_shuffle_v1_unroll_2(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        shuffle_v1_unroll_2(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_shuffle_v2(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        sse2_shuffle_v2(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

#[cfg(target_arch = "x86_64")]
fn bench_shuffle_v3(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        sse2_shuffle_v3(
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
    group.bench_with_input(BenchmarkId::new("sse2 shuffle v2", size), &size, |b, _| {
        bench_shuffle_v2(b, input, output)
    });

    #[cfg(target_arch = "x86_64")]
    group.bench_with_input(BenchmarkId::new("sse2 shuffle v3", size), &size, |b, _| {
        bench_shuffle_v3(b, input, output)
    });

    if !important_benches_only {
        group.bench_with_input(BenchmarkId::new("sse2 shuffle v1", size), &size, |b, _| {
            bench_shuffle_v1(b, input, output)
        });

        group.bench_with_input(
            BenchmarkId::new("sse2 shuffle v1 unroll 2", size),
            &size,
            |b, _| bench_shuffle_v1_unroll_2(b, input, output),
        );
    }
}
