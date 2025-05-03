use criterion::{black_box, BenchmarkId};
use dxt_lossless_transform_bc2::split_blocks::split::permute_512;
use safe_allocator_api::RawAlloc;

fn bench_permute(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        permute_512(
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
    group.bench_with_input(BenchmarkId::new("avx512 permute", size), &size, |b, _| {
        bench_permute(b, input, output)
    });

    if !important_benches_only {}
}
