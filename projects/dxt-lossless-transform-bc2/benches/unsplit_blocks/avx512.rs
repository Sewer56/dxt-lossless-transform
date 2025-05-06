use criterion::{black_box, BenchmarkId};
use dxt_lossless_transform_bc2::split_blocks::unsplit::avx512::avx512_shuffle;
use safe_allocator_api::RawAlloc;

fn bench_shuffle(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        avx512_shuffle(
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
    group.bench_with_input(BenchmarkId::new("avx512 shuffle", size), &size, |b, _| {
        bench_shuffle(b, input, output)
    });

    if !important_benches_only {}
}
