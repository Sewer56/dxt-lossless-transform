use criterion::{black_box, BenchmarkId};
use dxt_lossless_transform_bc2::bench::transform::standard::permute_512;
use dxt_lossless_transform_bc2::bench::transform::standard::permute_512_v2;
use safe_allocator_api::RawAlloc;

#[cfg(feature = "nightly")]
fn bench_permute(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        permute_512(
            black_box(input.as_ptr()),
            black_box(output.as_mut_ptr()),
            black_box(input.len()),
        )
    });
}

fn bench_permute_v2(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        permute_512_v2(
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
        BenchmarkId::new("avx512 permute v2", size),
        &size,
        |b, _| bench_permute_v2(b, input, output),
    );

    if !important_benches_only {
        group.bench_with_input(BenchmarkId::new("avx512 permute", size), &size, |b, _| {
            bench_permute(b, input, output)
        });
    }
}
