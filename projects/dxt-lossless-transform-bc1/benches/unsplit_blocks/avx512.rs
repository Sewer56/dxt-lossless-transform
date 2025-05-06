use criterion::*;
use dxt_lossless_transform_bc1::split_blocks::unsplit::avx512::*;
use safe_allocator_api::RawAlloc;

fn bench_permute_512_unroll_2(b: &mut criterion::Bencher, input: &RawAlloc, output: &mut RawAlloc) {
    b.iter(|| unsafe {
        permute_512_detransform_unroll_2(
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
    _important_benches_only: bool,
) {
    group.bench_with_input(
        BenchmarkId::new("avx512 permute unroll 2", size),
        &size,
        |b, _| bench_permute_512_unroll_2(b, input, output),
    );
}
