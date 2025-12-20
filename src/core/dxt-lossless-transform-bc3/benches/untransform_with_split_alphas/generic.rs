use crate::SplitAlphasInputBuffers;
use criterion::BenchmarkId;
use dxt_lossless_transform_bc3::bench::untransform::with_split_alphas::generic_untransform_with_split_alphas;
use safe_allocator_api::RawAlloc;
use std::hint::black_box;

fn bench_generic(
    b: &mut criterion::Bencher,
    input: &SplitAlphasInputBuffers,
    output: &mut RawAlloc,
    block_count: usize,
) {
    b.iter(|| unsafe {
        generic_untransform_with_split_alphas(
            black_box(input.alpha0.as_ptr()),
            black_box(input.alpha1.as_ptr()),
            black_box(input.alpha_indices.as_ptr() as *const u16),
            black_box(input.colors.as_ptr() as *const u32),
            black_box(input.color_indices.as_ptr() as *const u32),
            black_box(output.as_mut_ptr()),
            black_box(block_count),
        )
    });
}

pub(crate) fn run_benchmarks(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    input: &SplitAlphasInputBuffers,
    output: &mut RawAlloc,
    size: usize,
    block_count: usize,
    important_benches_only: bool,
) {
    group.bench_with_input(BenchmarkId::new("generic", size), &size, |b, _| {
        bench_generic(b, input, output, block_count)
    });

    if !important_benches_only {}
}
