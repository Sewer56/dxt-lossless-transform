use crate::SplitAlphasOutputBuffers;
use criterion::BenchmarkId;
use dxt_lossless_transform_bc3::bench::transform::with_split_alphas::avx2_transform_with_split_alphas;
use safe_allocator_api::RawAlloc;
use std::hint::black_box;

fn bench_avx2(
    b: &mut criterion::Bencher,
    input: &RawAlloc,
    output: &mut SplitAlphasOutputBuffers,
    block_count: usize,
) {
    b.iter(|| unsafe {
        avx2_transform_with_split_alphas(
            black_box(input.as_ptr()),
            black_box(output.alpha0.as_mut_ptr()),
            black_box(output.alpha1.as_mut_ptr()),
            black_box(output.alpha_indices.as_mut_ptr() as *mut u16),
            black_box(output.colors.as_mut_ptr() as *mut u32),
            black_box(output.color_indices.as_mut_ptr() as *mut u32),
            black_box(block_count),
        )
    });
}

pub(crate) fn run_benchmarks(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    input: &RawAlloc,
    output: &mut SplitAlphasOutputBuffers,
    size: usize,
    block_count: usize,
    important_benches_only: bool,
) {
    group.bench_with_input(BenchmarkId::new("avx2", size), &size, |b, _| {
        bench_avx2(b, input, output, block_count)
    });

    if !important_benches_only {}
}
